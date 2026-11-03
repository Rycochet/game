//! What stands over a cave or dungeon.

use bevy::prelude::*;
use shared::poi::PoiKind;
use shared::site::CaveEntrance;

use super::{
    BLOOD, DARK_STONE, EARTH, GRASS, ICE, MOSS, PLASTER, PoiModel, ROCK, Rgb, STONE, WOOD, circle,
    rectangle,
};

/// What stands over a cave or dungeon: its mouth, or the way down.
pub(super) fn underground(model: &mut PoiModel, kind: PoiKind, entrance: CaveEntrance) {
    use PoiKind::*;
    let at = Vec2::ZERO;
    let tomb = |model: &mut PoiModel, stone: Rgb, roof: Rgb| {
        model.block(at, Vec2::new(3.0, 2.4), 2.4, stone);
        model.cone(at, &rectangle(Vec2::new(3.3, 2.7)), 2.4, 1.6, roof);
        model.doorway(at.x, 2.4, 0.7, 1.6);
    };
    match kind {
        Crypt => tomb(model, DARK_STONE, DARK_STONE),
        MossyCrypt => tomb(model, DARK_STONE, MOSS),
        BloodyCrypt => tomb(model, DARK_STONE, BLOOD),
        Mausoleum => {
            model.block(at, Vec2::new(3.8, 3.0), 2.8, STONE);
            for corner in rectangle(Vec2::new(4.2, 3.4)) {
                model.block(corner, Vec2::splat(0.35), 3.2, PLASTER);
            }
            model.cone(at, &rectangle(Vec2::new(4.8, 4.0)), 3.2, 2.2, DARK_STONE);
            model.doorway(at.x, 3.0, 0.8, 1.8);
        }
        Grave => {
            model.block(at + Vec2::new(0.0, 0.6), Vec2::new(1.2, 2.0), 0.4, EARTH);
            model.block(
                at + Vec2::new(0.0, -1.8),
                Vec2::new(1.0, 0.25),
                2.2,
                DARK_STONE,
            );
        }
        Barrel | Barrels => {
            let spots: &[Vec2] = if kind == Barrel {
                &[Vec2::ZERO]
            } else {
                &[
                    Vec2::new(-1.3, 0.6),
                    Vec2::new(1.3, 0.8),
                    Vec2::new(0.0, -1.2),
                ]
            };
            for &spot in spots {
                let round = circle(Vec2::splat(1.1), 8);
                model.prism(spot, &round, (0.0, 2.6), WOOD);
            }
        }
        Boulders => {
            for (spot, size, height) in [
                (Vec2::new(-2.0, 1.0), 2.2, 2.0),
                (Vec2::new(2.2, 0.4), 1.8, 1.4),
                (Vec2::new(0.2, -2.2), 2.6, 2.6),
            ] {
                let round = circle(Vec2::splat(size), 6);
                model.prism(spot, &round, (0.0, height * 0.5), ROCK);
                model.cone(spot, &round, height * 0.5, height * 0.8, ROCK);
            }
        }
        Ruins => {
            for (spot, half, height) in [
                (Vec2::new(-3.0, 0.0), Vec2::new(0.5, 3.0), 2.6),
                (Vec2::new(0.5, -3.0), Vec2::new(3.0, 0.5), 1.5),
                (Vec2::new(3.0, 1.5), Vec2::new(0.5, 1.2), 0.9),
            ] {
                model.block(spot, half, height, STONE);
            }
        }
        _ => {
            // A natural cave: a mound with its mouth to the south.
            let colour = match kind {
                GrassyCave => GRASS,
                EarthCave => EARTH,
                IceCave => ICE,
                _ => ROCK,
            };
            let base = circle(Vec2::new(5.5, 4.5), 9);
            model.prism(at, &base, (0.0, 1.4), colour);
            model.cone(at, &base, 1.4, 3.4, colour);
            if entrance == CaveEntrance::Opening {
                model.doorway(at.x, 4.5, 1.3, 1.3);
            }
        }
    }
}
