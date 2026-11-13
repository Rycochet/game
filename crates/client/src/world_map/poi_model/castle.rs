//! A castle (`PoiCastle` beside its `PoiWall`): the keep, the towers on its
//! wall where `tower_spots` puts them (short ones by the gate), and any moat
//! round it all with a bridge from the gate.

use bevy::prelude::*;
use shared::site::{PoiCastle, PoiWall, Rect, TOWER_SIZE, tower_spots};

use super::wall::build_of;
use super::{DARK_STONE, FLOOR, PoiModel, Rgb, WOOD, rectangle, ring};
use shared::site::Outline;

const WATER: Rgb = [58, 112, 176];

/// The narrowest a moat is drawn, in grid points, so it shows at the world
/// map's scale.
const LEAST_MOAT: f32 = 1.0;

/// A tower's floor on the model, above the wall.
const TOWER_FLOOR: f32 = 0.9;

/// How far a castle reaches past its wall, in cells: half a tower, the bank
/// and the moat.
pub(super) fn beyond(castle: &PoiCastle) -> f32 {
    let bank = if castle.moat > 0 { 1.0 } else { 0.0 };
    (TOWER_SIZE / 2) as f32 + 0.5 + bank + f32::from(castle.moat)
}

/// The towers and moat of a castle whose keep is `half` (in grid points) at
/// the origin, with `wall` round it, `per_cell` grid points a cell.
pub(super) fn castle(
    model: &mut PoiModel,
    (width, height): (u16, u16),
    half: Vec2,
    (wall, castle): (&PoiWall, &PoiCastle),
    per_cell: f32,
) {
    let (colour, wall_height, _) = build_of(wall.material);
    // The wall's north-west cell's middle, and the rectangle it runs round.
    let corner = Vec2::new(
        -half.x - (f32::from(wall.west) + 0.5) * per_cell,
        -half.y - (f32::from(wall.north) + 0.5) * per_cell,
    );
    let rect = Rect {
        x: 0,
        y: 0,
        width: i64::from(width) + i64::from(wall.west) + i64::from(wall.east) + 2,
        height: i64::from(height) + i64::from(wall.north) + i64::from(wall.south) + 2,
    };
    for spot in tower_spots(rect) {
        let at = corner + Vec2::new(spot.x as f32, spot.y as f32) * per_cell;
        let floors = if spot.by_gate {
            castle.tower_floors.min(2)
        } else {
            castle.tower_floors
        };
        let top = wall_height + TOWER_FLOOR * f32::from(floors);
        let side = Vec2::splat(spot.size as f32 / 2.0 * per_cell);
        model.block(at, side, top, colour);
        model.battlements(at, &rectangle(side * 0.8), top, colour);
    }
    if castle.moat > 0 {
        // From the bank past the towers to the moat's outer edge, each way.
        let reach = (TOWER_SIZE / 2) as f32 + 1.0;
        let size = Vec2::new(rect.width as f32 - 1.0, rect.height as f32 - 1.0) * per_cell;
        let middle = corner + size / 2.0;
        let inner = size / 2.0 + Vec2::splat((reach + 0.5) * per_cell);
        let wide = (f32::from(castle.moat) * per_cell).max(LEAST_MOAT);
        let outer = inner + Vec2::splat(wide);
        let band = (outer - inner) / 2.0;
        let strips = [
            (
                Vec2::new(0.0, -(inner.y + band.y)),
                Vec2::new(outer.x, band.y),
            ),
            (Vec2::new(0.0, inner.y + band.y), Vec2::new(outer.x, band.y)),
            (
                Vec2::new(-(inner.x + band.x), 0.0),
                Vec2::new(band.x, inner.y),
            ),
            (Vec2::new(inner.x + band.x, 0.0), Vec2::new(band.x, inner.y)),
        ];
        for (at, half) in strips {
            model.prism(middle + at, &rectangle(half), (0.0, 0.12), WATER);
        }
        // The bridge from the gate, two cells wide.
        let gate = Vec2::new(middle.x, middle.y + inner.y + band.y);
        let bridge = Vec2::new(per_cell, band.y + per_cell * 0.5);
        model.prism(gate, &rectangle(bridge), (0.0, 0.25), WOOD);
    }
}

/// A castle's keep: its walls rising above the outer wall, with battlements.
pub(super) fn keep(model: &mut PoiModel, outline: Outline, half: Vec2, floors: u8) {
    let height = FLOOR * f32::from(floors.max(1)) * 0.8;
    let shape = ring(outline, half);
    model.prism(Vec2::ZERO, &shape, (0.0, height), DARK_STONE);
    model.battlements(Vec2::ZERO, &shape, height, DARK_STONE);
    model.doorway(0.0, half.y, 0.6, 1.5);
}
