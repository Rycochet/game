//! Builds the dungeon's meshes from what is seen and lit: floors, walls and
//! stairs as textured quads, one mesh per kind of surface (each has its own
//! tile picture), with the light in their vertex colours. What is only
//! remembered goes into a second set of meshes, drawn with grey pictures.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use std::collections::BTreeSet;

use bevy::prelude::*;
use shared::dungeon::{Cell, Level, Sight};
use shared::protocol::SeenStairs;
use shared::site::{BuildingLook, FieldLook};

use super::field;
use super::holes::build_hole;
use super::liquid::build_cell;
use super::stairs::{build_stairs, hides_viewer};
use super::theme::Shape;
use super::town;
use super::walls::{Layers, Scene, build_opening, build_wall};
use super::{AMBIENT_LIGHT, CELL_SIZE};

/// The kinds of surface, each drawn with its own tile picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Floor,
    Wall,
    Door,
    Ground,
    Water,
    Lava,
    /// The wall's picture fading out across the floor beside a natural wall.
    WallBlend,
    /// A town's roads, gardens, buildings and hedges, from one picture.
    Town,
}

impl Surface {
    pub const ALL: [Self; 8] = [
        Self::Floor,
        Self::Wall,
        Self::Door,
        Self::Ground,
        Self::Water,
        Self::Lava,
        Self::WallBlend,
        Self::Town,
    ];

    /// The surface the floor of a cell is drawn with.
    pub fn of_flat(cell: Cell) -> Self {
        match cell {
            Cell::Ground
            | Cell::CoveredGround
            | Cell::Field
            | Cell::Fence
            | Cell::FieldGate
            | Cell::HerbBed => Self::Ground,
            Cell::Water => Self::Water,
            Cell::Lava => Self::Lava,
            _ => Self::Floor,
        }
    }
}

/// How much light reaches each cell of a level, as linear RGB from 0 (only
/// the dull ambient light) to 1 (fully lit).
pub struct LightGrid {
    min_x: i64,
    min_y: i64,
    width: i64,
    height: i64,
    light: Vec<Vec3>,
}

impl LightGrid {
    pub fn new(level: &Level) -> Self {
        Self::filled(level, Vec3::ZERO)
    }

    /// The same light on every cell.
    pub fn filled(level: &Level, light: Vec3) -> Self {
        let (min_x, min_y, width, height) = level.bounds();
        Self {
            min_x,
            min_y,
            width,
            height,
            light: vec![light; (width * height) as usize],
        }
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        let (col, row) = (x - self.min_x, y - self.min_y);
        ((0..self.width).contains(&col) && (0..self.height).contains(&row))
            .then_some((row * self.width + col) as usize)
    }

    /// Sets the light on a cell.
    pub fn set(&mut self, x: i64, y: i64, light: Vec3) {
        if let Some(i) = self.index(x, y) {
            self.light[i] = light;
        }
    }

    /// Adds as much of `colour` as `by` says at each cell.
    pub fn add(&mut self, colour: Vec3, by: impl Fn(i64, i64) -> f32) {
        for row in 0..self.height {
            for col in 0..self.width {
                let i = (row * self.width + col) as usize;
                let amount = by(col + self.min_x, row + self.min_y);
                self.light[i] = (self.light[i] + colour * amount).min(Vec3::ONE);
            }
        }
    }

    /// The light at a cell (none outside the level).
    pub fn at(&self, x: i64, y: i64) -> Vec3 {
        self.index(x, y).map_or(Vec3::ZERO, |i| self.light[i])
    }

    /// Adds a light at `(x, y)` reaching `radius` cells. Walls block it (the
    /// same rule as sight, from `shared`); it fades slowly at first, then
    /// faster, to nothing just beyond the radius.
    pub fn add_light(&mut self, level: &Level, (x, y): (i64, i64), radius: u32, colour: Vec3) {
        let mut reached = Sight::new(level);
        reached.add_view(level, x, y, Some(radius));
        let reach = radius as f32 + 1.0;
        let r = radius as i64;
        for cy in y - r..=y + r {
            for cx in x - r..=x + r {
                if !reached.is_seen(cx, cy) {
                    continue;
                }
                let distance = Vec2::new((cx - x) as f32, (cy - y) as f32).length();
                let strength = 1.0 - (distance / reach).powi(2);
                if let Some(i) = self.index(cx, cy) {
                    self.light[i] = (self.light[i] + colour * strength).min(Vec3::ONE);
                }
            }
        }
    }

