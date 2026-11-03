//! A garden's hedges, and where their arches go.

use bevy::prelude::*;
use shared::dungeon::Cell;

use super::super::walls::{Layers, Scene};
use super::{
    ARCH_HALF, ARCH_PERCENT, ARCH_RUN, HALF_CELL, HEDGE_HEIGHT, HEDGE_INSET, HEDGE_THICK, Hedge,
    cell_centre, flat_layer,
};

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
pub(super) fn has_arch(
    scene: &Scene,
    (x, y): (i64, i64),
    side: (i64, i64),
    tangent: (i64, i64),
) -> bool {
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
pub(super) fn stretch_hash((x, y): (i64, i64), side: (i64, i64)) -> u64 {
    let mixed = (x as u64)
        .wrapping_mul(0x9e37_79b9_7f4a_7c15)
        .wrapping_add((y as u64).wrapping_mul(0xc2b2_ae3d_27d4_eb4f))
        .wrapping_add((side.0 + 2 * side.1 + 3) as u64);
    let mixed = (mixed ^ (mixed >> 29)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed ^ (mixed >> 32)
}
