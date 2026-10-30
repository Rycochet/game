//! Draws walls, doors and windows as thin 3D shapes, by the rules in
//! `shared::dungeon` (`WallShape`, how walls join).
//!
//! A wall cell is drawn in four quarters around its centre. Each quarter is
//! part of the open space round a corner of the cell, and has its own piece of
//! floor, up to two wall faces and a piece of the wall's top. A quarter is
//! drawn only if an open cell sharing that corner is seen (or remembered), so
//! the far side of a wall, and the insides of solid rock, stay dark. The floor
//! stops at the wall's foot, and beside natural walls the wall's picture fades
//! out across it.

use bevy::prelude::*;
use shared::dungeon::{CELL_UNITS, Cell, Level, Sight, WALL_HALF_UNITS};

use super::liquid::{self, FULL_DEPTH, Outline, outline_at, surfaces_near};
use super::mesh::{
    Builder, LightGrid, MEMORY_SHADE, Surface, View, WallFace, faces_camera, with_ambient,
};
use super::theme::Shape;
use super::{CELL_SIZE, WALL_HEIGHT};

/// Half a cell.
const HALF_CELL: f32 = CELL_SIZE / 2.0;

/// Half a wall's thickness at its narrowest.
const WALL_HALF: f32 = WALL_HALF_UNITS as f32 / CELL_UNITS as f32 * CELL_SIZE;

/// Half a natural wall's thickness at the floor and at its top, where the
/// curve like a letter C is widest.
const NATURAL_WALL_HALF: f32 = CELL_SIZE * 0.2;

/// How finely a natural wall is cut up, for its curve and rough surface: up
/// its height and (on faces long enough) along it.
const NATURAL_BANDS: usize = 6;
const NATURAL_SEGMENTS: usize = 3;

/// How far a natural wall's surface wanders in and out, and how much that
/// darkens or lightens it.
const ROUGHNESS: f32 = 0.45;
const ROUGH_SHADE: f32 = 0.12;

/// How far a natural wall's picture fades out across the floor from its foot:
/// no further than a wall end's distance from the edge of its cell, so it
/// never stops abruptly at the next cell. It is cut into steps this long, and
/// lifted this far off the floor to stay in front of it.
const BLEND_WIDTH: f32 = CELL_SIZE * 0.25;
const BLEND_STEP: f32 = BLEND_WIDTH / 3.0;
const BLEND_LIFT: f32 = 0.02;

/// Half the width of the hole for a door or window (with the frame round it,
/// the cell's width).
const OPENING_HALF: f32 = CELL_SIZE * 0.4;

/// A door's height, and where an arched door's curve starts.
const DOOR_TOP: f32 = WALL_HEIGHT * 0.78;
const ARCH_START: f32 = WALL_HEIGHT * 0.5;

/// The bottom and top of a window's hole: the frame below is taller than the
/// lintel above.
const WINDOW_SILL: f32 = WALL_HEIGHT * 0.42;
const WINDOW_TOP: f32 = WALL_HEIGHT * 0.82;

/// Natural windows are a line of thin columns, as wide as the gaps between
/// them, each this wide at its narrowest and at its foot.
const PILLARS: i64 = 5;
const PILLAR_SPACING: f32 = CELL_SIZE * 0.2;
const PILLAR_HALF: f32 = CELL_SIZE * 0.05;
const PILLAR_FOOT_HALF: f32 = CELL_SIZE * 0.08;

/// Shading, since the walls are unlit: faces running north-south are a little
/// darker than those running east-west, which shows corners; the tops of walls
/// are darker still. A natural wall is lighter low down, where it leans towards
/// the light, and darker where it overhangs.
const SIDEWAYS_SHADE: f32 = 0.85;
const TOP_SHADE: f32 = 0.55;
const FRAME_SHADE: f32 = 0.7;
const NATURAL_LOW_SHADE: f32 = 1.0;
const NATURAL_HIGH_SHADE: f32 = 0.6;

/// The quarters of a cell, as the directions they lie in (`x` east, `z` south).
const QUARTERS: [(f32, f32); 4] = [(1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0)];

/// Which set of meshes something is drawn into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    /// Solid and lit.
    Solid,
    /// Half see-through and lit: faces turned away from the camera.
    SeeThrough,
    /// Grey: out of sight, remembered.
    Memory,
}

/// The meshes being built, by layer and surface.
pub struct Layers {
    pub solid: Vec<Builder>,
    pub memory: Vec<Builder>,
    pub see_through: Builder,
    /// The solid wall faces, which block clicks.
    pub solid_faces: Vec<WallFace>,
}

impl Layers {
    pub(super) fn builder(&mut self, layer: Layer, surface: Surface) -> &mut Builder {
        match layer {
            Layer::Solid => &mut self.solid[surface as usize],
            Layer::Memory => &mut self.memory[surface as usize],
            Layer::SeeThrough => &mut self.see_through,
        }
    }
}

/// What is needed to draw any wall, door or window of a level.
pub struct Scene<'a> {
    pub level: &'a Level,
    pub views: &'a [View],
    pub sight: &'a Sight,
    pub remembered: &'a Sight,
    pub light: &'a LightGrid,
    pub camera: Vec3,
    pub shape: Shape,
}

/// A rectangle of floor in a quarter of a cell, as distances from the cell's
/// centre.
#[derive(Clone, Copy)]
struct Patch {
    east: (f32, f32),
    south: (f32, f32),
}

impl Patch {
    fn contains(&self, east: f32, south: f32) -> bool {
        (self.east.0..=self.east.1).contains(&east)
            && (self.south.0..=self.south.1).contains(&south)
    }

    fn distance_to(&self, east: f32, south: f32) -> f32 {
        let dx = (self.east.0 - east).max(east - self.east.1).max(0.0);
        let dz = (self.south.0 - south).max(south - self.south.1).max(0.0);
        Vec2::new(dx, dz).length()
    }
}

