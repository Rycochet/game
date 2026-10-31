//! What only farms have: fields, what grows in them, and their fences and gates.
//!
//! What stage a field is at is purely for looks, so it is chosen here. For now
//! that is from the number the server sends with it; the time of year is to
//! come into it. Crops are thin upright pictures, standing in a `#` in each cell,
//! with their own mesh, which only changes with what is in sight.

use bevy::prelude::*;
use shared::dungeon::{Cell, Sight};
use shared::rng::Seed;
use shared::site::FieldLook;

use super::CELL_SIZE;
use super::mesh::Builder;
use super::town::{self, FENCE_TILE, GRASS_TILE, PLOUGHED_TILE};
use super::walls::{Layer, Layers, Scene};

const HALF_CELL: f32 = CELL_SIZE / 2.0;

/// The picture of crops as they grow: one tile per stage, side by side.
pub const CROPS_PICTURE: &str = "field_crops.png";
const CROP_TILES: f32 = 3.0;

/// Pixels across a cell, for crops' heights.
const CELL_PIXELS: f32 = 32.0;

/// Where a cell's crops stand, each way, as a share of the cell from its centre.
const CROP_ROWS: [f32; 2] = [-0.25, 0.25];

/// A fence: its height, its rails' heights, how thick posts and rails are.
const FENCE_HEIGHT: f32 = CELL_SIZE * 0.6;
const RAIL_HEIGHTS: [f32; 2] = [CELL_SIZE * 0.25, CELL_SIZE * 0.5];
const POST_HALF: f32 = CELL_SIZE * 0.05;
const RAIL_HALF: f32 = CELL_SIZE * 0.025;

/// How far a gate's posts stand from the middle of its cell. Its leaf is as long
/// as the gap between them, and has a bar this thick upright at its open end.
const GATE_POST: f32 = CELL_SIZE * 0.42;
const GATE_END_HALF: f32 = CELL_SIZE * 0.035;

/// The sides of a fence's wood are darker than its top.
const WOOD_SIDE_SHADE: f32 = 0.8;

/// For choosing what each cell of a ruined field has gone back to.
const RUINED_TILES: Seed = Seed::new(0x6669_656c_6473);

/// How far a field has grown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Bare earth, in furrows running north to south.
    Ploughed,
    /// Flat green.
    Grass,
    /// Green, and 6 pixels tall.
    Short,
    /// Turning yellow-green, and 12 pixels tall.
    Tall,
    /// Yellow and ready to harvest, 18 pixels tall.
    Ripe,
}

impl Stage {
    const CROPS: [Self; 5] = [
        Self::Ploughed,
        Self::Grass,
        Self::Short,
        Self::Tall,
        Self::Ripe,
    ];
    const PASTURE: [Self; 2] = [Self::Grass, Self::Short];

    /// The stage of a field. Pasture is only ever kept grazed.
    pub fn of(field: &FieldLook) -> Self {
        let choices: &[Self] = if field.livestock {
            &Self::PASTURE
        } else {
            &Self::CROPS
        };
        choices[field.variety as usize % choices.len()]
    }

    /// What stands in the field: which tile of the crops' picture, and how
    /// many pixels tall.
    fn crop(self) -> Option<(f32, f32)> {
        match self {
            Self::Ploughed | Self::Grass => None,
            Self::Short => Some((0.0, 6.0)),
            Self::Tall => Some((1.0, 12.0)),
            Self::Ripe => Some((2.0, 18.0)),
        }
    }
}

/// What the ground of a cell of a ruined field has gone back to: mostly grass,
/// some still ploughed, some (`None`) the land's own. All clients agree.
fn ruined_tile(x: i64, y: i64) -> Option<usize> {
    match RUINED_TILES.at(&[x, y]) % 100 {
        0..20 => None,
        20..40 => Some(PLOUGHED_TILE),
        _ => Some(GRASS_TILE),
    }
}

/// The flat ground of a field's cell. False if it is the land's own, to be
/// drawn as any outside ground is.
pub fn build_field(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) -> bool {
    let tile = if scene.ruined {
        ruined_tile(x, y)
    } else {
        let field = scene.fields.iter().find(|field| field.contains(x, y));
        Some(match field.map(Stage::of) {
            Some(Stage::Ploughed) => PLOUGHED_TILE,
            _ => GRASS_TILE,
        })
    };
    let Some(tile) = tile else {
        return false;
    };
    let patch = Rect::from_corners(Vec2::splat(-HALF_CELL), Vec2::splat(HALF_CELL));
    let layer = town::flat_layer(scene, x, y);
    town::flat_patch(layers, scene, layer, town::cell_centre(x, y), patch, tile);
    true
}

