//! Quests' own dungeons made afresh about an hour after the character came
//! out (owner), the progress made inside them lost, and copies forgotten.

use bevy::prelude::*;
use shared::quest::{QuestId, Target};

use super::dungeon::{RESET_AFTER, site_of};
use super::{cache, done_flags, log};
use crate::ServerWorld;
use crate::connections::Client;

/// Every quest dungeon of the character's left more than `RESET_AFTER` ago
/// is made afresh: forgotten, with what was done inside. Steps already
/// finished stay done (Claude). `false` if the client has gone.
pub(crate) fn reset_due(client: &mut Client, world: &mut ServerWorld, now: i64) -> bool {
    let due: Vec<QuestId> = (client.character.quests.dungeons.iter())
        .filter(|(_, stay)| stay.left.is_some_and(|left| now - left >= RESET_AFTER))
        .map(|(&id, _)| id)
        .collect();
    let mut sent = true;
    for id in due {
        if reset(client, world, id) {
            sent &= log::tell_changed(client, world, id);
        }
    }
    sent
}

pub(super) fn reset_if_due(client: &mut Client, world: &mut ServerWorld, id: QuestId, now: i64) {
    let stay = client.character.quests.dungeons.get(&id);
    if stay
        .and_then(|stay| stay.left)
        .is_some_and(|left| now - left >= RESET_AFTER)
    {
        reset(client, world, id);
    }
}

/// Quest `id`'s dungeon made afresh for this character; whether its progress
/// changed.
fn reset(client: &mut Client, world: &mut ServerWorld, id: QuestId) -> bool {
    info!("{}'s copy of {id:?}'s dungeon resets", client.in_log());
    forget_copy(client, world, id);
    // Kept: a hidden entrance stays on their map while the quest is taken.
    if let Some(stay) = client.character.quests.dungeons.get_mut(&id) {
        stay.left = None;
        stay.back = None;
    }
    let Some(quest) = cache::quest(world, id) else {
        return false;
    };
    let Some(progress) = client.character.quests.taken.get_mut(&id) else {
        return false;
    };
    progress.counts.resize(quest.steps.len(), 0);
    let mut changed = false;
    for (step, count) in quest.steps.iter().zip(progress.counts.iter_mut()) {
        let finished = *count >= step.count.unwrap_or(1);
        if matches!(step.target, Target::Dungeon { .. }) && !finished && *count > 0 {
            *count = 0;
            changed = true;
        }
    }
    progress.ready = quest.ready_to_turn_in(&done_flags(&quest, &progress.counts));
    changed
}

/// The character's copy of quest `id`'s dungeon is forgotten, as is what
/// they remember of it: on a reset, and when the quest ends.
pub(super) fn forget_copy(client: &mut Client, world: &mut ServerWorld, id: QuestId) {
    client.unsaved = true;
    if let Some(site) = site_of(client, id) {
        world.forget_site(site);
        client.character.forget_site(&site);
    }
}
