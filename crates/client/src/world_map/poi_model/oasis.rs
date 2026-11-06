//! An oasis from the world map: its pools, ringed by grass, with palms.

use bevy::prelude::*;

use super::{GRASS, MOSS, PoiModel, WOOD, circle, hashed};

/// Fresh water.
const WATER: [u8; 3] = [58, 112, 176];

/// Where one or two pools lie, and how big each is: two side by side.
const ONE_POOL: [(f32, f32, f32); 1] = [(0.0, 0.0, 3.6)];
const TWO_POOLS: [(f32, f32, f32); 2] = [(-2.6, -1.2, 2.6), (2.8, 1.6, 2.2)];

/// How many palms stand round each pool, how tall, and their crowns' size.
const PALMS: usize = 5;
const TRUNK: f32 = 2.6;
const CROWN: f32 = 1.1;

/// The pools, the grass round them and the palms on it.
pub(super) fn oasis(model: &mut PoiModel, pools: u8, variety: u64) {
    let places: &[(f32, f32, f32)] = if pools >= 2 { &TWO_POOLS } else { &ONE_POOL };
    let mut number = variety;
    for &(x, z, radius) in places {
        let at = Vec2::new(x, z);
        model.prism(
            at,
            &circle(Vec2::splat(radius + 1.2), 20),
            (0.0, 0.08),
            GRASS,
        );
        model.prism(at, &circle(Vec2::splat(radius), 20), (0.0, 0.14), WATER);
        let turn = hashed(&mut number) * std::f32::consts::TAU;
        for palm in 0..PALMS {
            let angle = turn + palm as f32 * std::f32::consts::TAU / PALMS as f32;
            let spot = at + Vec2::from_angle(angle) * (radius + 0.7);
            let height = TRUNK * (0.8 + 0.4 * hashed(&mut number));
            model.block(spot, Vec2::splat(0.14), height, WOOD);
            let crown = circle(Vec2::splat(CROWN), 6);
            model.cone(spot, &crown, height - 0.3, 0.6, MOSS);
        }
    }
}
