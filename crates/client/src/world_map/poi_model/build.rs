//! The model of a poi, towns and farm fields.

use bevy::prelude::*;
use shared::poi::PoiKind;
use shared::site::{Outline, PoiLook};

use super::{
    ADOBE, DARK_STONE, EARTH, FARMHOUSE_SHARE, GRASS, GREY_ROOF, MOSS, PLASTER, PoiModel, REACH,
    RED_ROOF, STONE, THATCH, WOOD, building, circle, half_size, hashed, oasis, rectangle, ruined,
    underground,
};

/// The model of a poi. `variety` makes towns of the same size differ.
pub fn build(kind: PoiKind, look: PoiLook, variety: u64) -> PoiModel {
    let mut model = PoiModel::default();
    match look {
        PoiLook::Town { buildings } => town(&mut model, buildings, variety),
        PoiLook::Building {
            outline,
            width,
            height,
            floors,
            ruin,
            wall,
        } => {
            let mut half = Vec2::new(half_size(width), half_size(height));
            // A wall round it takes room: the building inside it is smaller.
            if let Some(wall) = &wall {
                let per_cell = super::wall::scale(width, height, wall);
                half = Vec2::new(f32::from(width), f32::from(height)) * per_cell / 2.0;
                super::wall::wall(&mut model, half, wall, per_cell);
            }
            if ruin > 0 {
                ruined(&mut model, kind, outline, half, floors, ruin, variety);
            } else {
                building(&mut model, kind, outline, half, floors);
            }
        }
        PoiLook::Cave { entrance } => underground(&mut model, kind, entrance),
        PoiLook::Oasis { pools } => oasis(&mut model, pools, variety),
        PoiLook::Farm {
            width,
            height,
            ruin,
            fields,
        } => {
            let half = Vec2::new(half_size(width), half_size(height)) * FARMHOUSE_SHARE;
            farm_fields(&mut model, fields, ruin, variety);
            if ruin > 0 {
                ruined(&mut model, kind, Outline::Rectangle, half, 2, ruin, variety);
            } else {
                building(&mut model, kind, Outline::Rectangle, half, 2);
            }
        }
    }
    model
}

/// More houses the bigger it is, packed in a disc; the biggest get walls and a
/// keep. Spread by the golden angle, which fills a disc evenly.
pub(super) fn town(model: &mut PoiModel, buildings: u16, variety: u64) {
    const GOLDEN_ANGLE: f32 = 2.399_963;
    let count = 3 + (f32::from(buildings).sqrt() * 1.6) as usize;
    let big = buildings >= 26;
    let radius = if big {
        REACH * 0.8
    } else {
        3.2 + count as f32 * 0.3
    };
    let turn = (variety % 8) as f32 * 0.7;
    for i in 0..count {
        let share = (i as f32 + 0.5) / count as f32;
        let angle = i as f32 * GOLDEN_ANGLE + turn;
        let at = Vec2::from_angle(angle) * share.sqrt() * radius;
        let pick = (i as u64 + variety).wrapping_mul(2_654_435_761) >> 7;
        let walls = [PLASTER, ADOBE, STONE, WOOD][(pick % 4) as usize];
        let roof = [RED_ROOF, GREY_ROOF, THATCH, RED_ROOF][((pick >> 3) % 4) as usize];
        let size = 0.6 + ((pick >> 5) % 4) as f32 * 0.1;
        model.house(at, Vec2::new(size, size * 0.8), 1.0 + size, walls, roof);
    }
    if big {
        let keep = rectangle(Vec2::splat(1.3));
        model.prism(Vec2::ZERO, &keep, (0.0, 4.5), DARK_STONE);
        model.battlements(Vec2::ZERO, &keep, 4.5, DARK_STONE);
    }
    if buildings >= 61 {
        for corner in circle(Vec2::splat(REACH * 0.92), 8) {
            model.block(corner, Vec2::splat(0.6), 2.6, STONE);
        }
    }
}

/// Flat patches round a farmhouse, one per field, up to its eight places.
pub(super) fn farm_fields(model: &mut PoiModel, fields: u8, ruin: u8, variety: u64) {
    const PLACES: [(f32, f32, f32); 8] = [
        (0.0, 5.5, 1.7),
        (-4.0, 4.0, 1.5),
        (4.0, 4.0, 1.5),
        (-5.5, 0.0, 1.7),
        (5.5, 0.0, 1.7),
        (-4.0, -4.0, 1.5),
        (0.0, -5.5, 1.7),
        (4.0, -4.0, 1.5),
    ];
    let mut number = variety;
    for &(x, z, half) in PLACES.iter().take(usize::from(fields)) {
        let colour = if ruin > 0 {
            MOSS
        } else {
            [EARTH, GRASS, THATCH][(hashed(&mut number) * 3.0) as usize % 3]
        };
        let shape = rectangle(Vec2::splat(half));
        model.prism(Vec2::new(x, z), &shape, (0.0, 0.1), colour);
    }
}
