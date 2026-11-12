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

    // Out to the world map and into the farm again, by a click: in the
    // wilderness, with the others back and the dead one still gone.
    test.send(&[ClientMessage::Up]);
    keep_wild_areas(std::slice::from_mut(&mut test.client), &mut test.world);
    test.client.character.world_position = (farm.x, farm.y);
    test.send(&[ClientMessage::EnterSite { chunk: farm.chunk }]);
    keep_wild_areas(std::slice::from_mut(&mut test.client), &mut test.world);
    let numbers: Vec<u32> = farm_mobs(&test, &farm).iter().map(|m| m.number).collect();
    assert!(!numbers.contains(&dead.number), "still dead");
    for mob in alive {
        assert!(numbers.contains(&mob.number), "about again");
    }
}

#[test]
fn a_site_visit_takes_back_its_mobs_resting_in_the_wilderness() {
    let (mut test, farm) = at_a_farm();
    let id = SiteId::Poi(farm.chunk);
    test.send(&[ClientMessage::Up]);
    keep_wild_areas(std::slice::from_mut(&mut test.client), &mut test.world);
    let kept = |test: &TestClient| {
        let kept = test.world.wilderness.areas.kept.values();
        kept.flat_map(|(mobs, _)| mobs)
            .filter(|mob| mob.kept_in == Some(id))
            .count()
    };
    assert!(kept(&test) > 0, "resting where they were");
    let about = take_site_mobs_back(&mut test.world, id);
    assert!(about.is_empty(), "nobody is about");
    assert_eq!(kept(&test), 0, "home to the farm's own visit");
}

/// A client at the nearest town to the start, on the world map at its poi.
fn near_a_town() -> (TestClient, Poi) {
    let world = crate::ServerWorld::new(shared::world::WorldSeed::new(12345));
    let mut test = TestClient::of(world);
    let (x, y) = test.client.world_position();
    let start = test.world.pois.chunk_at(x, y);
    let town = (1..24)
        .flat_map(|ring| (-ring..=ring).flat_map(move |dy| (-ring..=ring).map(move |dx| (dx, dy))))
        .filter_map(|(dx, dy)| {
            let chunk = ChunkPos {
                x: start.x + dx,
                y: start.y + dy,
            };
            test.world.pois.in_chunk(chunk)
        })
        .find(|poi| poi.kind.is_town())
        .expect("a town near the start");
    test.client.character.world_position = (town.x, town.y);
    (test, town)
}

#[test]
fn arriving_inside_a_town_finds_its_people_about() {
    let (mut test, town) = near_a_town();
    let id = SiteId::Poi(town.chunk);
    // In by the entrance, then back out from inside, by Up where they stand
    // (as joining there, or leaving and exploring again on the spot does).
    test.send(&[ClientMessage::Down]);
    keep_wild_areas(std::slice::from_mut(&mut test.client), &mut test.world);
    assert!(!farm_mobs(&test, &town).is_empty(), "a town has people");
    let wild = &test.world.wilderness;
    let chunk = wild.chunk(town.chunk).expect("worked out");
    let (ox, oy) = chunk
        .plan
        .cutout()
        .and_then(|c| c.offset())
        .expect("set in");
    let inside = chunk.spawns[0].position;
    test.send(&[ClientMessage::Up]);
    keep_wild_areas(std::slice::from_mut(&mut test.client), &mut test.world);
    test.client
        .character
        .left_at
        .insert(id, (0, (inside.0 - ox, inside.1 - oy)));
    test.client.character.world_position = (town.x, town.y);
    test.send(&[ClientMessage::Down]);
    let at = test.client.visit.as_ref().unwrap().position();
    assert_ne!(at, chunk_entrance(&test, &town), "arrived inside");
    keep_wild_areas(std::slice::from_mut(&mut test.client), &mut test.world);
    assert!(!farm_mobs(&test, &town).is_empty(), "its people are about");
}

/// Where going into a poi arrives when nobody has been in it.
fn chunk_entrance(test: &TestClient, poi: &Poi) -> (i64, i64) {
    let chunk = test.world.wilderness.chunk(poi.chunk).expect("worked out");
    chunk.entrance.expect("an entrance")
}
