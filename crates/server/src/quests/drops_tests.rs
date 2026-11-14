//! Quest drops: a "Collect" step's remains, dropped only for the quester, at
//! its poi, from its kind, until the count.

use shared::item::ItemKind;
use shared::poi::{CHUNK_SIZE, ChunkPos};
use shared::quest::{Hook, Quest, Target};

use super::cache;
use super::drops::drop_remains;
use super::tests::NOW;
use crate::connections::tests::TestClient;
use crate::storage::Progress;

/// A quest near the start with something to collect, and the step.
fn a_collect_quest(test: &mut TestClient) -> (Quest, usize) {
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
            let step = quest.steps.iter().position(|step| {
                step.after.is_empty()
                    && matches!(
                        step.target,
                        Target::Poi {
                            hook: Hook::Collect(_),
                            ..
                        }
                    )
            });
            if let Some(step) = step {
                return (quest.clone(), step);
            }
        }
    }
    panic!("no quest near the start with something to collect");
}

fn remains_carried(test: &TestClient, quest: &Quest) -> u32 {
    (test.client.inventory).quest_kind(quest.id, Some(ItemKind::Remains))
}

#[test]
fn remains_drop_only_for_the_quester_from_its_kind_at_its_poi_until_the_count() {
    let mut test = TestClient::new();
    let (quest, step) = a_collect_quest(&mut test);
    let Target::Poi {
        chunk,
        hook: Hook::Collect(kind),
    } = quest.steps[step].target
    else {
        unreachable!()
    };
    // Without the quest, nothing.
    assert!(drop_remains(&mut test.client, &mut test.world, kind, chunk));
    assert_eq!(remains_carried(&test, &quest), 0);

    let progress = Progress {
        at: NOW,
        counts: vec![0; quest.steps.len()],
        ready: false,
        escort: None,
    };
    test.client
        .character
        .quests
        .taken
        .insert(quest.id, progress);
    // Elsewhere, or another kind: nothing.
    let elsewhere = ChunkPos {
        x: chunk.x + 1,
        ..chunk
    };
    drop_remains(&mut test.client, &mut test.world, kind, elsewhere);
    let other = if kind == shared::creature::CreatureKind::Wolf {
        shared::creature::CreatureKind::Kobold
    } else {
        shared::creature::CreatureKind::Wolf
    };
    drop_remains(&mut test.client, &mut test.world, other, chunk);
    assert_eq!(remains_carried(&test, &quest), 0);

    let count = quest.steps[step].count.unwrap();
    for _ in 0..count + 2 {
        drop_remains(&mut test.client, &mut test.world, kind, chunk);
    }
    assert_eq!(remains_carried(&test, &quest), count, "until the count");
    let counts = &test.client.character.quests.taken[&quest.id].counts;
    assert_eq!(counts[step], count);
    let one = (test.client.inventory.items.iter())
        .find(|placed| placed.item.kind == ItemKind::Remains)
        .unwrap();
    assert_eq!(one.item.quest, Some(quest.id));
    assert!(
        one.item
            .name
            .as_ref()
            .is_some_and(|name| name.starts_with(kind.def().name))
    );
}
