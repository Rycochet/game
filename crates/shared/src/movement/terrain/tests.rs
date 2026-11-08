use super::*;
use crate::world::{WorldParams, WorldSeed};

#[test]
fn every_terrain_has_a_pioneering_skill() {
    for terrain in Terrain::ALL {
        assert_eq!(terrain.skill().name().to_lowercase(), terrain.key());
        assert_eq!(terrain.skill().parent().unwrap().key(), "pioneering");
        assert!(terrain.per_mille() > 1000);
    }
}

#[test]
fn roads_and_water_beat_terrain() {
    let on = |road, wet| {
        Footing {
            road,
            wet,
            terrain: Some(Terrain::Desert),
        }
        .slowed_by()
    };
    assert_eq!(on(None, false), Some(Terrain::Desert));
    assert_eq!(on(Some(crate::road::RoadKind::Path), false), None);
    assert_eq!(on(None, true), None);
}

#[test]
fn land_has_a_terrain_and_open_water_none() {
    let world = WorldParams::from_seed(WorldSeed::new(12345).terrain());
    let ground = WorldGround::new(&world);
    let conditions = WorldConditions {
        water: world.high_water(),
        ice: true,
    };
    let (mut land, mut sea) = (0, 0);
    for y in (-1500..1500).step_by(64) {
        for x in (-1500..1500).step_by(64) {
            match (
                ground.elevation(x, y, conditions),
                ground.terrain(x, y, conditions),
            ) {
                (Some(_), Some(_)) => land += 1,
                (None, None) => sea += 1,
                other => panic!("({x}, {y}): {other:?}"),
            }
        }
    }
    assert!(land > 0 && sea > 0);
}