/// Where the patches' edges cut a quarter along one direction.
fn cuts(foot: &[Patch], edges: impl Fn(&Patch) -> (f32, f32)) -> Vec<f32> {
    let mut cuts = vec![0.0, HALF_CELL];
    for patch in foot {
        let (low, high) = edges(patch);
        cuts.extend([low.min(HALF_CELL), high.min(HALF_CELL)]);
    }
    cuts.sort_by(f32::total_cmp);
    cuts.dedup();
    cuts
}

/// One cell being drawn. `joins` says which sides (north, east, south, west)
/// the wall runs out to: light doesn't blend across it there.
struct Tile<'a> {
    scene: &'a Scene<'a>,
    x: i64,
    y: i64,
    centre: Vec3,
    joins: [bool; 4],
}

/// Draws the wall at `(x, y)`, whose cell is remembered.
pub fn build_wall(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let Some(shape) = scene.level.wall_shape(x, y) else {
        return;
    };
    let joins = [shape.north, shape.east, shape.south, shape.west];
    let reach = shape.reach().map(units_to_size);
    let tile = Tile::new(scene, (x, y), joins);
    let turned_away = tile.turned_away(reach);
    for quarter in QUARTERS {
        tile.build_wall_quarter(layers, quarter, reach, turned_away);
    }
    // A wall's curve lets the camera see in under it from the side, as far as
    // its foot. See-through walls keep their dark band, and the far side of a
    // wall stays dark.
    if !turned_away.contains(&true) {
        for quarter in QUARTERS {
            if tile.quarter_in(scene.sight, quarter) {
                tile.fill_foot(layers, quarter, reach);
            }
        }
    }
}

/// Draws the door or window at `(x, y)`, whose cell is remembered: its floor
/// and the wall with a hole in it.
pub fn build_opening(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let east_west = scene.level.opening_runs_east_west(x, y);
    let joins = [!east_west, east_west, !east_west, east_west];
    let tile = Tile::new(scene, (x, y), joins);
    let in_sight = scene.sight.is_seen(x, y);
    let floor_layer = if in_sight {
        Layer::Solid
    } else {
        Layer::Memory
    };
    let window = scene.level.cell(x, y) == Cell::Window;
    let half = half_thickness(scene.shape, 0.0);
    // A framed window's sill stands right across; a door's wall only beside
    // the hole. Columns barely touch the floor, so it runs on under them, but
    // it fades in from the sides like a door to meet the walls beside it.
    let along = if window && scene.shape == Shape::Built {
        0.0
    } else {
        OPENING_HALF
    };
    let wall = if east_west {
        Patch {
            east: (along, HALF_CELL),
            south: (0.0, half),
        }
    } else {
        Patch {
            east: (0.0, half),
            south: (along, HALF_CELL),
        }
    };
    let columns = window && scene.shape == Shape::Natural;
    let foot: &[Patch] = if columns { &[] } else { &[wall] };
    for quarter in QUARTERS {
        tile.floor_quarter(layers, floor_layer, quarter, foot, &[wall]);
    }
    tile.build_opening_wall(layers, east_west, window, in_sight);
}

fn units_to_size(units: i64) -> f32 {
    units as f32 / CELL_UNITS as f32 * CELL_SIZE
}

/// Half a wall's thickness at a height.
pub fn half_thickness(shape: Shape, height: f32) -> f32 {
    match shape {
        Shape::Built => WALL_HALF,
        Shape::Natural => {
            let from_middle = 2.0 * height / WALL_HEIGHT - 1.0;
            WALL_HALF + (NATURAL_WALL_HALF - WALL_HALF) * from_middle * from_middle
        }
    }
}

/// How far a wall goes towards a side at a height, given how far it goes at
/// its narrowest (`reach`). It goes no further than the edge of its cell.
fn reach_at(shape: Shape, reach: f32, height: f32) -> f32 {
    if reach >= HALF_CELL {
        HALF_CELL
    } else {
        reach + half_thickness(shape, height) - WALL_HALF
    }
}

/// How a wall is lit at a height, besides the light reaching it.
fn height_shade(shape: Shape, height: f32) -> f32 {
    match shape {
        Shape::Built => 1.0,
        Shape::Natural => {
            let up = height / WALL_HEIGHT;
            NATURAL_LOW_SHADE + (NATURAL_HIGH_SHADE - NATURAL_LOW_SHADE) * up
        }
    }
}

/// The heights a face is cut at, from the floor up, between two heights.
fn heights(shape: Shape, low: f32, high: f32) -> Vec<f32> {
    let bands = match shape {
        Shape::Built => 1,
        Shape::Natural => NATURAL_BANDS,
    };
    (0..=bands)
        .map(|i| low + (high - low) * i as f32 / bands as f32)
        .collect()
}

/// A number from -1 to 1 that depends only on the numbers given: for looks,
/// so it needn't be the same as anything in `shared`.
fn noise(a: i64, b: i64, c: i64) -> f32 {
    let mixed =
        (a.wrapping_mul(73_856_093) ^ b.wrapping_mul(19_349_663) ^ c.wrapping_mul(83_492_791))
            .wrapping_mul(2_654_435_761);
    ((mixed >> 16) & 0xffff) as f32 / 32_767.5 - 1.0
}

/// The picture coordinates for a point `across` and at `height` on a face
/// (`across` from the cell's centre), so that it lines up with the next cell.
fn face_uv(across: f32, height: f32) -> [f32; 2] {
    [
        (across / CELL_SIZE + 0.5).clamp(0.0, 1.0),
        (1.0 - height / CELL_SIZE).clamp(0.0, 1.0),
    ]
}

/// The picture coordinates for a point on something flat, from the centre.
pub(super) fn flat_uv(east: f32, south: f32) -> [f32; 2] {
    [
        (east / CELL_SIZE + 0.5).clamp(0.0, 1.0),
        (south / CELL_SIZE + 0.5).clamp(0.0, 1.0),
    ]
}

