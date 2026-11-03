//! Buildings, whole or ruined.

use bevy::prelude::*;
use shared::poi::PoiKind;
use shared::site::Outline;

use super::{
    ADOBE, DARK_STONE, EARTH, FLOOR, GRASS, GREY_ROOF, ICE, MOSS, PLASTER, PoiModel, RED_ROOF,
    SAND, SNOW, STONE, THATCH, WALL_STONE, WOOD, circle, hashed, rectangle, ring,
};

pub(super) fn building(
    model: &mut PoiModel,
    kind: PoiKind,
    outline: Outline,
    half: Vec2,
    floors: u8,
) {
    use PoiKind::*;
    let at = Vec2::ZERO;
    let height = FLOOR * f32::from(floors.max(1));
    let shape = ring(outline, half);
    let walls_and_roof = |model: &mut PoiModel, walls, roof, rise| {
        model.prism(at, &shape, (0.0, height), walls);
        model.cone(at, &ring(outline, half * 1.08), height, rise, roof);
        model.doorway(at.x, half.y, 0.5, 1.4);
    };
    let flat = |model: &mut PoiModel, walls| {
        model.prism(at, &shape, (0.0, height), walls);
        model.doorway(at.x, half.y, 0.5, 1.4);
    };
    let rise = half.min_element() * 0.8;
    match kind {
        ThatchedHut => walls_and_roof(model, WOOD, THATCH, rise),
        StiltHut | GreenStiltHut => {
            let roof = if kind == StiltHut { THATCH } else { MOSS };
            for post in circle(half * 0.8, 4) {
                model.block(post, Vec2::splat(0.2), 1.5, WOOD);
            }
            // One storey on its posts: the ground under it counts as a floor.
            model.prism(at, &shape, (1.5, 1.5 + FLOOR), WOOD);
            model.cone(at, &ring(outline, half * 1.08), 1.5 + FLOOR, rise, roof);
        }
        RedTent | BlueTent | GreenTent => {
            let cloth = match kind {
                RedTent => RED_ROOF,
                BlueTent => [58, 84, 164],
                _ => MOSS,
            };
            model.prism(at, &shape, (0.0, 0.05), EARTH);
            model.cone(at, &shape, 0.05, half.min_element() * 1.3, cloth);
        }
        IceShelter => {
            model.prism(at, &shape, (0.0, 0.05), SNOW);
            let lower = ring(outline, half * 0.75);
            model.prism(at, &shape, (0.05, 0.9), ICE);
            model.prism(at, &lower, (0.9, 1.5), SNOW);
            model.cone(at, &lower, 1.5, half.min_element() * 0.5, ICE);
        }
        AdobeHouse | AdobeStore => flat(model, ADOBE),
        RedRoofCottage => walls_and_roof(model, PLASTER, RED_ROOF, rise),
        GreyRoofCottage => walls_and_roof(model, PLASTER, GREY_ROOF, rise),
        StoneCottage => walls_and_roof(model, STONE, GREY_ROOF, rise),
        SnowyCottage => walls_and_roof(model, PLASTER, SNOW, rise),
        Warehouse => walls_and_roof(model, WOOD, GREY_ROOF, rise * 0.6),
        ArcheryRange => {
            model.prism(at, &shape, (0.0, 0.1), GRASS);
            model.block(
                Vec2::new(0.0, -half.y * 0.7),
                Vec2::new(half.x * 0.8, 0.4),
                1.2,
                WOOD,
            );
            for x in [-half.x * 0.5, half.x * 0.5] {
                model.block(Vec2::new(x, half.y * 0.5), Vec2::splat(0.5), 1.6, RED_ROOF);
            }
        }
        StoneTower => {
            model.prism(at, &shape, (0.0, height), STONE);
            model.battlements(at, &shape, height, STONE);
            model.doorway(at.x, half.y, 0.4, 1.3);
        }
        WoodenTower => walls_and_roof(model, WOOD, THATCH, rise),
        Farm => walls_and_roof(model, PLASTER, THATCH, rise),
        HerbalistHut => walls_and_roof(model, WOOD, MOSS, rise),
        Castle | Fort => {
            let (colour, wall_height) = if kind == Castle {
                (STONE, 2.6)
            } else {
                (WOOD, 1.8)
            };
            model.prism(at, &shape, (0.0, wall_height), colour);
            model.battlements(at, &shape, wall_height, colour);
            model.doorway(at.x, half.y, 0.7, 1.6);
            let keep = ring(outline, half * 0.5);
            model.prism(at, &keep, (wall_height, wall_height + height), DARK_STONE);
            model.battlements(at, &keep, wall_height + height, DARK_STONE);
        }
        StonePyramid | SandPyramid => {
            let colour = if kind == StonePyramid { STONE } else { SAND };
            let steps = floors.max(3);
            let step = half.min_element() / f32::from(steps);
            for i in 0..steps {
                let side = half.min_element() - step * f32::from(i);
                let from = if i == 0 { 0.0 } else { f32::from(i) * 1.1 };
                let to = f32::from(i + 1) * 1.1;
                model.prism(at, &rectangle(Vec2::splat(side)), (from, to), colour);
            }
            model.doorway(at.x, half.min_element(), 0.5, 1.0);
        }
        // Not buildings (`Generator::for_poi`), but still something to see.
        _ => flat(model, STONE),
    }
}

/// What is left of a building: no roof, and broken walls round a bare floor,
/// with more of them gone and the rest lower the more ruined it is.
pub(super) fn ruined(
    model: &mut PoiModel,
    kind: PoiKind,
    outline: Outline,
    half: Vec2,
    floors: u8,
    ruin: u8,
    variety: u64,
) {
    use PoiKind::*;
    let walls = match kind {
        ThatchedHut | StiltHut | GreenStiltHut | Warehouse | WoodenTower | Fort | ArcheryRange
        | HerbalistHut => WOOD,
        AdobeHouse | AdobeStore => ADOBE,
        SandPyramid => SAND,
        _ => STONE,
    };
    let shape = ring(outline, half);
    model.prism(Vec2::ZERO, &shape, (0.0, 0.1), EARTH);
    let full = FLOOR * f32::from(floors.max(1));
    let harm = f32::from(ruin);
    let mut number = variety;
    for (i, &from) in shape.iter().enumerate() {
        let to = shape[(i + 1) % shape.len()];
        let stones = (from.distance(to) / WALL_STONE).ceil().max(1.0) as usize;
        for stone in 0..stones {
            let (gone, low) = (hashed(&mut number), hashed(&mut number));
            if gone < harm * 0.14 {
                continue;
            }
            let at = from.lerp(to, (stone as f32 + 0.5) / stones as f32);
            let height = (full * (1.0 - low * harm * 0.17)).max(0.4);
            model.block(at, Vec2::splat(WALL_STONE / 2.0), height, walls);
        }
    }
}
