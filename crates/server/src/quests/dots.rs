//! The maps' quest dots on the level the player is on (owner): for each quest
//! taken, where an objective that can be done now is, as far as the
//! character may know it. In a site of a step's poi, the stairs they remember
//! that lead towards the level it is done on; in the wilderness, the step's
//! poi or town; for a quest to hand in, its giver's door once in their town
//! (towns are known whole), else the town; a quest boss once in sight, a quest
//! item once where it lies has been seen. The world map needs none: the
//! client has every step's poi, made known on taking the quest.

use shared::protocol::{QuestDots, QuestNews, ServerMessage, level_chunk};
use shared::quest::{Hook, Quest, QuestId, Step, Target};

use super::boss_signs::boss_cell;
use super::{cache, done_flags};
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::Visit;

/// What the dots were last worked out from and sent as (`Seen::dots`).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct DotsSeen {
    /// The level and cell the player was at, and how far along each quest
    /// was: worked out again only when one changes.
    from: Option<DotsFrom>,
    sent: Vec<QuestDots>,
}

#[derive(Clone, Debug, PartialEq)]
struct DotsFrom {
    level: Option<(crate::storage::SiteId, u32, (i64, i64))>,
    progress: Vec<(QuestId, Vec<u32>)>,
    /// Where the quest boss in sight is, if one is.
    boss: Option<(i64, i64)>,
    /// Where escortees are, as their quests' progress keeps them, and the
    /// first on a level in use (they move).
    escorts: Vec<(QuestId, Option<crate::storage::EscortAt>)>,
    escortee: Option<shared::spot::Spot>,
}

/// Sends the dots if they have changed. `false` if the client has gone.
pub(super) fn tell_dots(client: &mut Client, world: &mut ServerWorld) -> bool {
    let from = DotsFrom {
        level: (client.visit.as_ref()).map(|v| (v.id(), v.index(), v.position())),
        progress: (client.character.quests.taken.iter())
            .map(|(id, progress)| (*id, progress.counts.clone()))
            .collect(),
        boss: boss_cell(client, world),
        escorts: (client.character.quests.taken.iter())
            .filter(|(_, progress)| progress.escort.is_some())
            .map(|(id, progress)| (*id, progress.escort))
            .collect(),
        escortee: (client.character.quests.taken.iter())
            .filter(|(_, progress)| progress.escort.is_some())
            .find_map(|(id, _)| world.escortee_at(super::escortee::escortee_id(client, *id)))
            .map(|(_, spot)| spot),
    };
    if client.quest_seen.dots.from.as_ref() == Some(&from) {
        return true;
    }
    client.quest_seen.dots.from = Some(from);
    let dots = dots_here(client, world);
    if dots == client.quest_seen.dots.sent {
        return true;
    }
    client.quest_seen.dots.sent = dots.clone();
    let news = QuestNews::Dots(dots);
    client.link.send(&ServerMessage::Quest(news))
}

/// Each quest taken with dots on the level the player is on.
pub(crate) fn dots_here(client: &Client, world: &mut ServerWorld) -> Vec<QuestDots> {
    let Some(visit) = &client.visit else {
        return Vec::new();
    };
    let mut all = Vec::new();
    for (&id, progress) in &client.character.quests.taken {
        let Some(quest) = cache::quest(world, id) else {
            continue;
        };
        let done = done_flags(&quest, &progress.counts);
        let mut cells = if quest.ready_to_turn_in(&done) {
            return_cells(visit, world, &quest)
        } else {
            let open = quest.open_steps(&done).into_iter();
            open.flat_map(|i| step_cells(client, visit, world, &quest, &quest.steps[i]))
                .collect()
        };
        cells.sort_unstable();
        cells.dedup();
        if !cells.is_empty() {
            all.push(QuestDots { id, cells });
        }
    }
    all
}

/// Handing in: the giver's door in their town, else the town in the
/// wilderness.
pub(super) fn return_cells(
    visit: &Visit,
    world: &mut ServerWorld,
    quest: &Quest,
) -> Vec<(i64, i64)> {
    let town = quest.id.town;
    if let Some(door) = visit.giver_door(world, town, quest.building) {
        return vec![door];
    }
    let poi = world.pois.in_chunk(town);
    poi.and_then(|poi| visit.wild_cell_of(&poi))
        .into_iter()
        .collect()
}

/// One step that can be done now.
fn step_cells(
    client: &Client,
    visit: &Visit,
    world: &mut ServerWorld,
    quest: &Quest,
    step: &Step,
) -> Vec<(i64, i64)> {
    let (chunk, hook) = match step.target {
        Target::Poi { chunk, hook } => (chunk, Some(hook)),
        Target::Town { chunk } => (chunk, None),
        // Anywhere round the town: nowhere to point.
        Target::Wilds { .. } => return Vec::new(),
        Target::Dungeon { hook, .. } => {
            return super::dungeon_dots::dungeon_cells(client, visit, world, quest, hook);
        }
    };
    // Someone escorted, once met (`escort_dots`).
    if hook == Some(Hook::Escortee)
        && let Some(cells) = super::escort_dots::escort_cells(client, visit, world, quest)
    {
        return cells;
    }
    if visit.is_wilderness() {
        let poi = world.pois.in_chunk(chunk);
        return poi
            .and_then(|poi| visit.wild_cell_of(&poi))
            .into_iter()
            .collect();
    }
    let in_it = visit.poi().is_some_and(|poi| poi.chunk == chunk);
    // The boss itself, once in sight.
    if in_it
        && hook == Some(Hook::Boss)
        && let Some(cell) = boss_cell(client, world)
    {
        return vec![cell];
    }
    let memory = client.character.memory.get(&visit.id());
    let remembered = memory.and_then(|levels| levels.get(&visit.index()));
    let seen = |(x, y): (i64, i64)| {
        let (chunk, bit) = level_chunk(x, y);
        remembered.is_some_and(|masks| masks.get(&chunk).is_some_and(|m| m & (1 << bit) != 0))
    };
    // Their own quest item, once where it lies has been seen.
    if in_it
        && hook == Some(Hook::Item)
        && let Some(find) = super::finds::find_here(client, world)
        && seen(find.at.cell())
    {
        return vec![find.at.cell()];
    }
    let goal = hook.and_then(|hook| goal_level(visit, hook));
    match goal {
        Some(goal) if in_it => visit.stairs_towards(goal, seen),
        _ => Vec::new(),
    }
}

/// The level of its site a hook's step is done on, when it is one level:
/// reaching the deepest, or the boss or quest item there
/// (`entities::quest_mobs`, `finds`); counted kills may be on any.
pub(super) fn goal_level(visit: &Visit, hook: Hook) -> Option<u32> {
    let levels = visit.site()?.levels();
    match hook {
        Hook::Explore | Hook::Boss | Hook::Item | Hook::Escortee => Some(levels.saturating_sub(1)),
        _ => None,
    }
}