impl<'a> Tile<'a> {
    fn new(scene: &'a Scene<'a>, (x, y): (i64, i64), joins: [bool; 4]) -> Self {
        Self {
            scene,
            x,
            y,
            centre: Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE),
            joins,
        }
    }

    /// The other cells around the corner of this cell in a quarter.
    fn around_corner(&self, (sx, sz): (f32, f32)) -> [(i64, i64); 3] {
        let (dx, dy) = (sx as i64, sz as i64);
        [
            (self.x + dx, self.y),
            (self.x, self.y + dy),
            (self.x + dx, self.y + dy),
        ]
    }

    /// Whether `sight` includes an open cell around the corner in a quarter.
    fn quarter_in(&self, sight: &Sight, quarter: (f32, f32)) -> bool {
        let level = self.scene.level;
        self.around_corner(quarter)
            .iter()
            .any(|&(x, y)| level.is_open(x, y) && sight.is_seen(x, y))
    }

    /// The light at a point on this cell, blending between the cell's corners
    /// except across the wall itself.
    fn light_at(&self, point: Vec3) -> Vec3 {
        let local = point - self.centre;
        let (sx, sz) = (
            if local.x < 0.0 { -1.0f32 } else { 1.0 },
            if local.z < 0.0 { -1.0f32 } else { 1.0 },
        );
        let Scene { level, light, .. } = self.scene;
        let corner = |sx: f32, sz: f32| {
            let cx = self.x + i64::from(sx > 0.0);
            let cy = self.y + i64::from(sz > 0.0);
            light.corner(level, cx, cy)
        };
        let here = corner(sx, sz).unwrap_or(light.at(self.x, self.y));
        let wall_north_south = self.joins[if sz < 0.0 { 0 } else { 2 }];
        let wall_east_west = self.joins[if sx < 0.0 { 3 } else { 1 }];
        let beyond_x = if wall_north_south {
            here
        } else {
            corner(-sx, sz).unwrap_or(here)
        };
        let beyond_z = if wall_east_west {
            here
        } else {
            corner(sx, -sz).unwrap_or(here)
        };
        let fx = (local.x.abs() / HALF_CELL).min(1.0);
        let fz = (local.z.abs() / HALF_CELL).min(1.0);
        here + (beyond_x - here) * (1.0 - fx) / 2.0 + (beyond_z - here) * (1.0 - fz) / 2.0
    }

    /// A vertex colour at a point: its light (or memory's grey), shaded.
    fn colour(&self, layer: Layer, point: Vec3, shade: f32) -> Vec3 {
        match layer {
            Layer::Memory => Vec3::splat(MEMORY_SHADE * shade),
            _ => with_ambient(self.light_at(point)) * shade,
        }
    }

    /// Adds a four-cornered surface, each corner `(position, picture
    /// coordinates, shade)`.
    fn add(
        &self,
        layers: &mut Layers,
        layer: Layer,
        surface: Surface,
        normal: Vec3,
        corners: [(Vec3, [f32; 2], f32); 4],
    ) {
        let positions = corners.map(|(position, _, _)| position);
        let uvs = corners.map(|(_, uv, _)| uv);
        let colours = corners.map(|(position, _, shade)| self.colour(layer, position, shade));
        layers
            .builder(layer, surface)
            .free(positions, normal, uvs, colours);
    }

    /// The floor in a quarter of the cell, like the open cell nearest it, up
    /// to the wall's `foot`. In natural places the wall's picture fades out
    /// across it, starting from `fade_from`.
    fn floor_quarter(
        &self,
        layers: &mut Layers,
        layer: Layer,
        (sx, sz): (f32, f32),
        foot: &[Patch],
        fade_from: &[Patch],
    ) {
        let level = self.scene.level;
        // Liquid is drawn for itself, with this floor round it.
        let surface = self
            .around_corner((sx, sz))
            .iter()
            .map(|&(x, y)| level.cell(x, y))
            .find(|cell| cell.is_open() && !cell.is_liquid())
            .map_or(Surface::Floor, Surface::of_flat);
        let point = |east: f32, south: f32| {
            let (e, s) = (sx * east, sz * south);
            (self.centre + Vec3::new(e, 0.0, s), flat_uv(e, s), 1.0)
        };
        let all = [foot, fade_from].concat();
        let across = cuts(&all, |patch| patch.east);
        let down = cuts(&all, |patch| patch.south);
        for x in across.windows(2) {
            for z in down.windows(2) {
                let middle = ((x[0] + x[1]) / 2.0, (z[0] + z[1]) / 2.0);
                if foot.iter().any(|p| p.contains(middle.0, middle.1)) {
                    continue;
                }
                let corners = [
                    point(x[0], z[0]),
                    point(x[1], z[0]),
                    point(x[1], z[1]),
                    point(x[0], z[1]),
                ];
                self.add(layers, layer, surface, Vec3::Y, corners);
                if self.scene.shape == Shape::Natural {
                    self.fade_wall(layers, layer, (sx, sz), (x, z), fade_from);
                }
            }
        }
    }

    /// The wall's picture over a piece of floor, fading out with the distance
    /// from `from`.
    fn fade_wall(
        &self,
        layers: &mut Layers,
        layer: Layer,
        (sx, sz): (f32, f32),
        (x, z): (&[f32], &[f32]),
        from: &[Patch],
    ) {
        let steps = |low: f32, high: f32| ((high - low) / BLEND_STEP).ceil().max(1.0) as usize;
        let (across, down) = (steps(x[0], x[1]), steps(z[0], z[1]));
        let at = |low: f32, high: f32, i: usize, count: usize| {
            low + (high - low) * i as f32 / count as f32
        };
        let opacity = |east: f32, south: f32| {
            from.iter()
                .map(|patch| 1.0 - patch.distance_to(east, south) / BLEND_WIDTH)
                .fold(0.0, f32::max)
        };
        let vertex = |i: usize, j: usize| {
            let (east, south) = (at(x[0], x[1], i, across), at(z[0], z[1], j, down));
            let (e, s) = (sx * east, sz * south);
            let position = self.centre + Vec3::new(e, BLEND_LIFT, s);
            (position, flat_uv(e, s), opacity(east, south))
        };
        for i in 0..across {
            for j in 0..down {
                let corners = [
                    vertex(i, j),
                    vertex(i + 1, j),
                    vertex(i + 1, j + 1),
                    vertex(i, j + 1),
                ];
                let fades = corners.map(|(_, _, fade)| fade);
                if fades.iter().all(|&fade| fade <= 0.0) {
                    continue;
                }
                let colours = corners.map(|(position, _, _)| self.colour(layer, position, 1.0));
                layers.builder(layer, Surface::WallBlend).faded(
                    corners.map(|(position, _, _)| position),
                    Vec3::Y,
                    corners.map(|(_, uv, _)| uv),
                    colours,
                    fades,
                );
            }
        }
    }

    /// Where a wall stands on the floor in a quarter: the part running one
    /// way, then what the other way adds to it (as `wall_top`).
    fn wall_foot(&self, (east_west, north_south): (usize, usize), reach: [f32; 4]) -> Vec<Patch> {
        let (z_plane, _, x_end) = self.face_at(north_south, east_west, reach, 0.0);
        let (x_plane, _, z_end) = self.face_at(east_west, north_south, reach, 0.0);
        let mut foot = vec![Patch {
            east: (0.0, x_end),
            south: (0.0, z_plane),
        }];
        if z_end > z_plane {
            foot.push(Patch {
                east: (0.0, x_plane),
                south: (z_plane, z_end),
            });
        }
        foot
    }

    /// The sides (north, east, south, west) a face towards side `facing` of
    /// a quarter runs towards: `facing` itself and the side it runs out to.
    fn quarter_sides((sx, sz): (f32, f32)) -> (usize, usize) {
        (if sx < 0.0 { 3 } else { 1 }, if sz < 0.0 { 0 } else { 2 })
    }

    /// Which stretch of wall a face is part of: all of it, unless the wall
    /// turns a corner here, when each way it runs is a stretch of its own.
    fn stretch(&self, facing: usize) -> usize {
        let north_south = self.joins[0] || self.joins[2];
        let east_west = self.joins[1] || self.joins[3];
        if north_south && east_west {
            facing % 2
        } else {
            0
        }
    }

    /// For each stretch of this wall, whether a face someone sees is turned
    /// away from the camera: then all its seen faces are see-through, so both
    /// sides of a wall always look the same.
    fn turned_away(&self, reach: [f32; 4]) -> [bool; 2] {
        let mut away = [false; 2];
        for quarter in QUARTERS {
            let (east_west, north_south) = Self::quarter_sides(quarter);
            for (facing, running) in [(north_south, east_west), (east_west, north_south)] {
                if self.face_seen(quarter, facing)
                    && let Some((middle, normal)) =
                        self.face_middle(quarter, facing, running, reach)
                    && !faces_camera(middle, normal, self.scene.camera)
                {
                    away[self.stretch(facing)] = true;
                }
            }
        }
        away
    }

    /// Whether a viewer sees a face towards side `facing` in a quarter: it
    /// sees that quarter, and isn't on the other side of the wall. Someone in
    /// line with a wall, side-on to it, sees both its sides.
    fn face_seen(&self, quarter: (f32, f32), facing: usize) -> bool {
        let (dx, dy) = [(0, -1), (1, 0), (0, 1), (-1, 0)][facing];
        self.scene.views.iter().any(|view| {
            let (vx, vy) = view.cell;
            self.quarter_in(&view.sight, quarter) && (vx - self.x) * dx + (vy - self.y) * dy >= 0
        })
    }

    /// The middle of a face and the way it faces, if the face exists.
    fn face_middle(
        &self,
        (sx, sz): (f32, f32),
        facing: usize,
        running: usize,
        reach: [f32; 4],
    ) -> Option<(Vec3, Vec3)> {
        let (plane, from, to) = self.face_at(facing, running, reach, WALL_HEIGHT / 2.0);
        if to <= from {
            return None;
        }
        let along = (from + to) / 2.0;
        let (east, south, normal) = if facing.is_multiple_of(2) {
            (sx * along, sz * plane, Vec3::Z * sz)
        } else {
            (sx * plane, sz * along, Vec3::X * sx)
        };
        Some((
            self.centre + Vec3::new(east, WALL_HEIGHT / 2.0, south),
            normal,
        ))
    }

    /// One quarter of a wall: floor, up to two faces, and its top. `reach` is
    /// how far the wall goes at its narrowest, as `[north, east, south, west]`;
    /// `turned_away` is from `Self::turned_away`.
    fn build_wall_quarter(
        &self,
        layers: &mut Layers,
        quarter: (f32, f32),
        reach: [f32; 4],
        turned_away: [bool; 2],
    ) {
        let scene = self.scene;
        let in_sight = self.quarter_in(scene.sight, quarter);
        if !in_sight && !self.quarter_in(scene.remembered, quarter) {
            return;
        }
        let (east_west, north_south) = Self::quarter_sides(quarter);
        let see_through = |facing: usize| turned_away[self.stretch(facing)];
        let faces = [
            // Facing north or south, along the part of the wall running
            // east-west, then the other way.
            self.wall_face(
                layers,
                quarter,
                in_sight,
                north_south,
                east_west,
                reach,
                see_through(north_south),
            ),
            self.wall_face(
                layers,
                quarter,
                in_sight,
                east_west,
                north_south,
                reach,
                see_through(east_west),
            ),
        ];
        // A quarter whose only faces are remembered is remembered as a whole,
        // so its floor and top match them.
        let live = faces.contains(&Some(Layer::Solid)) || faces.contains(&Some(Layer::SeeThrough));
        let remembered = !in_sight || (!live && faces.contains(&Some(Layer::Memory)));
        let foot = self.wall_foot((east_west, north_south), reach);
        let floor_layer = if remembered {
            Layer::Memory
        } else {
            Layer::Solid
        };
        match outline_at(scene.level, (self.x, self.y), quarter) {
            Some(outline) => self.wet_quarter(layers, floor_layer, quarter, outline, &foot),
            None => self.floor_quarter(layers, floor_layer, quarter, &foot, &foot),
        }
        let top_layer = if remembered {
            Layer::Memory
        } else if !faces.contains(&Some(Layer::Solid))
            && (see_through(north_south) || see_through(east_west))
        {
            Layer::SeeThrough
        } else {
            Layer::Solid
        };
        self.wall_top(layers, top_layer, quarter, (east_west, north_south), reach);
    }

    /// The ground under a wall's foot in a quarter, shaded like the wall's top.
    fn fill_foot(&self, layers: &mut Layers, quarter: (f32, f32), reach: [f32; 4]) {
        let (sx, sz) = quarter;
        for patch in self.wall_foot(Self::quarter_sides(quarter), reach) {
            let point = |east: f32, south: f32| {
                let (e, s) = (sx * east, sz * south);
                (self.centre + Vec3::new(e, 0.0, s), flat_uv(e, s), TOP_SHADE)
            };
            let corners = [
                point(patch.east.0, patch.south.0),
                point(patch.east.1, patch.south.0),
                point(patch.east.1, patch.south.1),
                point(patch.east.0, patch.south.1),
            ];
            self.add(layers, Layer::Solid, Surface::Wall, Vec3::Y, corners);
        }
    }

    /// A quarter of a wall's cell that liquid reaches: the liquid up to the
    /// wall, with the floor beside it stopping at the wall's foot and fading
    /// from it as usual. (The liquid runs on under the wall, out of sight.)
    fn wet_quarter(
        &self,
        layers: &mut Layers,
        layer: Layer,
        quarter: (f32, f32),
        outline: Outline,
        foot: &[Patch],
    ) {
        let (liquid, floor) = surfaces_near(self.scene.level, (self.x, self.y));
        let straight = outline.straight_shore();
        if let Some((east, south)) = straight {
            let mut dry = foot.to_vec();
            dry.push(Patch {
                east: (0.0, east),
                south: (0.0, south),
            });
            self.floor_quarter(layers, layer, quarter, &dry, foot);
        }
        let drawn = liquid::Quarter {
            layer,
            centre: self.centre,
            quarter,
            outline,
            liquid,
            floor,
            with_floor: straight.is_none(),
            colour: &|point| self.colour(layer, point, 1.0),
        };
        drawn.build_in_wall(layers);
        drawn.close_behind_wall(layers, self.scene.level, (self.x, self.y));
    }

    /// Where a face stands in liquid, carries it on down to the bed: straight
    /// down from its foot, whatever curve it has above. The picture starts
    /// again from its top at the floor, so it repeats and isn't stretched.
    fn face_under_liquid(
        &self,
        layers: &mut Layers,
        layer: Layer,
        quarter: (f32, f32),
        (facing, running): (usize, usize),
        reach: [f32; 4],
    ) {
        let Some(outline) = outline_at(self.scene.level, (self.x, self.y), quarter) else {
            return;
        };
        let (plane, from, to) = self.face_at(facing, running, reach, 0.0);
        let along_east = facing.is_multiple_of(2);
        let Some((from, to)) = outline.span_along(along_east, plane, (from, to)) else {
            return;
        };
        let (sx, sz) = quarter;
        let (normal, turned) = if along_east {
            (Vec3::Z * sz, 1.0)
        } else {
            (Vec3::X * sx, SIDEWAYS_SHADE)
        };
        let corner = |along: f32, depth: f32| {
            let (east, south) = if along_east {
                (sx * along, sz * plane)
            } else {
                (sx * plane, sz * along)
            };
            let across = if along_east { east } else { south };
            (
                self.centre + Vec3::new(east, -depth, south),
                [across / CELL_SIZE + 0.5, depth / CELL_SIZE],
                turned * liquid::depth_shade(depth),
            )
        };
        let corners = [
            corner(from, 0.0),
            corner(to, 0.0),
            corner(to, FULL_DEPTH),
            corner(from, FULL_DEPTH),
        ];
        self.add(layers, layer, Surface::Wall, normal, corners);
    }

    /// Where a wall's face stands from the cell's centre, and between which
    /// distances along it, at a height. It faces towards side `facing` and
    /// runs out towards side `running`.
    fn face_at(
        &self,
        facing: usize,
        running: usize,
        reach: [f32; 4],
        height: f32,
    ) -> (f32, f32, f32) {
        let shape = self.scene.shape;
        let half = half_thickness(shape, height);
        let to = reach_at(shape, reach[running], height);
        if self.joins[facing] {
            (half, half, to)
        } else {
            (reach_at(shape, reach[facing], height), 0.0, to)
        }
    }

    /// Draws a face of a wall, if it should be seen, and says on which layer.
    /// `see_through` says its stretch of wall is see-through.
    #[allow(clippy::too_many_arguments)]
    fn wall_face(
        &self,
        layers: &mut Layers,
        quarter: (f32, f32),
        in_sight: bool,
        facing: usize,
        running: usize,
        reach: [f32; 4],
        see_through: bool,
    ) -> Option<Layer> {
        let (sx, sz) = quarter;
        let scene = self.scene;
        let faces_south_or_north = facing.is_multiple_of(2);
        // From local (along, out from the centre, height) to the world.
        let place = |along: f32, out: f32, height: f32| {
            let (east, south) = if faces_south_or_north {
                (sx * along, sz * out)
            } else {
                (sx * out, sz * along)
            };
            self.centre + Vec3::new(east, height, south)
        };
        let normal = if faces_south_or_north {
            Vec3::Z * sz
        } else {
            Vec3::X * sx
        };
        let (plane, from, to) = self.face_at(facing, running, reach, WALL_HEIGHT / 2.0);
        if to <= from {
            return None;
        }
        let middle = place((from + to) / 2.0, plane, WALL_HEIGHT / 2.0);
        let faces = faces_camera(middle, normal, scene.camera);
        let layer = if in_sight {
            if !self.face_seen(quarter, facing) {
                // The quarter shows from round the wall's end, but this face
                // is out of sight: as any other memory of it.
                if !faces {
                    return None;
                }
                Layer::Memory
            } else if see_through {
                Layer::SeeThrough
            } else {
                Layer::Solid
            }
        } else if faces {
            Layer::Memory
        } else {
            return None;
        };
        if layer == Layer::Solid {
            layers.solid_faces.push(WallFace {
                centre: middle,
                normal,
                half_across: (to - from) / 2.0,
                half_up: WALL_HEIGHT / 2.0,
            });
        }

        for corners in self.face_pieces(quarter, facing, running, reach) {
            self.add(layers, layer, Surface::Wall, normal, corners);
        }
        if layer != Layer::Memory {
            self.face_under_liquid(layers, layer, quarter, (facing, running), reach);
        }
        Some(layer)
    }

    /// The pieces a face towards side `facing` is cut into, each four corners
    /// of `(position, picture coordinates, shade)`.
    fn face_pieces(
        &self,
        (sx, sz): (f32, f32),
        facing: usize,
        running: usize,
        reach: [f32; 4],
    ) -> Vec<[(Vec3, [f32; 2], f32); 4]> {
        let shape = self.scene.shape;
        let faces_south_or_north = facing.is_multiple_of(2);
        let place = |along: f32, out: f32, height: f32| {
            let (east, south) = if faces_south_or_north {
                (sx * along, sz * out)
            } else {
                (sx * out, sz * along)
            };
            self.centre + Vec3::new(east, height, south)
        };
        let (_, from, to) = self.face_at(facing, running, reach, WALL_HEIGHT / 2.0);
        let turned = if faces_south_or_north {
            1.0
        } else {
            SIDEWAYS_SHADE
        };
        let segments = match shape {
            Shape::Natural if to - from > CELL_SIZE * 0.2 => NATURAL_SEGMENTS,
            _ => 1,
        };
        let cuts = heights(shape, 0.0, WALL_HEIGHT);
        let mut pieces = Vec::new();
        // Only points inside the face wander, so faces still meet each other.
        let corner = |band: usize, segment: usize| {
            let height = cuts[band];
            let (plane, from, to) = self.face_at(facing, running, reach, height);
            let along = from + (to - from) * segment as f32 / segments as f32;
            let inside = shape == Shape::Natural
                && band > 0
                && band + 1 < cuts.len()
                && segment > 0
                && segment < segments;
            let bump = if inside {
                let seed = 4 * self.x + 7 * self.y + facing as i64 * 11 + (sx + 2.0 * sz) as i64;
                noise(seed, band as i64, segment as i64)
            } else {
                0.0
            };
            let point = place(along, plane + bump * ROUGHNESS, height);
            let across = if faces_south_or_north {
                point.x - self.centre.x
            } else {
                point.z - self.centre.z
            };
            let shade = height_shade(shape, height) * turned * (1.0 + bump * ROUGH_SHADE);
            (point, face_uv(across, height), shade)
        };
        for band in 0..cuts.len() - 1 {
            for segment in 0..segments {
                pieces.push([
                    corner(band, segment),
                    corner(band, segment + 1),
                    corner(band + 1, segment + 1),
                    corner(band + 1, segment),
                ]);
            }
        }
        pieces
    }

    /// The top of a wall in a quarter: the part running one way, then what
    /// the other way adds to it.
    fn wall_top(
        &self,
        layers: &mut Layers,
        layer: Layer,
        (sx, sz): (f32, f32),
        (east_west, north_south): (usize, usize),
        reach: [f32; 4],
    ) {
        let (z_plane, _, x_end) = self.face_at(north_south, east_west, reach, WALL_HEIGHT);
        let (x_plane, _, z_end) = self.face_at(east_west, north_south, reach, WALL_HEIGHT);
        let point = |east: f32, south: f32| {
            let (e, s) = (sx * east, sz * south);
            let position = self.centre + Vec3::new(e, WALL_HEIGHT, s);
            (position, flat_uv(e, s), TOP_SHADE)
        };
        let rect = |x0: f32, x1: f32, z0: f32, z1: f32| {
            [point(x0, z0), point(x1, z0), point(x1, z1), point(x0, z1)]
        };
        self.add(
            layers,
            layer,
            Surface::Wall,
            Vec3::Y,
            rect(0.0, x_end, 0.0, z_plane),
        );
        if z_end > z_plane {
            self.add(
                layers,
                layer,
                Surface::Wall,
                Vec3::Y,
                rect(0.0, x_plane, z_plane, z_end),
            );
        }
    }

    /// The wall of a door or window: its two sides, each drawn on the layer
    /// it should be seen on, and the frame round the hole. A natural window
    /// is a line of columns instead.
    fn build_opening_wall(
        &self,
        layers: &mut Layers,
        east_west: bool,
        window: bool,
        in_sight: bool,
    ) {
        let scene = self.scene;
        let (along, out) = if east_west {
            (Vec3::X, Vec3::Z)
        } else {
            (Vec3::Z, Vec3::X)
        };
        let side_middle = |side: f32| self.centre + out * side * WALL_HALF;
        let in_front = |view: &View, side: f32| {
            let viewer = Vec3::new(
                view.cell.0 as f32 * CELL_SIZE,
                0.0,
                view.cell.1 as f32 * CELL_SIZE,
            );
            (viewer - side_middle(side)).dot(out * side) > 0.0
        };
        let seeing = |view: &&View| view.sight.is_seen(self.x, self.y);
        // Someone standing in line with the wall sees both sides.
        let in_line = !scene
            .views
            .iter()
            .filter(seeing)
            .any(|view| in_front(view, 1.0) || in_front(view, -1.0));
        let faces = |side: f32| {
            let middle = side_middle(side) + Vec3::Y * (WALL_HEIGHT / 2.0);
            faces_camera(middle, out * side, scene.camera)
        };
        let seen =
            |side: f32| in_line || scene.views.iter().filter(seeing).any(|v| in_front(v, side));
        // As with walls, both sides look the same: see-through if either is
        // seen turned away from the camera.
        let see_through = [1.0, -1.0].iter().any(|&side| seen(side) && !faces(side));
        let side_layers = [1.0, -1.0].map(|side| {
            if in_sight {
                match (seen(side), see_through) {
                    (false, _) => None,
                    (true, false) => Some(Layer::Solid),
                    (true, true) => Some(Layer::SeeThrough),
                }
            } else {
                let faces = faces(side);
                faces.then_some(Layer::Memory)
            }
        });
        let Some(frame_layer) = [Layer::Solid, Layer::SeeThrough, Layer::Memory]
            .into_iter()
            .find(|&layer| side_layers.contains(&Some(layer)))
        else {
            return;
        };
        let opening = Opening {
            along,
            out,
            window,
            shape: scene.shape,
        };
        if scene.shape == Shape::Natural && window {
            self.build_pillars(layers, frame_layer, &opening);
            return;
        }
        for (side, layer) in [1.0, -1.0].into_iter().zip(side_layers) {
            if let Some(layer) = layer {
                self.opening_side(layers, layer, &opening, side);
            }
        }
        self.opening_frame(layers, frame_layer, &opening);
    }

    /// One side of a wall with a hole in it, cut into columns from one edge
    /// of the cell to the other.
    fn opening_side(&self, layers: &mut Layers, layer: Layer, opening: &Opening, side: f32) {
        let columns = opening.columns();
        let turned = if opening.along == Vec3::X {
            1.0
        } else {
            SIDEWAYS_SHADE
        };
        let shape = opening.shape;
        let normal = opening.out * side;
        for pair in columns.windows(2) {
            let (a0, a1) = (pair[0], pair[1]);
            // Whether this column is across the hole, not beside it.
            let hole = a0 >= -OPENING_HALF && a1 <= OPENING_HALF;
            let (top0, top1) = if hole {
                (opening.top(a0), opening.top(a1))
            } else {
                (0.0, 0.0)
            };
            let parts: &[(f32, f32, f32, f32)] = &[
                (top0, top1, WALL_HEIGHT, WALL_HEIGHT),
                (0.0, 0.0, opening.bottom(), opening.bottom()),
            ];
            let count = if hole && opening.window { 2 } else { 1 };
            for &(low0, low1, high0, high1) in &parts[..count] {
                let bands = heights(shape, 0.0, 1.0);
                let corner = |a: f32, low: f32, high: f32, t: f32| {
                    let height = low + (high - low) * t;
                    let point = self.centre
                        + opening.along * a
                        + Vec3::Y * height
                        + normal * half_thickness(shape, height);
                    (
                        point,
                        face_uv(a, height),
                        height_shade(shape, height) * turned,
                    )
                };
                for band in bands.windows(2) {
                    let corners = [
                        corner(a0, low0, high0, band[0]),
                        corner(a1, low1, high1, band[0]),
                        corner(a1, low1, high1, band[1]),
                        corner(a0, low0, high0, band[1]),
                    ];
                    self.add(layers, layer, Surface::Wall, normal, corners);
                }
                if layer == Layer::Solid {
                    let (low, high) = ((low0 + low1) / 2.0, (high0 + high1) / 2.0);
                    layers.solid_faces.push(WallFace {
                        centre: self.centre
                            + opening.along * ((a0 + a1) / 2.0)
                            + Vec3::Y * ((low + high) / 2.0)
                            + normal * WALL_HALF,
                        normal,
                        half_across: (a1 - a0) / 2.0,
                        half_up: (high - low) / 2.0,
                    });
                }
            }
        }
    }

    /// The insides of the hole (the sides, its top and the window's sill) and
    /// the top of the wall.
    fn opening_frame(&self, layers: &mut Layers, layer: Layer, opening: &Opening) {
        let shape = opening.shape;
        let surface = match shape {
            Shape::Built => Surface::Door,
            Shape::Natural => Surface::Wall,
        };
        let centre = self.centre;
        let across = |a: f32, height: f32, out: f32| {
            centre + opening.along * a + Vec3::Y * height + opening.out * out
        };
        // Skipped when turned away from the camera, except in the see-through
        // layer, where the back shows too.
        let wanted = |middle: Vec3, normal: Vec3| {
            layer == Layer::SeeThrough || faces_camera(middle, normal, self.scene.camera)
        };
        // The sides of the hole.
        for sign in [1.0f32, -1.0] {
            let a = sign * OPENING_HALF;
            let normal = opening.along * -sign;
            let bottom = opening.bottom();
            let cuts = heights(shape, bottom, opening.top(a));
            for band in cuts.windows(2) {
                let corner = |height: f32, out: f32| {
                    let half = half_thickness(shape, height) * out;
                    (
                        across(a, height, half),
                        face_uv(half, height),
                        FRAME_SHADE * height_shade(shape, height),
                    )
                };
                let corners = [
                    corner(band[0], -1.0),
                    corner(band[0], 1.0),
                    corner(band[1], 1.0),
                    corner(band[1], -1.0),
                ];
                if wanted(corners[0].0, normal) {
                    self.add(layers, layer, surface, normal, corners);
                }
            }
        }
        // Over the hole: flat, or curved for an arch.
        let columns: Vec<f32> = opening
            .columns()
            .into_iter()
            .filter(|a| a.abs() <= OPENING_HALF)
            .collect();
        for pair in columns.windows(2) {
            let corner = |a: f32, out: f32| {
                let height = opening.top(a);
                let half = half_thickness(shape, height) * out;
                (
                    across(a, height, half),
                    flat_uv(a, half),
                    FRAME_SHADE * RISING_SHADE,
                )
            };
            let corners = [
                corner(pair[0], -1.0),
                corner(pair[1], -1.0),
                corner(pair[1], 1.0),
                corner(pair[0], 1.0),
            ];
            if wanted(corners[0].0, Vec3::NEG_Y) {
                self.add(layers, layer, surface, Vec3::NEG_Y, corners);
            }
        }
        if opening.window {
            let height = opening.bottom();
            let half = half_thickness(shape, height);
            let corner = |a: f32, out: f32| (across(a, height, out), flat_uv(a, out), FRAME_SHADE);
            let corners = [
                corner(-OPENING_HALF, -half),
                corner(OPENING_HALF, -half),
                corner(OPENING_HALF, half),
                corner(-OPENING_HALF, half),
            ];
            self.add(layers, layer, surface, Vec3::Y, corners);
        }
        let half = half_thickness(shape, WALL_HEIGHT);
        let corner = |a: f32, out: f32| (across(a, WALL_HEIGHT, out), flat_uv(a, out), TOP_SHADE);
        let corners = [
            corner(-HALF_CELL, -half),
            corner(HALF_CELL, -half),
            corner(HALF_CELL, half),
            corner(-HALF_CELL, half),
        ];
        self.add(layers, layer, Surface::Wall, Vec3::Y, corners);
    }

    /// A natural window: a line of columns, or of stalagmites below
    /// stalactites, as chosen by the cell's position.
    fn build_pillars(&self, layers: &mut Layers, layer: Layer, opening: &Opening) {
        for i in 0..PILLARS {
            let a = (i - PILLARS / 2) as f32 * PILLAR_SPACING;
            let foot = self.centre + opening.along * a;
            let kind = noise(self.x, self.y, i);
            if kind < 0.2 {
                // A whole column, narrowest half way up.
                let width = |height: f32| {
                    let from_middle = 2.0 * height / WALL_HEIGHT - 1.0;
                    PILLAR_HALF + (PILLAR_FOOT_HALF - PILLAR_HALF) * from_middle * from_middle
                };
                self.pillar(layers, layer, opening, foot, (0.0, WALL_HEIGHT), &width);
            } else {
                // Each comes to a point, with a gap between them.
                let gap = noise(self.y, self.x, i) * 0.12 + 0.5;
                let low = WALL_HEIGHT * (gap - 0.1);
                let high = WALL_HEIGHT * (gap + 0.1);
                let rising = |height: f32| PILLAR_FOOT_HALF * (1.0 - height / low).max(0.0);
                let hanging = |height: f32| {
                    PILLAR_FOOT_HALF * ((height - high) / (WALL_HEIGHT - high)).max(0.0)
                };
                self.pillar(layers, layer, opening, foot, (0.0, low), &rising);
                self.pillar(layers, layer, opening, foot, (high, WALL_HEIGHT), &hanging);
            }
        }
    }

    /// One four-sided column between two heights, `width` being half its
    /// width at a height.
    fn pillar(
        &self,
        layers: &mut Layers,
        layer: Layer,
        opening: &Opening,
        foot: Vec3,
        (low, high): (f32, f32),
        width: &dyn Fn(f32) -> f32,
    ) {
        let cuts = heights(Shape::Natural, low, high);
        let (along, out) = (opening.along, opening.out);
        for (normal, right, shade) in [
            (out, along, 1.0),
            (-out, along, 1.0),
            (along, out, SIDEWAYS_SHADE),
            (-along, out, SIDEWAYS_SHADE),
        ] {
            let middle = foot + normal * PILLAR_HALF + Vec3::Y * ((low + high) / 2.0);
            if layer != Layer::SeeThrough && !faces_camera(middle, normal, self.scene.camera) {
                continue;
            }
            for band in cuts.windows(2) {
                let corner = |height: f32, sign: f32| {
                    let w = width(height);
                    let across = (foot - self.centre).dot(right) + sign * w;
                    (
                        foot + normal * w + right * (sign * w) + Vec3::Y * height,
                        face_uv(across, height),
                        shade * height_shade(Shape::Natural, height),
                    )
                };
                let corners = [
                    corner(band[0], -1.0),
                    corner(band[0], 1.0),
                    corner(band[1], 1.0),
                    corner(band[1], -1.0),
                ];
                self.add(layers, layer, Surface::Wall, normal, corners);
            }
        }
    }
}

