//! Draws what only towns have, all from one picture (`assets/town_atlas.png`,
//! made by `tools/make_town_atlas.py`): roads, gardens with hedges, and solid
//! buildings with roofs.
//!
//! A building's walls stand down the middle of its outermost cells, as in
//! `walls`, but it is drawn as a whole: walls of different buildings don't
//! join, so a strip of ground shows between them. Its roof slopes up from the
//! walls across those cells, fading to black, and is flat and black inside.
//! A building in the way of a viewer goes half see-through.

use std::collections::BTreeSet;

use bevy::prelude::*;
use shared::dungeon::{CELL_UNITS, Cell, RoomId, WALL_HALF_UNITS};
use shared::site::{BuildingKind, BuildingLook, RoofKind, WallKind};

use super::mesh::{MEMORY_SHADE, Surface, WallFace, faces_camera};
use super::walls::{Layer, Layers, Scene};
use super::{CELL_SIZE, WALL_HEIGHT};

/// Tiles across and down the atlas.
const COLUMNS: f32 = 8.0;
const ROWS: f32 = 6.0;
/// Pixels across a tile of the atlas.
const ATLAS_TILE_PIXELS: u32 = 32;

/// Where things are in the atlas: the order `make_town_atlas.py` puts them.
const GARDEN_TILE: usize = 6;
const HEDGE_TILE: usize = 7;
const FIRST_WALL_TILE: usize = 8;
const FIRST_ROOF_TILE: usize = 13;
const WINDOW_TILE: usize = 18;
const FIRST_DOOR_TILE: usize = 20;

const HALF_CELL: f32 = CELL_SIZE / 2.0;

/// How far a wall stands from the centre of its cell.
const WALL_OUT: f32 = WALL_HALF_UNITS as f32 / CELL_UNITS as f32 * CELL_SIZE;

/// How high a roof rises above the walls, over how far in from them: the rest
/// of the outermost cells, so what is inside those is flat.
const ROOF_RISE: f32 = CELL_SIZE * 0.5;
const ROOF_RAMP: f32 = HALF_CELL + WALL_OUT;

/// Shading, since it is all unlit: faces running north-south are a little
/// darker, which shows corners.
const SIDEWAYS_SHADE: f32 = 0.85;

/// How far doors and windows stand in front of their wall, to stay in front.
const OVERLAY_LIFT: f32 = 0.05;

/// A window's half width, and its bottom and top.
const WINDOW_HALF: f32 = CELL_SIZE * 0.25;
const WINDOW_SILL: f32 = WALL_HEIGHT * 0.4;
const WINDOW_TOP: f32 = WALL_HEIGHT * 0.8;

/// A hedge's distance in from the garden's edge, its thickness and height.
const HEDGE_INSET: f32 = CELL_SIZE * 0.2;
const HEDGE_THICK: f32 = CELL_SIZE * 0.2;
const HEDGE_HEIGHT: f32 = CELL_SIZE * 0.2;

/// The hedge's sides are darker than its top.
const HEDGE_SIDE_SHADE: f32 = 0.7;

/// An arch's half width (to the inside of its posts) and height.
const ARCH_HALF: f32 = CELL_SIZE * 0.25;
const ARCH_HEIGHT: f32 = CELL_SIZE * 1.5;

/// How likely a stretch of hedge at least `ARCH_RUN` long has an arch, in
/// percent.
const ARCH_PERCENT: u64 = 50;
const ARCH_RUN: i64 = 3;

/// Picture coordinates in a tile of the atlas, from coordinates in that tile.
fn atlas_uv(tile: usize, u: f32, v: f32) -> [f32; 2] {
    let (column, row) = (tile as f32 % COLUMNS, (tile as f32 / COLUMNS).floor());
    [
        (column + u.clamp(0.0, 1.0)) / COLUMNS,
        (row + v.clamp(0.0, 1.0)) / ROWS,
    ]
}

/// A tile's coordinates for a point `(east, south)` from a cell's centre.
fn flat_uv(tile: usize, east: f32, south: f32) -> [f32; 2] {
    atlas_uv(tile, east / CELL_SIZE + 0.5, south / CELL_SIZE + 0.5)
}

