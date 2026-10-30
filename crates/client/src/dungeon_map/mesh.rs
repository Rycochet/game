//! Builds the dungeon's meshes from what is seen and lit: floors, walls and
//! stairs as textured quads, one mesh per kind of surface (each has its own
//! tile picture), with the light in their vertex colours. What is only
//! remembered goes into a second set of meshes, drawn with grey pictures.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use shared::dungeon::{Cell, Level, Sight};
use shared::protocol::SeenStairs;
use shared::site::StairsDirection;

use super::liquid::build_cell;
use super::theme::Shape;
use super::walls::{Layers, Scene, build_opening, build_wall};
use super::{AMBIENT_LIGHT, CELL_SIZE, WALL_HEIGHT};

/// Steps in a flight of stairs.
const STAIR_STEPS: usize = 4;

/// Half the width of a staircase (across the way it runs): half a cell wide,
/// centred, with floor either side.
const STAIRS_HALF_WIDTH: f32 = CELL_SIZE / 4.0;

/// How high stairs up climb within their cell (they carry on through the
/// ceiling), and how deep the hole for stairs down is.
pub const STAIRS_UP_HEIGHT: f32 = WALL_HEIGHT * 0.75;
const STAIRS_DOWN_DEPTH: f32 = CELL_SIZE * 0.5;

/// Shading on stairs (they are unlit, so this gives them their shape):
/// risers and sides darker than treads; inside the hole, `RIM_SHADE` at the
/// top and darker the deeper, down to `DEEPEST_SHADE` at the bottom.
const RISER_SHADE: f32 = 0.6;
const SIDE_SHADE: f32 = 0.8;
const RIM_SHADE: f32 = 0.6;
const DEEPEST_SHADE: f32 = 0.05;

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
}

impl Surface {
    pub const ALL: [Self; 7] = [
        Self::Floor,
        Self::Wall,
        Self::Door,
        Self::Ground,
        Self::Water,
        Self::Lava,
        Self::WallBlend,
    ];

    /// The surface the floor of a cell is drawn with.
    pub fn of_flat(cell: Cell) -> Self {
        match cell {
            Cell::Ground => Self::Ground,
            Cell::Water => Self::Water,
            Cell::Lava => Self::Lava,
            Cell::Floor | Cell::Wall | Cell::SecretDoor | Cell::Door | Cell::Window => Self::Floor,
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
        let (min_x, min_y, width, height) = level.bounds();
        Self {
            min_x,
            min_y,
            width,
            height,
            light: vec![Vec3::ZERO; (width * height) as usize],
        }
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        let (col, row) = (x - self.min_x, y - self.min_y);
        ((0..self.width).contains(&col) && (0..self.height).contains(&row))
            .then_some((row * self.width + col) as usize)
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
    /// The solid wall sides, which block clicks.
    pub solid_faces: Vec<WallFace>,
}

/// Builds the meshes for everything in `sight` (all the views combined), lit
/// by `light`, and for the rest of `remembered` (which includes what is in
/// sight) in grey, with `stairs` drawn in 3D. Only remembered cells are
/// drawn: cells the client doesn't know may be in sight as wall. `camera` is
/// the camera's position, `shape` how the place's walls are built.
#[allow(clippy::too_many_arguments)]
pub fn build(
    level: &Level,
    views: &[View],
    sight: &Sight,
    remembered: &Sight,
    light: &LightGrid,
    stairs: &[SeenStairs],
    camera: Vec3,
    shape: Shape,
) -> DungeonMeshes {
    let mut layers = Layers {
        solid: Surface::ALL.iter().map(|_| Builder::default()).collect(),
        memory: Surface::ALL.iter().map(|_| Builder::default()).collect(),
        see_through: Builder::default(),
        solid_faces: Vec::new(),
    };
    let scene = Scene {
        level,
        views,
        sight,
        remembered,
        light,
        camera,
        shape,
    };
    let (min_x, min_y, width, height) = level.bounds();

    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if !remembered.is_seen(x, y) {
                continue;
            }
            let cell = level.cell(x, y);
            if cell.is_wall() {
                build_wall(&mut layers, &scene, (x, y));
            } else if cell.is_opening() {
                build_opening(&mut layers, &scene, (x, y));
            } else {
                let found = stairs.iter().find(|s| (s.x, s.y) == (x, y));
                build_open_cell(&mut layers, &scene, (x, y), found);
            }
        }
    }