    /// The light at a grid corner (between four cells; corner `(x, y)` is the
    /// north-west corner of cell `(x, y)`): the average over the open cells
    /// around it, so light fades smoothly across floors without leaking out
    /// of walls. `None` if no open cell touches it.
    pub fn corner(&self, level: &Level, x: i64, y: i64) -> Option<Vec3> {
        let mut total = Vec3::ZERO;
        let mut count = 0.0;
        for (cx, cy) in [(x - 1, y - 1), (x, y - 1), (x - 1, y), (x, y)] {
            if level.is_open(cx, cy) {
                total += self.at(cx, cy);
                count += 1.0;
            }
        }
        (count > 0.0).then(|| total / count)
    }
}

/// What a viewer (the player, or anything sharing its sight) sees from its
/// cell.
pub struct View {
    pub cell: (i64, i64),
    pub sight: Sight,
}

/// A piece of wall drawn solid, kept for click picking: its centre, the way
/// it faces, and half its size along the wall and up it.
#[derive(Clone, Copy, Debug)]
pub struct WallFace {
    pub centre: Vec3,
    pub normal: Vec3,
    pub half_across: f32,
    pub half_up: f32,
}

/// Anything facing up with a normal over `RAISED_NORMAL`, this far over the
/// floor, is a top (of a wall or a roof), which the mask doesn't cut, as sight
/// stops at the wall below it. Match `sight_mask.wesl`.
const RAISED: f32 = 1.0;
const RAISED_NORMAL: f32 = 0.5;

/// How bright remembered (out of sight) surfaces are drawn, on their grey
/// pictures.
pub const MEMORY_SHADE: f32 = 0.3;

/// The meshes the dungeon is drawn with.
pub struct DungeonMeshes {
    /// One per `Surface` (in `Surface::ALL` order): floors, wall sides facing
    /// the camera, stairs...
    pub solid: Vec<Mesh>,
    pub has_solid: Vec<bool>,
    /// The same for what is remembered but out of sight.
    pub memory: Vec<Mesh>,
    pub has_memory: Vec<bool>,
    /// Wall sides facing away from the camera (the wall stands between the
    /// camera and what is seen), drawn half see-through.
    pub see_through: Mesh,
    pub has_see_through: bool,
    /// Stairs in the way of a viewer, by surface (in `Surface::ALL` order),
    /// drawn half see-through.
    pub stairs_through: Vec<Mesh>,
    pub has_stairs_through: Vec<bool>,
    /// The solid wall sides, which block clicks.
    pub solid_faces: Vec<WallFace>,
}