fn road_tile(cell: Cell) -> Option<usize> {
    match cell {
        Cell::DirtTrack => Some(0),
        Cell::GravelTrack => Some(1),
        Cell::WoodenBoards => Some(2),
        Cell::Cobblestone => Some(3),
        Cell::Flagstones => Some(4),
        Cell::DressedStone => Some(5),
        Cell::Garden => Some(GARDEN_TILE),
        _ => None,
    }
}

fn wall_tile(kind: WallKind) -> usize {
    FIRST_WALL_TILE + WallKind::ALL.iter().position(|&k| k == kind).unwrap_or(0)
}

fn roof_tile(kind: RoofKind) -> usize {
    FIRST_ROOF_TILE + RoofKind::ALL.iter().position(|&k| k == kind).unwrap_or(0)
}

fn cell_centre(x: i64, y: i64) -> Vec3 {
    Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE)
}

/// Adds a four-cornered surface to the town's mesh in a layer. See-through
/// faces always turn towards the camera: the material culls back faces.
fn emit(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    positions: [Vec3; 4],
    normal: Vec3,
    uvs: [[f32; 2]; 4],
    colours: [Vec3; 4],
) {
    let town = Surface::Town as usize;
    match layer {
        Layer::Solid => layers.solid[town].free(positions, normal, uvs, colours),
        Layer::Memory => layers.memory[town].free(positions, normal, uvs, colours),
        Layer::SeeThrough => {
            let centre = (positions[0] + positions[2]) / 2.0;
            let front = if faces_camera(centre, normal, scene.camera) {
                normal
            } else {
                -normal
            };
            layers.stairs_through[town].free(positions, front, uvs, colours);
        }
    }
}

/// The flat road or garden at `(x, y)`.
pub fn build_flat(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let Some(tile) = road_tile(scene.level.cell(x, y)) else {
        return;
    };
    let layer = flat_layer(scene, x, y);
    let patch = Rect::from_corners(Vec2::splat(-HALF_CELL), Vec2::splat(HALF_CELL));
    flat_patch(layers, scene, layer, cell_centre(x, y), patch, tile);
}

fn flat_layer(scene: &Scene, x: i64, y: i64) -> Layer {
    if scene.sight.is_seen(x, y) {
        Layer::Solid
    } else {
        Layer::Memory
    }
}

fn layer_colour(layer: Layer) -> Vec3 {
    match layer {
        Layer::Memory => Vec3::splat(MEMORY_SHADE),
        _ => Vec3::ONE,
    }
}

/// A flat piece of a tile, as distances from `centre` (`x` east, `y` south).
fn flat_patch(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    centre: Vec3,
    patch: Rect,
    tile: usize,
) {
    let corners = [
        (patch.min.x, patch.min.y),
        (patch.max.x, patch.min.y),
        (patch.max.x, patch.max.y),
        (patch.min.x, patch.max.y),
    ];
    emit(
        layers,
        scene,
        layer,
        corners.map(|(east, south)| centre + Vec3::new(east, 0.0, south)),
        Vec3::Y,
        corners.map(|(east, south)| flat_uv(tile, east, south)),
        [layer_colour(layer); 4],
    );
}

/// Every building of the town.
pub fn build_buildings(layers: &mut Layers, scene: &Scene) {
    let hidden = in_the_way(scene);
    for building in scene.buildings {
        let id = RoomId(building.id);
        let seen = building_seen(scene, building, id);
        let hides = hidden.contains(&building.id);
        for rect in &building.rects {
            for y in rect.y..rect.y + rect.height {
                for x in rect.x..rect.x + rect.width {
                    build_cell(layers, scene, building, (x, y), id, seen, hides);
                }
            }
        }
    }
}

/// The buildings on the camera's line to a viewer, at any height a building
/// reaches.
fn in_the_way(scene: &Scene) -> BTreeSet<u16> {
    let mut found = BTreeSet::new();
    let top = WALL_HEIGHT + ROOF_RISE;
    for view in scene.views {
        let target = cell_centre(view.cell.0, view.cell.1);
        let camera = scene.camera;
        if camera.y <= top {
            continue;
        }
        // From where the line comes down through the roofs to the ground.
        let from = target + (camera - target) * (top / camera.y);
        let steps = ((from - target).length() / (CELL_SIZE / 4.0))
            .ceil()
            .max(1.0) as i32;
        for step in 0..=steps {
            let at = target.lerp(from, step as f32 / steps as f32);
            let cell = (
                (at.x / CELL_SIZE).round() as i64,
                (at.z / CELL_SIZE).round() as i64,
            );
            if let Some(room) = scene.level.room(cell.0, cell.1) {
                found.insert(room.0);
            }
        }
    }
    found
}

