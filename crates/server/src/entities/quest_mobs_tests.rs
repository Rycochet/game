//! A quest boss is its kind made better by its tier: its story's gender, and
//! two to every attribute for each step above Common.

use shared::item::Quality;
use shared::mob::chosen_spawns;
use shared::poi::{CHUNK_SIZE, Poi};
use shared::quest::{boss_of, boss_tier};
use shared::sheet::Attribute;

use crate::connections::tests::TestClient;
use crate::sites::test_enter_poi;
use crate::storage::SiteId;

/// A poi near the start whose boss a town's quest asks for.
fn a_boss_poi(test: &mut TestClient) -> Poi {
    let (x, y) = test.client.world_position();
    let reach = 16 * CHUNK_SIZE;
    let mut pois = (test.world.pois).in_area((x - reach, y - reach), (x + reach, y + reach));
    pois.sort_by_key(|poi| poi.chunk);
    (pois.into_iter())
        .find(|poi| crate::quests::boss_quest(&mut test.world, poi.chunk).is_some())
        .expect("a poi with a quest boss near the start")
}

#[test]
fn a_boss_is_stronger_by_its_tier_and_has_its_storys_gender() {
    let mut test = TestClient::new();
    let poi = a_boss_poi(&mut test);
    assert!(test_enter_poi(&mut test.client, &mut test.world, poi, 0));
    let levels = test.client.visit.as_ref().unwrap().site().unwrap().levels();
    assert!(test_enter_poi(
        &mut test.client,
        &mut test.world,
        poi,
        levels - 1
    ));
    let place = (SiteId::Poi(poi.chunk), levels - 1);
    let population = test.world.populations.0.get(&place).unwrap();
    let found = population.mobs.iter().find(|mob| mob.is_boss());
    let boss = found.unwrap_or_else(|| panic!("no boss at {poi:?}, on {levels} levels"));
    let story = boss_of(&poi);
    assert_eq!(boss.gender, story.gender);

    // As rolled, before its tier.
    let seed = poi.seed.derive("quest boss").derive("spawn");
    let (level, stairs) = (&population.level, &population.stairs);
    let rolled = (chosen_spawns(seed, story.kind, 1, level, stairs).pop())
        .or_else(|| chosen_spawns(seed, story.kind, 1, level, &[]).pop())
        .unwrap();
    let steps = boss_tier(&poi) as u16 - Quality::Common as u16;
    assert!(steps >= 1);
    for which in Attribute::ALL {
        let base = rolled.attributes.get(which) - rolled.gender.bonus().get(which)
            + story.gender.bonus().get(which);
        assert_eq!(boss.attributes.get(which), base + 2 * steps, "{which:?}");
    }
}
