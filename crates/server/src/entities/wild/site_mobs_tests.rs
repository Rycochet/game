use shared::poi::{ChunkPos, Poi, PoiKind};
use shared::protocol::ClientMessage;

use super::*;
use crate::connections::tests::TestClient;

/// A client in the wilderness at a farm near the start, with the areas
/// worked out, and the farm.
fn at_a_farm() -> (TestClient, Poi) {
    let world = crate::ServerWorld::new(shared::world::WorldSeed::new(12345));
    let mut test = TestClient::of(world);
    let (x, y) = test.client.world_position();
    let start = test.world.pois.chunk_at(x, y);
    let farm = (1..16)
        .flat_map(|ring| (-ring..=ring).flat_map(move |dy| (-ring..=ring).map(move |dx| (dx, dy))))
        .filter_map(|(dx, dy)| {
            let chunk = ChunkPos {
                x: start.x + dx,
                y: start.y + dy,
            };
            test.world.pois.in_chunk(chunk)
        })
        .find(|poi| poi.kind == PoiKind::Farm)
        .expect("a farm near the start");
    test.client.character.world_position = (farm.x, farm.y);
    test.send(&[ClientMessage::Down]);
    keep_wild_areas(std::slice::from_mut(&mut test.client), &mut test.world);
    (test, farm)
}

/// The farm's own mobs about in the player's area.
fn farm_mobs(test: &TestClient, farm: &Poi) -> Vec<Mob> {
    let visit = test.client.visit.as_ref().expect("in the wilderness");
    let population = &test.world.populations.0[&visit.population()];
    let id = SiteId::Poi(farm.chunk);
    let mobs = population.mobs.iter().filter(|mob| mob.kept_in == Some(id));
    mobs.cloned().collect()
}

#[test]
fn a_site_set_in_has_its_own_mobs_about_in_the_wilderness() {
    let (test, farm) = at_a_farm();
    let mobs = farm_mobs(&test, &farm);
    assert!(!mobs.is_empty(), "a farm has animals");
    // The same ids its own visit gives them, kept with the farm.
    let saved = &test.world.sites[&SiteId::Poi(farm.chunk)].levels[&0];
    for mob in &mobs {
        assert_eq!(saved.mobs.get(&mob.number), Some(&mob.id));
        assert!(saved.records.contains_key(&mob.id));
    }
}

#[test]
fn a_site_mob_killed_in_the_wilderness_stays_dead_in_its_site() {
    let (mut test, farm) = at_a_farm();
    let id = SiteId::Poi(farm.chunk);
    let mobs = farm_mobs(&test, &farm);
    let (dead, alive) = (&mobs[0], &mobs[1..]);
    let place = test.client.visit.as_ref().unwrap().population();
    super::super::gear::bury(&mut test.world, place, 0, dead.id, true);
    let saved = &test.world.sites[&id].levels[&0];
    assert!(saved.dead.contains(&dead.number), "kept with the farm");
    let wild = test.world.site(SiteId::Wilderness).levels.get(&0);
    assert!(wild.is_none_or(|level| !level.dead.contains(&dead.number)));

    // Out to the world map and into the farm: the others come home to it,
    // and nobody is in two places.
    test.send(&[ClientMessage::Up]);
    keep_wild_areas(std::slice::from_mut(&mut test.client), &mut test.world);
    test.send(&[ClientMessage::EnterSite { chunk: farm.chunk }]);
    let visit = test.client.visit.as_ref().expect("in the farm");
    assert_eq!(visit.population(), (id, 0));
    let numbers: Vec<u32> = test.world.populations.0[&(id, 0)]
        .mobs
        .iter()
        .map(|mob| mob.number)
        .collect();
    assert!(!numbers.contains(&dead.number), "still dead");
    for mob in alive {
        assert!(numbers.contains(&mob.number), "home again");
    }
    let kept = test.world.wilderness.areas.kept.values();
    assert!(
        kept.flat_map(|(mobs, _)| mobs)
            .all(|mob| mob.kept_in != Some(id))
    );
}