/// Whether anyone can see any part of a building.
fn building_seen(scene: &Scene, building: &BuildingLook, id: RoomId) -> bool {
    let level = scene.level;
    building.rects.iter().any(|rect| {
        (rect.y..rect.y + rect.height).any(|y| {
            (rect.x..rect.x + rect.width).any(|x| {
                let edge = [(0, -1), (1, 0), (0, 1), (-1, 0)]
                    .iter()
                    .any(|(dx, dy)| level.room(x + dx, y + dy) != Some(id));
                edge && scene.sight.is_seen(x, y)
            })
        })
    })
}

/// A face of a wall (or a door or window on it), upright, running along `x` or
/// `z`, facing `normal`, from `(a, ..)` to `(b, ..)` along it.
struct Piece {
    /// A point on the bottom of the face at one end, and at the other.
    from: Vec3,
    to: Vec3,
    normal: Vec3,
    /// Picture coordinates across, at each end.
    across: (f32, f32),
}

/// Which layer a face goes in, or none: seen and facing the camera it is
/// solid; seen and facing away, see-through; unseen and facing it, memory.
fn face_layer(
    scene: &Scene,
    own: (i64, i64),
    outer: (i64, i64),
    piece: &Piece,
    hides: bool,
) -> Option<Layer> {
    let level = scene.level;
    let middle = (piece.from + piece.to) / 2.0;
    let in_front = scene.views.iter().any(|view| {
        let viewer = cell_centre(view.cell.0, view.cell.1);
        piece.normal.dot(viewer - middle) >= 0.0
            && if level.is_open(outer.0, outer.1) {
                view.sight.is_seen(outer.0, outer.1)
            } else {
                view.sight.is_seen(own.0, own.1)
            }
    });
    let facing = faces_camera(middle, piece.normal, scene.camera);
    match (in_front, facing) {
        _ if hides && (in_front || facing) => Some(Layer::SeeThrough),
        (true, true) => Some(Layer::Solid),
        (true, false) => Some(Layer::SeeThrough),
        (false, true) => Some(Layer::Memory),
        (false, false) => None,
    }
}

fn side_shade(normal: Vec3) -> f32 {
    if normal.x != 0.0 { SIDEWAYS_SHADE } else { 1.0 }
}

/// A face from the ground up to `height`, in `tile`.
fn upright(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    piece: &Piece,
    (low, high): (f32, f32),
    tile: usize,
) {
    let colour = layer_colour(layer) * side_shade(piece.normal);
    let (v_low, v_high) = (1.0 - low / WALL_HEIGHT, 1.0 - high / WALL_HEIGHT);
    emit(
        layers,
        scene,
        layer,
        [
            piece.from + Vec3::Y * low,
            piece.to + Vec3::Y * low,
            piece.to + Vec3::Y * high,
            piece.from + Vec3::Y * high,
        ],
        piece.normal,
        [
            atlas_uv(tile, piece.across.0, v_low),
            atlas_uv(tile, piece.across.1, v_low),
            atlas_uv(tile, piece.across.1, v_high),
            atlas_uv(tile, piece.across.0, v_high),
        ],
        [colour; 4],
    );
}

/// A wall's face: goes into whichever layer it belongs in.
fn wall_face(
    layers: &mut Layers,
    scene: &Scene,
    (own, outer): ((i64, i64), (i64, i64)),
    piece: Piece,
    tile: usize,
    hides: bool,
) {
    let Some(layer) = face_layer(scene, own, outer, &piece, hides) else {
        return;
    };
    upright(layers, scene, layer, &piece, (0.0, WALL_HEIGHT), tile);
    if layer == Layer::Solid {
        let middle = (piece.from + piece.to) / 2.0;
        layers.solid_faces.push(WallFace {
            centre: middle + Vec3::Y * WALL_HEIGHT / 2.0,
            normal: piece.normal,
            half_across: (piece.to - piece.from).length() / 2.0,
            half_up: WALL_HEIGHT / 2.0,
        });
    }
}

