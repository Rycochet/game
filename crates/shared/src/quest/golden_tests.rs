//! A golden of one town's quests: if this changes, every town's quests
//! change, which needs a recorded decision (players' quest logs name quests
//! by their ids, and the server makes them again from the seed).

use super::test_world::{TestWorld, buildings};
use super::*;

/// What decides a quest, without its words.
fn fingerprint(quest: &Quest) -> String {
    let steps: Vec<String> = (quest.steps.iter())
        .map(|step| {
            format!(
                "{:?} {:?} {:?} {:?}",
                step.goal, step.target, step.count, step.after
            )
        })
        .collect();
    format!(
        "{:?} {} {:?} {:?} {:?}",
        quest.id.slot, quest.member, steps, quest.reward, quest.story.reason
    )
}

#[test]
fn golden_quests() {
    let world = TestWorld::new();
    let town = world.towns(24)[0];
    let quests = town_quests(&world.view(), &town, &buildings(&town));
    let found: Vec<String> = quests.iter().take(3).map(fingerprint).collect();
    assert_eq!(found, GOLDEN, "{town:?}");
}

/// Changed 2026-11-13 (quests PR 8a, quest dungeons): a roll for a cellar
/// of the giver's own comes before each quest's hook, so towns' quests move.
const GOLDEN: &[&str] = &[
    "Giver { building: 0, slot: 0, chain: 0 } 0 [\"Slay Poi { chunk: ChunkPos { x: 9, y: 1 }, hook: Kills(Lizardfolk) } Some(4) []\"] [Coins(40)] Rumour",
    "Giver { building: 0, slot: 0, chain: 1 } 0 [\"Visit Poi { chunk: ChunkPos { x: 11, y: 1 }, hook: Visit } None []\"] [Coins(35)] Rumour",
    "Giver { building: 1, slot: 0, chain: 0 } 0 [\"Explore Poi { chunk: ChunkPos { x: 11, y: 0 }, hook: Explore } None []\", \"Find Poi { chunk: ChunkPos { x: 11, y: 0 }, hook: Item } None [0]\"] [Coins(75), Training] Family",
];