/// How the underside of a lintel or arch is shaded.
const RISING_SHADE: f32 = 0.7;

/// The hole in the wall of a door or window.
struct Opening {
    /// The way the wall runs, and the way out of it.
    along: Vec3,
    out: Vec3,
    window: bool,
    shape: Shape,
}

impl Opening {
    /// Where the hole starts above the floor.
    fn bottom(&self) -> f32 {
        if self.window { WINDOW_SILL } else { 0.0 }
    }

    /// The top of the hole, at a distance from the middle of the cell: flat,
    /// or curved for a natural door.
    fn top(&self, a: f32) -> f32 {
        match (self.shape, self.window) {
            (Shape::Natural, false) => {
                let across = (a / OPENING_HALF).clamp(-1.0, 1.0);
                ARCH_START + (DOOR_TOP - ARCH_START) * (1.0 - across * across).sqrt()
            }
            (_, false) => DOOR_TOP,
            (_, true) => WINDOW_TOP,
        }
    }

    /// Where the wall is cut along its length: its ends, the sides of the
    /// hole, and points round an arch.
    fn columns(&self) -> Vec<f32> {
        let mut columns = vec![-HALF_CELL];
        if self.shape == Shape::Natural && !self.window {
            const STEPS: usize = 8;
            for i in 0..=STEPS {
                let angle = std::f32::consts::PI * (1.0 - i as f32 / STEPS as f32);
                columns.push(OPENING_HALF * angle.cos());
            }
        } else {
            columns.extend([-OPENING_HALF, OPENING_HALF]);
        }
        columns.push(HALF_CELL);
        columns
    }
}