/// The ground under a wall's outer half is always the land's own, whatever
/// the cell beyond it is: it shows as seen if that cell is.
fn strip(layers: &mut Layers, scene: &Scene, centre: Vec3, patch: Rect, neighbours: &[(i64, i64)]) {
    let seen = neighbours.iter().any(|&(x, y)| scene.sight.is_seen(x, y));
    let layer = if seen { Layer::Solid } else { Layer::Memory };
    ground_patch(layers, layer, centre, patch);
}

/// A piece of the land's own ground.
fn ground_patch(layers: &mut Layers, layer: Layer, centre: Vec3, patch: Rect) {
    let corners = [
        (patch.min.x, patch.min.y),
        (patch.max.x, patch.min.y),
        (patch.max.x, patch.max.y),
        (patch.min.x, patch.max.y),
    ];
    let builder = layers.builder(layer, Surface::Ground);
    builder.free(
        corners.map(|(east, south)| centre + Vec3::new(east, 0.0, south)),
        Vec3::Y,
        corners.map(|(east, south)| [east / CELL_SIZE + 0.5, south / CELL_SIZE + 0.5]),
        [layer_colour(layer); 4],
    );
}

/// One cell of a building, in quarters: the part of a quarter inside the wall
/// has roof; beyond the wall it has ground, unless the building carries on.
fn build_cell(
    layers: &mut Layers,
    scene: &Scene,
    building: &BuildingLook,
    (x, y): (i64, i64),
    id: RoomId,
    seen: bool,
    hides: bool,
) {
    let level = scene.level;
    let centre = cell_centre(x, y);
    let inside = |dx: i64, dy: i64| level.room(x + dx, y + dy) == Some(id);
    let neighbours: [[bool; 3]; 3] = std::array::from_fn(|row| {
        std::array::from_fn(|column| inside(column as i64 - 1, row as i64 - 1))
    });
    let roof_layer = if hides {
        Layer::SeeThrough
    } else if seen {
        Layer::Solid
    } else {
        Layer::Memory
    };
    let tile = roof_tile(building.roof);
    if neighbours.iter().flatten().all(|&inside| inside) {
        roof_centre(layers, scene, roof_layer, centre);
        return;
    }
    let wall = wall_tile(building.walls);
    let roof = Roof {
        centre,
        neighbours,
        layer: roof_layer,
        tile,
    };
    for (sx, sz) in [(1i64, 1i64), (1, -1), (-1, 1), (-1, -1)] {
        let (fx, fz) = (sx as f32, sz as f32);
        let east = inside(sx, 0);
        let south = inside(0, sz);
        let corner = inside(sx, sz);
        let span = |low: f32, high: f32, sign: f32| {
            let (a, b) = (low * sign, high * sign);
            (a.min(b), a.max(b))
        };
        let (x_in, x_out) = (span(0.0, WALL_OUT, fx), span(WALL_OUT, HALF_CELL, fx));
        let (z_in, z_out) = (span(0.0, WALL_OUT, fz), span(WALL_OUT, HALF_CELL, fz));
        let patch = |(x0, x1): (f32, f32), (z0, z1): (f32, f32)| {
            Rect::from_corners(Vec2::new(x0, z0), Vec2::new(x1, z1))
        };
        let beside_x = (x + sx, y);
        let beside_z = (x, y + sz);
        let diagonal = (x + sx, y + sz);
        roof.patch(layers, scene, patch(x_in, z_in));
        if east {
            roof.patch(layers, scene, patch(x_out, z_in));
        } else {
            strip(layers, scene, centre, patch(x_out, z_in), &[beside_x]);
        }
        if south {
            roof.patch(layers, scene, patch(x_in, z_out));
        } else {
            strip(layers, scene, centre, patch(x_in, z_out), &[beside_z]);
        }
        if east && south && corner {
            roof.patch(layers, scene, patch(x_out, z_out));
        } else {
            let open = [beside_x, beside_z, diagonal];
            strip(layers, scene, centre, patch(x_out, z_out), &open);
        }
        // The walls round what has roof.
        let point = |a: f32, b: f32| centre + Vec3::new(a * fx, 0.0, b * fz);
        let across_z = |a: f32, b: f32| (a * fz / CELL_SIZE + 0.5, b * fz / CELL_SIZE + 0.5);
        let across_x = |a: f32, b: f32| (a * fx / CELL_SIZE + 0.5, b * fx / CELL_SIZE + 0.5);
        let x_normal = Vec3::X * fx;
        let z_normal = Vec3::Z * fz;
        if !east {
            let piece = Piece {
                from: point(WALL_OUT, 0.0),
                to: point(WALL_OUT, WALL_OUT),
                normal: x_normal,
                across: across_z(0.0, WALL_OUT),
            };
            wall_face(layers, scene, ((x, y), beside_x), piece, wall, hides);
        }
        if !south {
            let piece = Piece {
                from: point(0.0, WALL_OUT),
                to: point(WALL_OUT, WALL_OUT),
                normal: z_normal,
                across: across_x(0.0, WALL_OUT),
            };
            wall_face(layers, scene, ((x, y), beside_z), piece, wall, hides);
        }
        if east && !(south && corner) {
            let outer = if south { diagonal } else { beside_z };
            let piece = Piece {
                from: point(WALL_OUT, WALL_OUT),
                to: point(HALF_CELL, WALL_OUT),
                normal: z_normal,
                across: across_x(WALL_OUT, HALF_CELL),
            };
            wall_face(layers, scene, ((x, y), outer), piece, wall, hides);
        }
        if south && !(east && corner) {
            let outer = if east { diagonal } else { beside_x };
            let piece = Piece {
                from: point(WALL_OUT, WALL_OUT),
                to: point(WALL_OUT, HALF_CELL),
                normal: x_normal,
                across: across_z(WALL_OUT, HALF_CELL),
            };
            wall_face(layers, scene, ((x, y), outer), piece, wall, hides);
        }
    }
    build_door_or_window(layers, scene, building, (x, y), hides);
}

