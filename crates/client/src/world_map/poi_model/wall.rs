//! A wall round a building (`PoiWall`: a walled fort's), by what it is made
//! of: dark logs, boards, rough stone or pale dressed stone, the stone ones
//! with merlons, and a gap with a post each side for each gate.

use bevy::prelude::*;
use shared::site::{PoiWall, WallMaterial};

use super::{PoiModel, Rgb, STONE, WOOD, half_size};

const LOG: Rgb = [92, 64, 40];
const DRESSED: Rgb = [190, 186, 174];

/// How thick the wall is, how wide a gate's gap is, and how far apart
/// merlons and logs' points are, in grid points.
const THICK: f32 = 0.22;
const GATE_HALF: f32 = 0.45;
const MERLON_SPACING: f32 = 0.7;
const LOG_SPACING: f32 = 0.45;

/// The model's scale for a building `width` × `height` cells with this wall
/// round it, and `beyond` cells more outside it on every side (a castle's
/// towers and moat), in grid points a cell: the whole of it fits as a building
/// that size would.
pub(super) fn scale(width: u16, height: u16, wall: &PoiWall, beyond: f32) -> f32 {
    let wide = f32::from(width) + f32::from(wall.west) + f32::from(wall.east) + 2.0 + 2.0 * beyond;
    let tall =
        f32::from(height) + f32::from(wall.north) + f32::from(wall.south) + 2.0 + 2.0 * beyond;
    let most = wide.max(tall);
    2.0 * half_size(most as u16) / most
}

/// A material's colour, the wall's height, and whether it has merlons.
pub(super) fn build_of(material: WallMaterial) -> (Rgb, f32, bool) {
    match material {
        WallMaterial::Log => (LOG, 1.2, false),
        WallMaterial::Plank => (WOOD, 1.1, false),
        WallMaterial::Stone => (STONE, 1.3, true),
        WallMaterial::Block => (DRESSED, 1.5, true),
    }
}

/// The wall round a building `half` (in grid points) at the origin, `per_cell`
/// grid points a cell.
pub(super) fn wall(model: &mut PoiModel, half: Vec2, wall: &PoiWall, per_cell: f32) {
    let (colour, height, merlons) = build_of(wall.material);
    let out = |cells: u8| (f32::from(cells) + 0.5) * per_cell;
    let (west, east) = (-half.x - out(wall.west), half.x + out(wall.east));
    let (north, south) = (-half.y - out(wall.north), half.y + out(wall.south));
    // North, east, south, west: from one corner to the next.
    let sides = [
        (Vec2::new(west, north), Vec2::new(east, north)),
        (Vec2::new(east, north), Vec2::new(east, south)),
        (Vec2::new(west, south), Vec2::new(east, south)),
        (Vec2::new(west, north), Vec2::new(west, south)),
    ];
    for (&(from, to), &gate) in sides.iter().zip(&wall.gates) {
        let middle = (from + to) / 2.0;
        let way = (to - from).normalize_or_zero();
        let pieces = if gate {
            vec![
                (from, middle - way * GATE_HALF),
                (middle + way * GATE_HALF, to),
            ]
        } else {
            vec![(from, to)]
        };
        for (a, b) in pieces {
            let half = ((b - a).abs() / 2.0).max(Vec2::splat(THICK));
            model.block((a + b) / 2.0, half, height, colour);
            if wall.material == WallMaterial::Log {
                // The logs' points along the top.
                let count = ((b - a).length() / LOG_SPACING).floor() as usize;
                for i in 0..=count {
                    let at = a.lerp(b, i as f32 / count.max(1) as f32);
                    model.cone(
                        at,
                        &super::rectangle(Vec2::splat(THICK)),
                        height,
                        0.3,
                        colour,
                    );
                }
            }
            if merlons {
                let length = (b - a).length();
                let count = (length / MERLON_SPACING).floor() as usize;
                for i in 0..=count {
                    let at = a.lerp(b, i as f32 / count.max(1) as f32);
                    model.prism(
                        at,
                        &super::rectangle(Vec2::splat(THICK)),
                        (height, height + 0.35),
                        colour,
                    );
                }
            }
        }
        if gate {
            for side in [-1.0, 1.0] {
                let at = middle + way * side * (GATE_HALF + THICK);
                model.block(at, Vec2::splat(THICK * 1.4), height + 0.5, colour);
            }
        }
    }
}