/// A block of wood. Its bottom isn't drawn.
fn wood(layers: &mut Layers, scene: &Scene, layer: Layer, low: Vec3, high: Vec3) {
    let uvs = [(0.1, 0.5), (0.9, 0.5), (0.9, 0.1), (0.1, 0.1)];
    let uvs = uvs.map(|(u, v)| town::atlas_uv(FENCE_TILE, u, v));
    let top = town::layer_colour(layer);
    let side = top * WOOD_SIDE_SHADE;
    let at = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
    let (l, h) = (low, high);
    let faces = [
        (
            [
                at(l.x, h.y, l.z),
                at(h.x, h.y, l.z),
                at(h.x, h.y, h.z),
                at(l.x, h.y, h.z),
            ],
            Vec3::Y,
            top,
        ),
        (
            [
                at(l.x, l.y, h.z),
                at(h.x, l.y, h.z),
                at(h.x, h.y, h.z),
                at(l.x, h.y, h.z),
            ],
            Vec3::Z,
            side,
        ),
        (
            [
                at(l.x, l.y, l.z),
                at(h.x, l.y, l.z),
                at(h.x, h.y, l.z),
                at(l.x, h.y, l.z),
            ],
            Vec3::NEG_Z,
            side,
        ),
        (
            [
                at(h.x, l.y, l.z),
                at(h.x, l.y, h.z),
                at(h.x, h.y, h.z),
                at(h.x, h.y, l.z),
            ],
            Vec3::X,
            side,
        ),
        (
            [
                at(l.x, l.y, l.z),
                at(l.x, l.y, h.z),
                at(l.x, h.y, h.z),
                at(l.x, h.y, l.z),
            ],
            Vec3::NEG_X,
            side,
        ),
    ];
    for (corners, normal, colour) in faces {
        town::emit(layers, scene, layer, corners, normal, uvs, [colour; 4]);
    }
}

/// A post, standing `up` high at a point on the ground.
fn post(layers: &mut Layers, scene: &Scene, layer: Layer, at: Vec3, up: f32) {
    let half = Vec3::new(POST_HALF, 0.0, POST_HALF);
    wood(layers, scene, layer, at - half, at + half + Vec3::Y * up);
}

/// A bar from one point to another along a grid direction, `half` thick each
/// way.
fn bar(layers: &mut Layers, scene: &Scene, layer: Layer, from: Vec3, to: Vec3, half: f32) {
    let margin = Vec3::splat(half);
    wood(
        layers,
        scene,
        layer,
        from.min(to) - margin,
        from.max(to) + margin,
    );
}

/// The two rails of a fence from one point to another on the ground.
fn rails(layers: &mut Layers, scene: &Scene, layer: Layer, from: Vec3, to: Vec3) {
    for height in RAIL_HEIGHTS {
        let up = Vec3::Y * height;
        bar(layers, scene, layer, from + up, to + up, RAIL_HALF);
    }
}

/// The wood of a fence's or gate's cell (nothing on any other cell).
pub fn build_fence(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let level = scene.level;
    let cell = level.cell(x, y);
    if !matches!(cell, Cell::Fence | Cell::FieldGate) {
        return;
    }
    let layer = town::flat_layer(scene, x, y);
    let centre = town::cell_centre(x, y);
    let way = |(dx, dy): (i64, i64)| Vec3::new(dx as f32, 0.0, dy as f32);
    let sides = [(0, -1), (1, 0), (0, 1), (-1, 0)];
    if cell == Cell::Fence {
        post(layers, scene, layer, centre, FENCE_HEIGHT);
        for side in sides {
            if matches!(
                level.cell(x + side.0, y + side.1),
                Cell::Fence | Cell::FieldGate
            ) {
                let edge = centre + way(side) * HALF_CELL;
                rails(layers, scene, layer, centre, edge);
            }
        }
        return;
    }
    // A gate's cell: a post each side, and its leaf standing open at 45 degrees,
    // into the field.
    let inward = sides
        .into_iter()
        .find(|side| level.cell(x + side.0, y + side.1) == Cell::Field)
        .unwrap_or((0, 1));
    let along = way((inward.1, inward.0));
    for sign in [-1.0, 1.0] {
        let at = centre + along * sign * GATE_POST;
        post(layers, scene, layer, at, FENCE_HEIGHT);
        rails(layers, scene, layer, at, centre + along * sign * HALF_CELL);
    }
    let hinge = centre + along * GATE_POST;
    let open = (way(inward) - along).normalize();
    let end = hinge + open * GATE_POST * 2.0;
    for height in RAIL_HEIGHTS {
        slanted_rail(layers, scene, layer, hinge, end, height);
    }
    let thick = Vec3::new(GATE_END_HALF, 0.0, GATE_END_HALF);
    let (bottom, top) = (
        RAIL_HEIGHTS[0] - RAIL_HALF * 2.0,
        RAIL_HEIGHTS[1] + RAIL_HALF * 2.0,
    );
    wood(
        layers,
        scene,
        layer,
        end - thick + Vec3::Y * bottom,
        end + thick + Vec3::Y * top,
    );
}