/// The flat black middle of a roof.
fn roof_centre(layers: &mut Layers, scene: &Scene, layer: Layer, centre: Vec3) {
    let height = Vec3::Y * (WALL_HEIGHT + ROOF_RISE);
    let corners = [
        Vec3::new(-HALF_CELL, 0.0, -HALF_CELL),
        Vec3::new(HALF_CELL, 0.0, -HALF_CELL),
        Vec3::new(HALF_CELL, 0.0, HALF_CELL),
        Vec3::new(-HALF_CELL, 0.0, HALF_CELL),
    ];
    emit(
        layers,
        scene,
        layer,
        corners.map(|corner| centre + corner + height),
        Vec3::Y,
        // Any part of any tile: it is black.
        [atlas_uv(FIRST_ROOF_TILE, 0.5, 0.5); 4],
        [Vec3::ZERO; 4],
    );
}

/// One outermost cell's roof.
struct Roof {
    centre: Vec3,
    /// Which cells round it are the same building, by row and column.
    neighbours: [[bool; 3]; 3],
    layer: Layer,
    tile: usize,
}

impl Roof {
    /// How far in from the walls a point is (from the cell's centre), up to
    /// `ROOF_RAMP`, measured square: it makes corners like a hipped roof.
    fn distance_in(&self, east: f32, south: f32) -> f32 {
        let mut nearest = ROOF_RAMP;
        // The space outside a wall reaches this far from its own cell's
        // centre, towards this cell.
        let reach = CELL_SIZE - WALL_OUT;
        for (row, cells) in self.neighbours.iter().enumerate() {
            for (column, &inside) in cells.iter().enumerate() {
                if inside {
                    continue;
                }
                let dx = (column as f32 - 1.0) * CELL_SIZE - east;
                let dz = (row as f32 - 1.0) * CELL_SIZE - south;
                nearest = nearest.min(dx.abs().max(dz.abs()) - reach);
            }
        }
        nearest.max(0.0)
    }

    fn patch(&self, layers: &mut Layers, scene: &Scene, patch: Rect) {
        let corners = [
            (patch.min.x, patch.min.y),
            (patch.max.x, patch.min.y),
            (patch.max.x, patch.max.y),
            (patch.min.x, patch.max.y),
        ];
        let along = corners.map(|(east, south)| self.distance_in(east, south) / ROOF_RAMP);
        let shade = layer_colour(self.layer);
        let positions = std::array::from_fn(|i| {
            let (east, south) = corners[i];
            self.centre + Vec3::new(east, WALL_HEIGHT + ROOF_RISE * along[i], south)
        });
        emit(
            layers,
            scene,
            self.layer,
            positions,
            Vec3::Y,
            corners.map(|(east, south)| flat_uv(self.tile, east, south)),
            along.map(|up| shade * (1.0 - up)),
        );
    }
}