    let Layers {
        solid,
        memory,
        see_through,
        solid_faces,
    } = layers;
    DungeonMeshes {
        has_solid: solid.iter().map(|b| !b.indices.is_empty()).collect(),
        solid: solid.into_iter().map(Builder::into_mesh).collect(),
        has_memory: memory.iter().map(|b| !b.indices.is_empty()).collect(),
        memory: memory.into_iter().map(Builder::into_mesh).collect(),
        has_see_through: !see_through.indices.is_empty(),
        see_through: see_through.into_mesh(),
        solid_faces,
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
            Some(found) => build_stairs(&mut layers.memory, found, grey, &|_| grey),
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
            build_stairs(&mut layers.solid, found, lit, &floor_colour);
        }
        None => {
            layers.solid[surface as usize].face(centre, Vec3::Y, CELL_SIZE / 2.0, floor_light);
        }
    }
}

// A flight of steps half a cell wide down the middle of the stairs' cell,
/// running the way they go down (`SeenStairs::descends`): stairs up stand
/// on the floor and climb to `STAIRS_UP_HEIGHT`; stairs down go into a hole
/// `STAIRS_DOWN_DEPTH` deep with floor either side. Stairs up are built of
/// the wall's masonry, so they stand out from the floor; the steps down are
/// floor going down between walls. `lit` is the colour at the stairs' cell
/// and `floor_colour` the colour at a point on the floor (light with the
/// ambient, or memory's grey).
fn build_stairs(
    solid: &mut [Builder],
    stairs: &SeenStairs,
    lit: Vec3,
    floor_colour: &impl Fn(Vec3) -> Vec3,
) {
    let (dx, dy) = stairs.descends;
    let centre = Vec3::new(
        stairs.x as f32 * CELL_SIZE,
        0.0,
        stairs.y as f32 * CELL_SIZE,
    );
    let descends = Vec3::new(dx as f32, 0.0, dy as f32);
    let across = Vec3::Y.cross(descends);
    let half = CELL_SIZE / 2.0;
    let width = STAIRS_HALF_WIDTH;
    let run = CELL_SIZE / STAIR_STEPS as f32;
    let [floor, wall] = [Surface::Floor as usize, Surface::Wall as usize];

    match stairs.direction {
        StairsDirection::Up => {
            // On the whole floor, climbing against the way down.
            solid[floor].quad(centre, Vec3::Y, Vec3::NEG_Z, (half, half), floor_colour);
            let forward = -descends;
            let rise = STAIRS_UP_HEIGHT / STAIR_STEPS as f32;
            for step in 0..STAIR_STEPS {
                let i = step as f32;
                let along = -half + (i + 0.5) * run;
                let top = (i + 1.0) * rise;
                let at = centre + forward * along;
                solid[wall].rect(at + Vec3::Y * top, Vec3::Y, forward, width, run / 2.0, lit);
                let front = centre + forward * (-half + i * run) + Vec3::Y * (top - rise / 2.0);
                let riser = lit * RISER_SHADE;
                solid[wall].rect(front, -forward, Vec3::Y, width, rise / 2.0, riser);
                for side in [across, -across] {
                    let middle = at + side * width + Vec3::Y * (top / 2.0);
                    let shade = lit * SIDE_SHADE;
                    solid[wall].rect(middle, side, Vec3::Y, run / 2.0, top / 2.0, shade);
                }
            }
            let back = centre + forward * half + Vec3::Y * (STAIRS_UP_HEIGHT / 2.0);
            let shade = lit * SIDE_SHADE;
            solid[wall].rect(back, forward, Vec3::Y, width, STAIRS_UP_HEIGHT / 2.0, shade);
        }
        StairsDirection::Down => {
            // Floor either side of the hole.
            let strip = (half - width) / 2.0;
            for side in [across, -across] {
                let middle = centre + side * (width + strip);
                solid[floor].quad(middle, Vec3::Y, descends, (strip, half), floor_colour);
            }
            let forward = descends;
            let drop = STAIRS_DOWN_DEPTH / STAIR_STEPS as f32;
            // Darker the deeper.
            let at_height = move |y: f32| {
                let depth = -y / STAIRS_DOWN_DEPTH;
                lit * (RIM_SHADE + (DEEPEST_SHADE - RIM_SHADE) * depth)
            };
            for step in 0..STAIR_STEPS {
                let i = step as f32;
                let top = -(i + 1.0) * drop;
                let shade = at_height(top);
                let at = centre + forward * (-half + (i + 0.5) * run);
                let tread = at + Vec3::Y * top;
                solid[floor].rect(tread, Vec3::Y, forward, width, run / 2.0, shade);
                if step + 1 < STAIR_STEPS {
                    // The drop to the next step down, facing down the stairs.
                    let edge = centre + forward * (-half + (i + 1.0) * run);
                    let middle = edge + Vec3::Y * (top - drop / 2.0);
                    let riser = shade * RISER_SHADE;
                    solid[wall].rect(middle, forward, Vec3::Y, width, drop / 2.0, riser);
                }
            }
            // The sides of the hole, facing in (the near ones face away from
            // the camera, so they are culled and don't hide the steps): the
            // long sides beside the steps, and the short ones at each end.
            let depth = STAIRS_DOWN_DEPTH / 2.0;
            for (outwards, reach, size) in [
                (across, width, (half, depth)),
                (-across, width, (half, depth)),
                (forward, half, (width, depth)),
                (-forward, half, (width, depth)),
            ] {
                let middle = centre + outwards * reach - Vec3::Y * depth;
                solid[wall].quad(middle, -outwards, Vec3::Y, size, |p| at_height(p.y));
            }
        }
    }
}

