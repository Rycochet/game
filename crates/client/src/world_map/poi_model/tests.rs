use bevy::prelude::*;
use shared::poi::PoiKind;
use shared::rng::Seed;
use shared::site::{Outline, PoiLook};

use super::*;

fn looks() -> impl Iterator<Item = (PoiKind, PoiLook)> {
    PoiKind::ALL
        .into_iter()
        .flat_map(|kind| (0..8).map(move |seed| (kind, PoiLook::of(kind, Seed::new(seed), false))))
}

#[test]
fn every_poi_has_a_model_that_fits_where_its_base_was() {
    for (kind, look) in looks() {
        let model = build(kind, look, 3);
        assert!(!model.corners.is_empty(), "{kind:?}");
        for corner in model.corners.iter().flatten() {
            let reach = corner.xz().length();
            assert!(reach <= REACH * 1.3, "{kind:?} {look:?}: {reach}");
        }
    }
}

#[test]
fn pictures_show_the_model_and_leave_the_rest_clear() {
    for (kind, look) in looks() {
        let pixels = build(kind, look, 0).picture(48);
        let solid = pixels.chunks(4).filter(|pixel| pixel[3] == 255).count();
        assert!(solid > 20, "{kind:?} {look:?}: {solid}");
        assert!(solid < 48 * 48, "{kind:?} {look:?}");
    }
}

#[test]
fn more_floors_stand_taller() {
    let building = |floors| {
        let look = PoiLook::Building {
            outline: Outline::Circle,
            width: 15,
            height: 15,
            floors,
            ruin: 0,
            wall: None,
            castle: None,
        };
        let model = build(PoiKind::StoneTower, look, 0);
        let top = model.corners.iter().flatten().map(|c| c.y);
        top.fold(f32::MIN, f32::max)
    };
    assert!(building(5) > building(2));
}

#[test]
fn bigger_towns_have_more_to_them() {
    let triangles = |buildings| {
        build(PoiKind::Town, PoiLook::Town { buildings }, 0)
            .corners
            .len()
    };
    assert!(triangles(5) < triangles(20));
    assert!(triangles(20) < triangles(40));
    assert!(triangles(40) < triangles(100));
}

#[test]
fn the_more_ruined_the_less_is_left() {
    let triangles = |ruin| {
        let look = PoiLook::Building {
            outline: Outline::Rectangle,
            width: 30,
            height: 30,
            floors: 2,
            ruin,
            wall: None,
            castle: None,
        };
        build(PoiKind::Castle, look, 1).corners.len()
    };
    assert!(triangles(5) < triangles(1));
}

/// A castle's towers and moat are drawn round its keep.
#[test]
fn castles_have_towers_and_moats() {
    let look = |castle| PoiLook::Building {
        outline: Outline::Rectangle,
        width: 40,
        height: 30,
        floors: 3,
        ruin: 0,
        wall: Some(shared::site::PoiWall {
            west: 6,
            north: 6,
            east: 6,
            south: 6,
            material: shared::site::WallMaterial::Stone,
            gates: [false, false, true, false],
        }),
        castle,
    };
    let triangles = |castle| build(PoiKind::Castle, look(castle), 0).corners.len();
    let castle = |moat| {
        Some(shared::site::PoiCastle {
            tower_floors: 3,
            moat,
        })
    };
    assert!(triangles(castle(0)) > triangles(None));
    // (Water's colour, as `castle::WATER`.)
    let water = |castle| {
        let model = build(PoiKind::Castle, look(castle), 0);
        model
            .colours
            .iter()
            .filter(|&&c| c == [58, 112, 176])
            .count()
    };
    assert_eq!(water(castle(0)), 0);
    assert!(water(castle(3)) > 0);
}