/// The door or window on a wall cell, if it has one.
fn build_door_or_window(
    layers: &mut Layers,
    scene: &Scene,
    building: &BuildingLook,
    (x, y): (i64, i64),
    hides: bool,
) {
    let level = scene.level;
    let cell = level.cell(x, y);
    let (faces, door) = match cell {
        Cell::BuildingDoor if building.door == (x, y) => (building.faces, true),
        Cell::BlindWindow => {
            let open = [(0, -1), (1, 0), (0, 1), (-1, 0)]
                .into_iter()
                .find(|&(dx, dy)| level.is_walkable(x + dx, y + dy));
            let Some(faces) = open else {
                return;
            };
            (faces, false)
        }
        _ => return,
    };
    let normal = Vec3::new(faces.0 as f32, 0.0, faces.1 as f32);
    let along = Vec3::new(faces.1 as f32, 0.0, -faces.0 as f32);
    // Pictures read left to right seen from the front.
    let middle = cell_centre(x, y) + normal * (WALL_OUT + OVERLAY_LIFT);
    let half = if door { HALF_CELL } else { WINDOW_HALF };
    let piece = Piece {
        from: middle - along * half,
        to: middle + along * half,
        normal,
        across: (0.0, 1.0),
    };
    let outer = (x + faces.0, y + faces.1);
    let Some(layer) = face_layer(scene, (x, y), outer, &piece, hides) else {
        return;
    };
    if door {
        let tile = FIRST_DOOR_TILE + kind_index(building);
        upright(layers, scene, layer, &piece, (0.0, WALL_HEIGHT), tile);
    } else {
        let colour = layer_colour(layer) * side_shade(normal);
        emit(
            layers,
            scene,
            layer,
            [
                piece.from + Vec3::Y * WINDOW_SILL,
                piece.to + Vec3::Y * WINDOW_SILL,
                piece.to + Vec3::Y * WINDOW_TOP,
                piece.from + Vec3::Y * WINDOW_TOP,
            ],
            normal,
            [
                atlas_uv(WINDOW_TILE, 0.0, 1.0),
                atlas_uv(WINDOW_TILE, 1.0, 1.0),
                atlas_uv(WINDOW_TILE, 1.0, 0.0),
                atlas_uv(WINDOW_TILE, 0.0, 0.0),
            ],
            [colour; 4],
        );
    }
}

/// Where a kind of building's door picture is in the atlas: left, top and
/// size, in pixels (the maps show it as a shop's sign).
pub fn door_picture(kind: BuildingKind) -> (u32, u32, u32) {
    let tile = (FIRST_DOOR_TILE + kind as usize) as u32;
    let (columns, size) = (COLUMNS as u32, ATLAS_TILE_PIXELS);
    ((tile % columns) * size, (tile / columns) * size, size)
}

fn kind_index(building: &BuildingLook) -> usize {
    building.kind as usize
}

/// The hedges of a garden's cell, along each side with no garden beyond it.
pub fn build_hedges(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let level = scene.level;
    let garden = |x: i64, y: i64| level.cell(x, y) == Cell::Garden;
    if !garden(x, y) {
        return;
    }
    let layer = flat_layer(scene, x, y);
    for side in [(0i64, -1i64), (1, 0), (0, 1), (-1, 0)] {
        if garden(x + side.0, y + side.1) {
            continue;
        }
        // The way along this side.
        let tangent = (side.1, side.0);
        let end = |sign: i64| {
            let next = (x + tangent.0 * sign, y + tangent.1 * sign);
            if !garden(next.0, next.1) {
                // An outside corner.
                HALF_CELL - HEDGE_INSET
            } else if garden(next.0 + side.0, next.1 + side.1) {
                // An inside corner: into the next cell to meet the hedge
                // coming the other way.
                HALF_CELL + HEDGE_INSET + HEDGE_THICK
            } else {
                HALF_CELL
            }
        };
        let hedge = Hedge {
            centre: cell_centre(x, y),
            out: Vec3::new(side.0 as f32, 0.0, side.1 as f32),
            along: Vec3::new(tangent.0 as f32, 0.0, tangent.1 as f32),
            layer,
        };
        let (low, high) = (-end(-1), end(1));
        if has_arch(scene, (x, y), side, tangent) {
            hedge.stretch(layers, scene, low, -ARCH_HALF - HEDGE_THICK, HEDGE_HEIGHT);
            hedge.stretch(layers, scene, ARCH_HALF + HEDGE_THICK, high, HEDGE_HEIGHT);
            hedge.arch(layers, scene);
        } else {
            hedge.stretch(layers, scene, low, high, HEDGE_HEIGHT);
        }
    }
}