/// Builds the meshes for everything in `sight` (all the views combined), lit
/// by `light`, and for the rest of `remembered` (which includes what is in
/// sight) in grey, with `stairs` drawn in 3D. Only remembered cells are
/// drawn: cells the client doesn't know may be in sight as wall. `camera` is
/// the camera's position, `shape` how the place's walls are built. Cells in
/// `cut` are in sight only in part, so what is lit there is copied in grey too.
#[allow(clippy::too_many_arguments)]
pub fn build(
    level: &Level,
    views: &[View],
    sight: &Sight,
    remembered: &Sight,
    light: &LightGrid,
    stairs: &[SeenStairs],
    buildings: &[BuildingLook],
    lit_buildings: &BTreeSet<u16>,
    fields: &[FieldLook],
    ruined: bool,
    camera: Vec3,
    shape: Shape,
    cut: &Sight,
) -> DungeonMeshes {
    let mut layers = Layers {
        solid: Surface::ALL.iter().map(|_| Builder::default()).collect(),
        memory: Surface::ALL.iter().map(|_| Builder::default()).collect(),
        see_through: Builder::default(),
        stairs_through: Surface::ALL.iter().map(|_| Builder::default()).collect(),
        solid_faces: Vec::new(),
    };
    let scene = Scene {
        level,
        views,
        sight,
        light,
        camera,
        shape,
        buildings,
        lit_buildings,
        fields,
        ruined,
    };
    let (min_x, min_y, width, height) = level.bounds();

    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if !remembered.is_seen(x, y) {
                continue;
            }
            let before = sizes(&layers.solid);
            build_cell_at(&mut layers, &scene, stairs, (x, y));
            if cut.is_seen(x, y) {
                copy_as_memory(&mut layers, &before);
            }
        }
    }

    // Buildings are drawn whole, not by the cell, and only the parts in sight
    // are lit, so all of those may be cut.
    let before = sizes(&layers.solid);
    town::build_buildings(&mut layers, &scene);
    copy_as_memory(&mut layers, &before);

    let Layers {
        solid,
        memory,
        see_through,
        stairs_through,
        solid_faces,
    } = layers;
    DungeonMeshes {
        has_solid: solid.iter().map(|b| !b.indices.is_empty()).collect(),
        solid: solid.into_iter().map(Builder::into_mesh).collect(),
        has_memory: memory.iter().map(|b| !b.indices.is_empty()).collect(),
        memory: memory.into_iter().map(Builder::into_mesh).collect(),
        has_see_through: !see_through.indices.is_empty(),
        see_through: see_through.into_mesh(),
        has_stairs_through: stairs_through
            .iter()
            .map(|b| !b.indices.is_empty())
            .collect(),
        stairs_through: stairs_through.into_iter().map(Builder::into_mesh).collect(),
        solid_faces,
    }
}

/// What is hidden looks like: how much of each of red, green and blue is kept and
/// added, with its opacity unchanged.
const HIDDEN_TINT: [(f32, f32); 4] = [(0.5, 0.05), (0.8, 0.3), (0.9, 0.4), (1.0, 0.0)];

/// How much each builder holds so far: its vertices and indices.
fn sizes(builders: &[Builder]) -> Vec<(usize, usize)> {
    builders
        .iter()
        .map(|builder| (builder.positions.len(), builder.indices.len()))
        .collect()
}

/// Copies what was lit since `before` into the memory meshes, in grey, for the
/// part of it the mask says is out of sight. Tops stay out: the mask doesn't cut
/// them (`Builder::copy_grey`).
fn copy_as_memory(layers: &mut Layers, before: &[(usize, usize)]) {
    for (i, &from) in before.iter().enumerate() {
        let Layers { solid, memory, .. } = &mut *layers;
        solid[i].copy_grey(from, &mut memory[i]);
    }
}

/// How many vertices each mesh holds so far.
fn layer_sizes(layers: &Layers) -> Vec<usize> {
    all_builders(layers)
        .map(|builder| builder.positions.len())
        .collect()
}

fn all_builders(layers: &Layers) -> impl Iterator<Item = &Builder> {
    layers
        .solid
        .iter()
        .chain(&layers.memory)
        .chain(std::iter::once(&layers.see_through))
}

/// Gives what was added since `before` the pale glow of something hidden, which
/// shows even in the dark.
fn tint_since(layers: &mut Layers, before: &[usize]) {
    let Layers {
        solid,
        memory,
        see_through,
        ..
    } = layers;
    let builders = solid
        .iter_mut()
        .chain(memory.iter_mut())
        .chain(std::iter::once(see_through));
    for (builder, &from) in builders.zip(before) {
        for colour in &mut builder.colours[from..] {
            for (part, (keep, add)) in colour.iter_mut().zip(HIDDEN_TINT) {
                *part = *part * keep + add;
            }
        }
    }
}