/// A vertex colour: `light` on top of the dull ambient light.
pub fn with_ambient(light: Vec3) -> Vec3 {
    AMBIENT_LIGHT + (Vec3::ONE - AMBIENT_LIGHT) * light
}

/// The light at the grid corner nearest a vertex's (x, z) position.
fn corner_light(level: &Level, light: &LightGrid, position: Vec3) -> Option<Vec3> {
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
    colours: Vec<[f32; 4]>,
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

    /// A rectangle `2 × half_across` by `2 × half_up` (along `up`), in
    /// `colour` (the vertex colour: light with the ambient, and any shading).
    /// The picture keeps its scale (one per cell), so small parts show part
    /// of it.
    fn rect(
        &mut self,
        centre: Vec3,
        normal: Vec3,
        up: Vec3,
        half_across: f32,
        half_up: f32,
        colour: Vec3,
    ) {
        self.quad(centre, normal, up, (half_across, half_up), |_| colour);
    }

    /// A rectangle as `rect`, with each corner's vertex colour from
    /// `colour_at` its position.
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

    fn into_mesh(self) -> Mesh {
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
mod tests {
    use super::*;

    /// What each viewer sees from its cell.
    fn views_from(level: &Level, cells: &[(i64, i64)]) -> Vec<View> {
        cells
            .iter()
            .map(|&(x, y)| {
                let mut sight = Sight::new(level);
                sight.add_view(level, x, y, None);
                View {
                    cell: (x, y),
                    sight,
                }
            })
            .collect()
    }

    /// The solid faces of a level seen by viewers, with the camera straight
    /// above a point well to the south, so it sees everything facing south.
    fn faces_seen(level: &Level, cells: &[(i64, i64)]) -> Vec<WallFace> {
        let views = views_from(level, cells);
        let mut sight = Sight::new(level);
        for view in &views {
            sight.add_view(level, view.cell.0, view.cell.1, None);
        }
        let light = LightGrid::new(level);
        let camera = Vec3::new(0.0, 600.0, 2000.0);
        build(
            level,
            &views,
            &sight,
            &sight,
            &light,
            &[],
            camera,
            Shape::Built,
        )
        .solid_faces
    }

    /// A row of wall with floor all round it, in a big room.
    fn room_with_wall() -> Level {
        let mut level = Level::new(-8, -8, 17, 17);
        for y in -7..=7 {
            for x in -7..=7 {
                level.set(x, y, Cell::Floor);
            }
        }
        for x in -2..=2 {
            level.set(x, 0, Cell::Wall);
        }
        level
    }

    #[test]
    fn only_the_side_of_a_wall_facing_a_viewer_is_drawn() {
        let level = room_with_wall();
        let thin = CELL_SIZE * 0.05;
        // From the south, the wall's south faces show.
        let on_wall =
            |face: &&WallFace| face.centre.z.abs() < CELL_SIZE && face.centre.x.abs() < 24.0;
        let south = faces_seen(&level, &[(0, 3)]);
        let south: Vec<_> = south.iter().filter(on_wall).collect();
        assert!(!south.is_empty());
        for face in south {
            assert_eq!(face.normal, Vec3::Z);
            assert!((face.centre.z - thin).abs() < 0.01);
        }
        // From the north it shows none (those faces are turned from the
        // camera, so aren't solid).
        let north = faces_seen(&level, &[(0, -3)]);
        assert_eq!(north.iter().filter(on_wall).count(), 0);
    }

    /// A viewer in line with a wall sees both its sides, both see-through,
    /// whichever way the camera is.
    #[test]
    fn walls_seen_side_on_are_drawn_see_through() {
        let mut level = room_with_wall();
        for x in -2..=2 {
            level.set(x, 0, Cell::Floor);
        }
        for y in -5..=-1 {
            level.set(0, y, Cell::Wall);
        }
        let views = views_from(&level, &[(0, 3)]);
        let sight = &views[0].sight;
        let light = LightGrid::new(&level);
        // East of the wall's line, so its east sides face the camera.
        let camera = Vec3::new(500.0, 600.0, 900.0);
        let shape = Shape::Built;
        let meshes = build(&level, &views, sight, sight, &light, &[], camera, shape);
        assert!(meshes.has_see_through);
        let on_the_wall = meshes
            .solid_faces
            .iter()
            .filter(|face| face.centre.x.abs() < CELL_SIZE)
            .filter(|face| (-6.0 * CELL_SIZE..-CELL_SIZE / 2.0).contains(&face.centre.z));
        assert_eq!(on_the_wall.count(), 0);
    }

    /// The far side of a column, or of a wall's end, shows from round the
    /// corner: it is drawn as memory, not left as a hole.
    #[test]
    fn remembered_far_sides_are_drawn_as_memory() {
        let mut level = room_with_wall();
        for x in -2..=2 {
            level.set(x, 0, Cell::Floor);
        }
        level.set(0, 0, Cell::Wall);
        let views = views_from(&level, &[(0, 3)]);
        let sight = &views[0].sight;
        let light = LightGrid::new(&level);
        // North of the column, so its north side faces the camera.
        let camera = Vec3::new(0.0, 600.0, -900.0);
        let shape = Shape::Built;
        let north_side = |meshes: &DungeonMeshes| {
            meshes.memory[Surface::Wall as usize]
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|values| values.as_float3())
                .unwrap()
                .iter()
                .any(|p| (p[2] + CELL_SIZE * 0.05).abs() < 0.01 && p[1] < 0.01)
        };
        let meshes = build(&level, &views, sight, sight, &light, &[], camera, shape);
        assert!(north_side(&meshes));
    }

    /// The floor of a wall's cell, seen from the south, stops at the wall's
    /// foot; only natural walls fade into the floor.
    #[test]
    fn floors_stop_at_the_foot_of_the_wall() {
        use super::super::walls::half_thickness;
        let level = room_with_wall();
        let views = views_from(&level, &[(0, 3)]);
        let sight = &views[0].sight;
        let light = LightGrid::new(&level);
        let camera = Vec3::new(0.0, 600.0, 900.0);
        for shape in [Shape::Built, Shape::Natural] {
            let meshes = build(&level, &views, sight, sight, &light, &[], camera, shape);
            let positions = meshes.solid[Surface::Floor as usize]
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|values| values.as_float3())
                .unwrap();
            let foot = half_thickness(shape, 0.0);
            // The wall's cell: x within -2..=2 and z on row 0.
            let in_cell = positions
                .iter()
                .filter(|p| p[0].abs() < 2.0 * CELL_SIZE && p[2].abs() < CELL_SIZE / 2.0 - 0.01);
            let nearest = in_cell.map(|p| p[2].abs()).fold(f32::MAX, f32::min);
            assert!((nearest - foot).abs() < 0.01, "{shape:?}: {nearest}");
            let blends = meshes.has_solid[Surface::WallBlend as usize];
            assert_eq!(blends, shape == Shape::Natural);
        }
    }

    /// The ground under a solid wall is filled, so the curve never shows the
    /// background; see-through walls keep their dark band.
    #[test]
    fn the_ground_is_filled_only_under_solid_walls() {
        let level = room_with_wall();
        let views = views_from(&level, &[(0, 3)]);
        let sight = &views[0].sight;
        let light = LightGrid::new(&level);
        let shape = Shape::Natural;
        let filled = |camera: Vec3| {
            let meshes = build(&level, &views, sight, sight, &light, &[], camera, shape);
            let solid = &meshes.solid[Surface::Wall as usize];
            solid
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|values| values.as_float3())
                .is_some_and(|positions| {
                    positions
                        .iter()
                        .any(|p| p[0].abs() < 0.01 && p[1].abs() < 0.01 && p[2].abs() < 0.01)
                })
        };
        assert!(filled(Vec3::new(0.0, 600.0, 900.0)));
        assert!(!filled(Vec3::new(0.0, 600.0, -900.0)));
    }

    #[test]
    fn a_door_is_a_wall_with_a_hole() {
        let mut level = room_with_wall();
        level.set(0, 0, Cell::Door);
        let faces = faces_seen(&level, &[(0, 3)]);
        // Beside the hole and above it, not across the bottom of it.
        let at_door = |x: f32| {
            faces.iter().any(|face| {
                let across = (face.centre.x - x).abs() <= face.half_across;
                let up = (face.centre.y - 2.0).abs() <= face.half_up;
                face.normal == Vec3::Z && face.centre.z.abs() < CELL_SIZE && across && up
            })
        };
        assert!(at_door(CELL_SIZE * 0.45));
        assert!(!at_door(0.0));
        assert!(
            faces
                .iter()
                .any(|face| { face.centre.x.abs() < 0.01 && face.centre.y > WALL_HEIGHT * 0.78 })
        );
    }

    #[test]
    fn a_window_has_a_taller_sill_than_lintel() {
        let mut level = room_with_wall();
        level.set(0, 0, Cell::Window);
        let faces = faces_seen(&level, &[(0, 3)]);
        let mut over_hole: Vec<_> = faces
            .iter()
            .filter(|face| face.centre.x.abs() < 0.01 && face.normal == Vec3::Z)
            .filter(|face| face.centre.z.abs() < CELL_SIZE)
            .collect();
        over_hole.sort_by(|a, b| a.centre.y.total_cmp(&b.centre.y));
        let [sill, lintel] = over_hole[..] else {
            panic!("{over_hole:?}");
        };
        assert!(sill.half_up > lintel.half_up);
    }

    #[test]
    fn natural_walls_are_narrowest_half_way_up() {
        use super::super::walls::half_thickness;
        let middle = half_thickness(Shape::Natural, WALL_HEIGHT / 2.0);
        assert!((middle - CELL_SIZE * 0.05).abs() < 0.01);
        assert!(half_thickness(Shape::Natural, 0.0) > middle * 2.0);
        assert!(half_thickness(Shape::Natural, WALL_HEIGHT) > middle * 2.0);
        assert_eq!(half_thickness(Shape::Built, 0.0), middle);
    }

    #[test]
    fn sides_face_the_camera_by_its_position() {
        // Camera south of the room, a little east of the middle.
        let camera = Vec3::new(20.0, 600.0, 900.0);
        let north_wall = Vec3::new(0.0, 12.0, -84.0);
        assert!(faces_camera(north_wall, Vec3::Z, camera));
        assert!(!faces_camera(-north_wall, Vec3::NEG_Z, camera));
        // A west-facing side east of the camera faces it, even though the
        // camera looks a little away from it...
        let east_wall = Vec3::new(180.0, 12.0, 0.0);
        assert!(faces_camera(east_wall, Vec3::NEG_X, camera));
        // ...until the camera passes its plane.
        assert!(!faces_camera(
            east_wall,
            Vec3::NEG_X,
            Vec3::new(200.0, 600.0, 900.0)
        ));
    }

    #[test]
    fn light_fades_and_stops_at_walls() {
        let level = Level::test_room();
        let mut light = LightGrid::new(&level);
        light.add_light(&level, (0, 0), 6, Vec3::ONE);
        assert_eq!(light.at(0, 0), Vec3::ONE);
        assert!(light.at(3, 0).x < light.at(1, 0).x);
        assert!(light.at(6, 0).x > 0.0);
        assert_eq!(light.at(7, 0), Vec3::ZERO);
        // Behind the short wall.
        assert_eq!(light.at(-3, 0), Vec3::ZERO);
        // Lights add up but never go past full.
        light.add_light(&level, (1, 0), 6, Vec3::ONE);
        assert_eq!(light.at(0, 0), Vec3::ONE);
    }

    #[test]
    fn only_what_is_seen_is_built() {
        let level = Level::test_room();
        let views = views_from(&level, &[(0, 0)]);
        let light = LightGrid::new(&level);
        let camera = Vec3::new(0.0, 600.0, 900.0);
        let sight = &views[0].sight;
        let shape = Shape::Built;
        let meshes = build(&level, &views, sight, sight, &light, &[], camera, shape);
        assert!(meshes.has_solid.iter().any(|&has| has));
        // The south wall's inner sides face away from the camera.
        assert!(meshes.has_see_through);
        assert!(!meshes.solid_faces.is_empty());
        // In a plain room everything is in sight, so nothing is only
        // remembered. (The short wall above has a far side, drawn as memory.)
        let mut room = Level::new(-4, -4, 9, 9);
        for y in -3..=3 {
            for x in -3..=3 {
                room.set(x, y, Cell::Floor);
            }
        }
        let room_views = views_from(&room, &[(0, 0)]);
        let room_sight = &room_views[0].sight;
        let room_light = LightGrid::new(&room);
        let plain = build(
            &room,
            &room_views,
            room_sight,
            room_sight,
            &room_light,
            &[],
            camera,
            shape,
        );
        assert!(plain.has_memory.iter().all(|&has| !has));

        let nothing = Sight::new(&level);
        let meshes = build(&level, &[], &nothing, &nothing, &light, &[], camera, shape);
        assert!(meshes.has_solid.iter().all(|&has| !has));
        assert!(!meshes.has_see_through);

        // Remembered, but out of sight: drawn in the grey meshes only.
        let meshes = build(&level, &[], &nothing, sight, &light, &[], camera, shape);
        assert!(meshes.has_solid.iter().all(|&has| !has));
        assert!(meshes.has_memory[Surface::Floor as usize]);
        assert!(meshes.has_memory[Surface::Wall as usize]);
    }
}