/// Whether this cell of a stretch of hedge has the arch: always if a door
/// faces it; otherwise some stretches long enough have one, not at their ends.
fn has_arch(scene: &Scene, (x, y): (i64, i64), side: (i64, i64), tangent: (i64, i64)) -> bool {
    let level = scene.level;
    let on_stretch = |x: i64, y: i64| {
        level.cell(x, y) == Cell::Garden && level.cell(x + side.0, y + side.1) != Cell::Garden
    };
    let door_at = |x: i64, y: i64| {
        let (dx, dy) = (x + side.0, y + side.1);
        level.cell(dx, dy) == Cell::BuildingDoor
            && scene
                .buildings
                .iter()
                .any(|b| b.door == (dx, dy) && b.faces == (-side.0, -side.1))
    };
    if door_at(x, y) {
        return true;
    }
    let mut start = (x, y);
    while on_stretch(start.0 - tangent.0, start.1 - tangent.1) {
        start = (start.0 - tangent.0, start.1 - tangent.1);
    }
    let mut length = 0;
    let mut has_door = false;
    let mut at = start;
    while on_stretch(at.0, at.1) {
        has_door |= door_at(at.0, at.1);
        length += 1;
        at = (at.0 + tangent.0, at.1 + tangent.1);
    }
    if has_door || length < ARCH_RUN {
        return false;
    }
    let hash = stretch_hash(start, side);
    if hash % 100 >= ARCH_PERCENT {
        return false;
    }
    let chosen = 1 + (hash >> 8) % (length as u64 - 2);
    (x - start.0).abs() + (y - start.1).abs() == chosen as i64
}