/// Whatever stands at a remembered cell.
fn build_cell_at(layers: &mut Layers, scene: &Scene, stairs: &[SeenStairs], (x, y): (i64, i64)) {
    let level = scene.level;
    let cell = level.cell(x, y);
    if cell.is_wall() {
        // A town's buildings are drawn whole, in `build`.
        if level.room(x, y).is_none() {
            let before = layer_sizes(layers);
            build_wall(layers, scene, (x, y));
            // Only sent to truesight: otherwise it arrives as a wall.
            if cell == Cell::SecretDoor {
                tint_since(layers, &before);
            }
        }
    } else if cell.is_road() || cell == Cell::Garden {
        town::build_flat(layers, scene, (x, y));
        town::build_hedges(layers, scene, (x, y));
    } else if cell.is_opening() {
        build_opening(layers, scene, (x, y));
    } else if cell == Cell::Hole {
        build_hole(layers, scene, (x, y));
    } else if cell == Cell::Field && field::build_field(layers, scene, (x, y)) {
        // Drawn: anything else of a field is the land's own ground.
    } else {
        let found = stairs.iter().find(|s| (s.x, s.y) == (x, y));
        build_open_cell(layers, scene, (x, y), found);
        field::build_fence(layers, scene, (x, y));
    }
}

/// A flat open cell, or stairs: lit if in sight, otherwise as memory.
fn build_open_cell(
    layers: &mut Layers,
    scene: &Scene,
    (x, y): (i64, i64),
    stairs: Option<&SeenStairs>,
) {
    let Scene { level, light, .. } = *scene;
    if level.cell(x, y).is_liquid() {
        build_cell(layers, scene, (x, y));
        return;
    }
    let centre = Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE);
    let surface = Surface::of_flat(level.cell(x, y));
    if !scene.sight.is_seen(x, y) {
        let grey = Vec3::splat(MEMORY_SHADE);
        match stairs {
            Some(found) => build_stairs(&mut layers.memory, None, found, surface, grey, &|_| grey),
            None => {
                let half = CELL_SIZE / 2.0;
                layers.memory[surface as usize].quad(
                    centre,
                    Vec3::Y,
                    Vec3::NEG_Z,
                    (half, half),
                    |_| grey,
                );
            }
        }
        return;
    }
    let floor_light = |p| corner_light(level, light, p).unwrap_or(light.at(x, y));
    match stairs {
        Some(found) => {
            let lit = with_ambient(light.at(x, y));
            let floor_colour = |p| with_ambient(floor_light(p));
            let through = hides_viewer(found, scene.views, scene.camera)
                .then_some(&mut layers.stairs_through[..]);
            build_stairs(
                &mut layers.solid,
                through,
                found,
                surface,
                lit,
                &floor_colour,
            );
        }
        None => {
            layers.solid[surface as usize].face(centre, Vec3::Y, CELL_SIZE / 2.0, floor_light);
        }
    }
}

/// A vertex colour: `light` on top of the dull ambient light.
pub fn with_ambient(light: Vec3) -> Vec3 {
    AMBIENT_LIGHT + (Vec3::ONE - AMBIENT_LIGHT) * light
}

/// The light at the grid corner nearest a vertex's (x, z) position.
pub(super) fn corner_light(level: &Level, light: &LightGrid, position: Vec3) -> Option<Vec3> {
    let x = (position.x / CELL_SIZE + 0.5).round() as i64;
    let y = (position.z / CELL_SIZE + 0.5).round() as i64;
    light.corner(level, x, y)
}

// Whether a wall side (centred on `centre`, facing `normal`) faces the
/// camera at `camera`: the camera is in front of the side's plane. Measured
/// from the camera's real position (not just the way it looks), so a side
/// square-on to the view turns away only once the camera passes its plane.
pub fn faces_camera(centre: Vec3, normal: Vec3, camera: Vec3) -> bool {
    normal.dot(camera - centre) >= 0.0
}

