//! Dots for a step done in a quest's own dungeon: from outside it, the giver's
//! door it is entered by; inside, for exploring it, the stairs the character
//! remembers leading on towards its deepest level (counted kills have none).

use shared::protocol::level_chunk;
use shared::quest::{Hook, Quest};

use super::dots::{goal_level, return_cells};
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::Visit;

pub(super) fn dungeon_cells(
    client: &Client,
    visit: &Visit,
    world: &mut ServerWorld,
    quest: &Quest,
    hook: Hook,
) -> Vec<(i64, i64)> {
    let inside = super::dungeon::site_of(client, quest.id) == Some(visit.id());
    if !inside {
        return return_cells(visit, world, quest);
    }
    let memory = client.character.memory.get(&visit.id());
    let remembered = memory.and_then(|levels| levels.get(&visit.index()));
    let seen = |(x, y): (i64, i64)| {
        let (chunk, bit) = level_chunk(x, y);
        remembered.is_some_and(|masks| masks.get(&chunk).is_some_and(|m| m & (1 << bit) != 0))
    };
    match goal_level(visit, hook) {
        Some(goal) => visit.stairs_towards(goal, seen),
        None => Vec::new(),
    }
}