/// A number for a stretch of hedge, for looks: all clients draw the same.
fn stretch_hash((x, y): (i64, i64), side: (i64, i64)) -> u64 {
    let mixed = (x as u64)
        .wrapping_mul(0x9e37_79b9_7f4a_7c15)
        .wrapping_add((y as u64).wrapping_mul(0xc2b2_ae3d_27d4_eb4f))
        .wrapping_add((side.0 + 2 * side.1 + 3) as u64);
    let mixed = (mixed ^ (mixed >> 29)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed ^ (mixed >> 32)
}

/// A hedge along one side of a cell.
struct Hedge {
    centre: Vec3,
    out: Vec3,
    along: Vec3,
    layer: Layer,
}

impl Hedge {
    /// A box from `low` to `high` along the side, `up` high.
    fn stretch(&self, layers: &mut Layers, scene: &Scene, low: f32, high: f32, up: f32) {
        if low < high {
            self.solid(layers, scene, (low, high), (0.0, up));
        }
    }

    /// Two posts and a stepped top, all of hedge.
    fn arch(&self, layers: &mut Layers, scene: &Scene) {
        let (inner, outer) = (ARCH_HALF, ARCH_HALF + HEDGE_THICK);
        self.solid(layers, scene, (-outer, -inner), (0.0, ARCH_HEIGHT));
        self.solid(layers, scene, (inner, outer), (0.0, ARCH_HEIGHT));
        let beam = ARCH_HEIGHT - HEDGE_THICK;
        self.solid(layers, scene, (-outer, outer), (beam, ARCH_HEIGHT));
        // Corners stepping in, to look rounded.
        let step = ARCH_HALF / 3.0;
        let low = beam - HEDGE_THICK;
        self.solid(layers, scene, (-inner, -inner + step), (low, beam));
        self.solid(layers, scene, (inner - step, inner), (low, beam));
    }

    /// A box of hedge, `(low, high)` along the side and up.
    fn solid(&self, layers: &mut Layers, scene: &Scene, along: (f32, f32), up: (f32, f32)) {
        let far = HALF_CELL - HEDGE_INSET;
        let near = far - HEDGE_THICK;
        let at = |a: f32, o: f32, h: f32| self.centre + self.along * a + self.out * o + Vec3::Y * h;
        let shade = layer_colour(self.layer);
        let side = shade * HEDGE_SIDE_SHADE;
        let uv = |u: f32, v: f32| atlas_uv(HEDGE_TILE, u, v);
        let uvs = [uv(0.1, 0.5), uv(0.9, 0.5), uv(0.9, 0.1), uv(0.1, 0.1)];
        let face = |layers: &mut Layers, corners: [Vec3; 4], normal: Vec3, colour: Vec3| {
            emit(layers, scene, self.layer, corners, normal, uvs, [colour; 4]);
        };
        let (a0, a1) = along;
        let (h0, h1) = up;
        face(
            layers,
            [
                at(a0, near, h1),
                at(a1, near, h1),
                at(a1, far, h1),
                at(a0, far, h1),
            ],
            Vec3::Y,
            shade,
        );
        face(
            layers,
            [
                at(a0, far, h0),
                at(a1, far, h0),
                at(a1, far, h1),
                at(a0, far, h1),
            ],
            self.out,
            side,
        );
        face(
            layers,
            [
                at(a0, near, h0),
                at(a1, near, h0),
                at(a1, near, h1),
                at(a0, near, h1),
            ],
            -self.out,
            side,
        );
        face(
            layers,
            [
                at(a1, near, h0),
                at(a1, far, h0),
                at(a1, far, h1),
                at(a1, near, h1),
            ],
            self.along,
            side,
        );
        face(
            layers,
            [
                at(a0, near, h0),
                at(a0, far, h0),
                at(a0, far, h1),
                at(a0, near, h1),
            ],
            -self.along,
            side,
        );
    }
}

#[cfg(test)]
mod tests {
    use shared::dungeon::{Level, Sight};
    use shared::site::Rect as CellRect;

    use super::super::mesh::{LightGrid, View, build};
    use super::super::theme::Shape;
    use super::*;

    /// A stone building in the middle of a road.
    fn town() -> (Level, Vec<BuildingLook>) {
        let mut level = Level::new(0, 0, 11, 11);
        for y in 0..11 {
            for x in 0..11 {
                level.set(x, y, Cell::Cobblestone);
            }
        }
        for y in 3..8 {
            for x in 3..8 {
                level.set(x, y, Cell::Wall);
                level.set_room(x, y, Some(RoomId(0)));
            }
        }
        let look = BuildingLook {
            id: 0,
            kind: BuildingKind::Bank,
            walls: WallKind::Stone,
            roof: RoofKind::SlateTiles,
            rects: vec![CellRect {
                x: 3,
                y: 3,
                width: 5,
                height: 5,
            }],
            door: (5, 7),
            faces: (0, 1),
        };
        level.set(5, 7, Cell::BuildingDoor);
        (level, vec![look])
    }

    /// Every wall face, on any side, shows the building's own walls.
    #[test]
    fn all_sides_of_a_building_use_its_wall_picture() {
        let (level, buildings) = town();
        let mut sight = Sight::new(&level);
        sight.add_view(&level, 9, 9, None);
        let view = View {
            cell: (9, 9),
            sight: sight.clone(),
        };
        let light = LightGrid::filled(&level, Vec3::ONE);
        let camera = Vec3::new(1000.0, 800.0, 1000.0);
        let meshes = build(
            &level,
            &[view],
            &sight,
            &sight,
            &light,
            &[],
            &buildings,
            camera,
            Shape::Built,
        );
        let mesh = &meshes.solid[Surface::Town as usize];
        let normals = mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .and_then(|v| v.as_float3())
            .unwrap();
        let bevy::mesh::VertexAttributeValues::Float32x2(uvs) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!("no picture coordinates");
        };
        let tile = wall_tile(WallKind::Stone);
        let (low, high) = (atlas_uv(tile, 0.0, 0.0), atlas_uv(tile, 1.0, 1.0));
        let door_tile = FIRST_DOOR_TILE + BuildingKind::Bank as usize;
        let (door_low, door_high) = (atlas_uv(door_tile, 0.0, 0.0), atlas_uv(door_tile, 1.0, 1.0));
        let inside = |uv: &[f32; 2], low: [f32; 2], high: [f32; 2]| {
            (low[0]..=high[0]).contains(&uv[0]) && (low[1]..=high[1]).contains(&uv[1])
        };
        let mut sides = 0;
        for (normal, uv) in normals.iter().zip(uvs) {
            if normal[1] == 0.0 {
                sides += 1;
                assert!(
                    inside(uv, low, high) || inside(uv, door_low, door_high),
                    "{normal:?}: {uv:?}"
                );
            }
        }
        assert!(sides > 0);
    }
}