/// Collects quads into one mesh.
#[derive(Default)]
pub struct Builder {
    pub positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    pub colours: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl Builder {
    /// A square face a whole cell across, centred on `centre`, facing
    /// `normal`, textured once. `light_at` gives each corner's light from
    /// its position; the vertex colour is that light on top of the ambient.
    fn face(&mut self, centre: Vec3, normal: Vec3, half: f32, light_at: impl Fn(Vec3) -> Vec3) {
        // "Up" on the face: north on floors and tops, up on walls.
        let up = if normal == Vec3::Y {
            Vec3::NEG_Z
        } else {
            Vec3::Y
        };
        self.quad(centre, normal, up, (half, half), |p| {
            with_ambient(light_at(p))
        });
    }

    /// A rectangle `2 × half_across` by `2 × half_up` (along `up`), with each
    /// corner's vertex colour from `colour_at` its position. The picture keeps
    /// its scale (one per cell), so small parts show part of it.
    fn quad(
        &mut self,
        centre: Vec3,
        normal: Vec3,
        up: Vec3,
        (half_across, half_up): (f32, f32),
        colour_at: impl Fn(Vec3) -> Vec3,
    ) {
        // "Right" makes the corners run anticlockwise seen from the front.
        let right = up.cross(normal);
        let (r, u) = (right * half_across, up * half_up);
        let (us, vs) = (2.0 * half_across / CELL_SIZE, 2.0 * half_up / CELL_SIZE);
        let first = self.positions.len() as u32;
        let corners = [
            (centre - r - u, [0.0, vs]),
            (centre + r - u, [us, vs]),
            (centre + r + u, [us, 0.0]),
            (centre - r + u, [0.0, 0.0]),
        ];
        for (position, uv) in corners {
            let lit = colour_at(position);
            self.positions.push(position.to_array());
            self.normals.push(normal.to_array());
            self.uvs.push(uv);
            self.colours.push([lit.x, lit.y, lit.z, 1.0]);
        }
        self.indices
            .extend([0, 1, 2, 0, 2, 3].map(|corner| first + corner));
    }

    /// Any four-cornered surface, with its picture coordinates and vertex
    /// colours. The corners go round it either way: `normal` says which side
    /// is the front.
    pub fn free(
        &mut self,
        positions: [Vec3; 4],
        normal: Vec3,
        uvs: [[f32; 2]; 4],
        colours: [Vec3; 4],
    ) {
        self.faded(positions, normal, uvs, colours, [1.0; 4]);
    }

    /// A surface as `free`, with an opacity at each corner (which only shows
    /// on a blended material).
    pub fn faded(
        &mut self,
        mut positions: [Vec3; 4],
        normal: Vec3,
        mut uvs: [[f32; 2]; 4],
        mut colours: [Vec3; 4],
        mut opacity: [f32; 4],
    ) {
        if (positions[1] - positions[0])
            .cross(positions[2] - positions[0])
            .dot(normal)
            < 0.0
        {
            positions.reverse();
            uvs.reverse();
            colours.reverse();
            opacity.reverse();
        }
        let first = self.positions.len() as u32;
        for i in 0..4 {
            let lit = colours[i];
            self.positions.push(positions[i].to_array());
            self.normals.push(normal.to_array());
            self.uvs.push(uvs[i]);
            self.colours.push([lit.x, lit.y, lit.z, opacity[i]]);
        }
        self.indices
            .extend([0, 1, 2, 0, 2, 3].map(|corner| first + corner));
    }

    /// Copies what was added since `(vertices, indices)` into `into` at the
    /// even shade of memory, keeping its opacity, without any raised top: the
    /// shader never cuts those (`RAISED` in `sight_mask.wesl`), so they would
    /// show twice.
    fn copy_grey(&self, (vertices, indices): (usize, usize), into: &mut Builder) {
        let first = into.positions.len() as u32;
        for i in vertices..self.positions.len() {
            let [_, _, _, alpha] = self.colours[i];
            into.positions.push(self.positions[i]);
            into.normals.push(self.normals[i]);
            into.uvs.push(self.uvs[i]);
            into.colours
                .push([MEMORY_SHADE, MEMORY_SHADE, MEMORY_SHADE, alpha]);
        }
        for triangle in self.indices[indices..].as_chunks::<3>().0 {
            let raised = triangle.iter().all(|&i| {
                let i = i as usize;
                self.positions[i][1] > RAISED && self.normals[i][1] > RAISED_NORMAL
            });
            if !raised {
                into.indices
                    .extend(triangle.iter().map(|&i| i - vertices as u32 + first));
            }
        }
    }

    pub(super) fn into_mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colours)
        .with_inserted_indices(Indices::U32(self.indices))
    }
}

#[cfg(test)]
mod tests;
