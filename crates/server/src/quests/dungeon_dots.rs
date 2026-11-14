//! Dots for a step done in a quest's own dungeon: from outside it, the giver's
//! door it is entered by (a hidden one's entrance, from the wilderness); inside, for exploring it, the stairs the character
//! remembers leading on towards its deepest level (counted kills have none).

use shared::protocol::level_chunk;
use shared::quest::{DungeonWay, Hook, Quest};

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
    match quest.dungeon {
        _ if inside => {}
        // A hidden entrance: where it is, when the wilderness about it is in view.
        Some(dungeon) if dungeon.way == DungeonWay::Hidden => {
            return visit.wild_cell_of(&dungeon.poi).into_iter().collect();
        }
        _ => return return_cells(visit, world, quest),
    }
    let memory = client.character.memory.get(&visit.id());
    let remembered = memory.and_then(|levels| levels.get(&visit.index()));
    let seen = |(x, y): (i64, i64)| {
        let (chunk, bit) = level_chunk(x, y);
        remembered.is_some_and(|masks| masks.get(&chunk).is_some_and(|m| m & (1 << bit) != 0))
    };
    // A locked stage door stands before the way down.
    let stage = super::stage_door::locked_stage(client, world);
    let mut cells: Vec<(i64, i64)> = stage.map(|(at, _)| at).into_iter().collect();
    if let Some(goal) = goal_level(visit, hook) {
        cells.extend(visit.stairs_towards(goal, seen));
    }
    cells
}