/// A rail at `height` between two points on the ground, whichever way they lie:
/// its top and both sides.
fn slanted_rail(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    from: Vec3,
    to: Vec3,
    height: f32,
) {
    let uvs = [(0.1, 0.5), (0.9, 0.5), (0.9, 0.1), (0.1, 0.1)];
    let uvs = uvs.map(|(u, v)| town::atlas_uv(FENCE_TILE, u, v));
    let top = town::layer_colour(layer);
    let across = (to - from).cross(Vec3::Y).normalize() * RAIL_HALF;
    let up = Vec3::Y * RAIL_HALF;
    let (from, to) = (from + Vec3::Y * height, to + Vec3::Y * height);
    let faces = [
        (
            [
                from - across + up,
                to - across + up,
                to + across + up,
                from + across + up,
            ],
            Vec3::Y,
            top,
        ),
        (
            [
                from + across - up,
                to + across - up,
                to + across + up,
                from + across + up,
            ],
            across.normalize(),
            top * WOOD_SIDE_SHADE,
        ),
        (
            [
                from - across - up,
                to - across - up,
                to - across + up,
                from - across + up,
            ],
            -across.normalize(),
            top * WOOD_SIDE_SHADE,
        ),
    ];
    for (corners, normal, colour) in faces {
        town::emit(layers, scene, layer, corners, normal, uvs, [colour; 4]);
    }
}

/// Everything standing in the fields, wherever anyone can see it: nothing is
/// drawn of it from memory, or in a ruin, whose fields have run to seed.
pub fn build_crops(
    fields: &[FieldLook],
    sight: &Sight,
    remembered: &Sight,
    ruined: bool,
) -> Builder {
    let mut crops = Builder::default();
    if ruined {
        return crops;
    }
    for field in fields {
        let Some((tile, pixels)) = Stage::of(field).crop() else {
            continue;
        };
        let rect = field.rect;
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                if sight.is_seen(x, y) && remembered.is_seen(x, y) {
                    cell_crops(&mut crops, town::cell_centre(x, y), tile, pixels);
                }
            }
        }
    }
    crops
}

/// A `#` of crops in a cell: two rows each way.
fn cell_crops(crops: &mut Builder, centre: Vec3, tile: f32, pixels: f32) {
    let top = pixels / CELL_PIXELS;
    let up = Vec3::Y * top * CELL_SIZE;
    let uv = |u: f32, v: f32| [(tile + u) / CROP_TILES, v];
    let uvs = [
        uv(0.0, 1.0),
        uv(1.0, 1.0),
        uv(1.0, 1.0 - top),
        uv(0.0, 1.0 - top),
    ];
    let colours = [Vec3::ONE; 4];
    for row in CROP_ROWS {
        let offset = row * CELL_SIZE;
        for (across, normal, along) in [
            (Vec3::X * offset, Vec3::X, Vec3::Z),
            (Vec3::Z * offset, Vec3::Z, Vec3::X),
        ] {
            let middle = centre + across;
            let (a, b) = (middle - along * HALF_CELL, middle + along * HALF_CELL);
            crops.free([a, b, b + up, a + up], normal, uvs, colours);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::site::Rect;

    fn field(livestock: bool, variety: u32) -> FieldLook {
        FieldLook {
            rect: Rect {
                x: 0,
                y: 0,
                width: 8,
                height: 8,
            },
            livestock,
            variety,
        }
    }

    #[test]
    fn crops_go_through_every_stage_and_pasture_stays_short() {
        for (variety, stage) in Stage::CROPS.into_iter().enumerate() {
            assert_eq!(Stage::of(&field(false, variety as u32)), stage);
        }
        for variety in 0..20 {
            let stage = Stage::of(&field(true, variety));
            assert!(matches!(stage, Stage::Grass | Stage::Short));
        }
    }

    #[test]
    fn crops_grow_taller() {
        let heights = [Stage::Short, Stage::Tall, Stage::Ripe].map(|s| s.crop().unwrap().1);
        assert_eq!(heights, [6.0, 12.0, 18.0]);
        assert_eq!(Stage::Grass.crop(), None);
    }

    /// About 20% land, 20% ploughed and 60% grass.
    #[test]
    fn ruined_fields_are_a_mix() {
        let mut counts = [0; 3];
        for y in 0..200 {
            for x in 0..200 {
                counts[match ruined_tile(x, y) {
                    None => 0,
                    Some(PLOUGHED_TILE) => 1,
                    Some(_) => 2,
                }] += 1;
            }
        }
        assert!((7000..9000).contains(&counts[0]), "{counts:?}");
        assert!((7000..9000).contains(&counts[1]), "{counts:?}");
        assert!((23000..25000).contains(&counts[2]), "{counts:?}");
    }
}
