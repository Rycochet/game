//! A "Slay" step's extras: on coming to a level of its site, enough of its
//! kind for what the quester still needs, never kept.

use shared::poi::{CHUNK_SIZE, ChunkPos, Poi};
use shared::quest::{Hook, Quest, Target};

use super::arrivals::{kills_wanted, quest_mobs};
use super::cache;
use super::tests::NOW;
use crate::connections::tests::TestClient;
use crate::sites::test_enter_poi;
use crate::storage::{Progress, SiteId};

/// A quest near the start with kills to count at a poi, the step, and the poi.
fn a_slay_quest(test: &mut TestClient) -> (Quest, usize, Poi) {
    let (x, y) = test.client.world_position();
    let reach = 16 * CHUNK_SIZE;
    let mut towns: Vec<ChunkPos> = (test.world.pois)
        .in_area((x - reach, y - reach), (x + reach, y + reach))
        .into_iter()
        .filter(|poi| poi.kind.is_town())
        .map(|poi| poi.chunk)
        .collect();
    towns.sort_unstable();
    for chunk in towns {
        let town = cache::town(&mut test.world, chunk).expect("a town");
        for quest in town.quests.iter() {
            let kills = quest.steps.iter().position(|step| {
                matches!(
                    step.target,
                    Target::Poi {
                        hook: Hook::Kills(_),
                        ..
                    }
                ) && step.after.is_empty()
            });
            if let Some(step) = kills {
                let at = quest.steps[step].target.chunk();
                let poi = test.world.pois.in_chunk(at).expect("its poi");
                return (quest.clone(), step, poi);
            }
        }
    }
    panic!("no quest near the start with kills to count");
}

#[test]
fn a_slay_step_brings_extras_of_its_kind_until_there_are_enough() {
    let mut test = TestClient::new();
    let (quest, step, poi) = a_slay_quest(&mut test);
    let Target::Poi {
        hook: Hook::Kills(kind),
        ..
    } = quest.steps[step].target
    else {
        unreachable!()
    };
    let progress = Progress {
        at: NOW,
        counts: vec![0; quest.steps.len()],
        ready: false,
        escort: None,
    };
    let book = &mut test.client.character.quests;
    book.taken.insert(quest.id, progress);
    let wanted = kills_wanted(&test.client, &mut test.world, poi.chunk);
    let needed = quest.steps[step].count.unwrap();
    assert_eq!(wanted, vec![(kind, needed)]);

    assert!(test_enter_poi(&mut test.client, &mut test.world, poi, 0));
    quest_mobs(&test.client, &mut test.world);
    let place = (SiteId::Poi(poi.chunk), 0);
    let population = test.world.populations.0.get(&place).unwrap();
    let of_kind = population.mobs.iter().filter(|mob| mob.kind == kind);
    let extras: Vec<_> = of_kind.clone().filter(|mob| !mob.is_kept()).collect();
    assert!(of_kind.count() as u32 >= needed.min(8), "enough to slay");
    assert!(extras.iter().all(|mob| mob.kind == kind));

    // Coming again with them still about brings no more.
    let before = population.mobs.len();
    quest_mobs(&test.client, &mut test.world);
    let population = test.world.populations.0.get(&place).unwrap();
    assert_eq!(population.mobs.len(), before);
}
