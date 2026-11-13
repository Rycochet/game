//! A quest boss killed (owner): its step done for every quester on it who
//! shared in the kill; anyone else who shared in it, without the quest, has
//! it remembered (`QuestBook::slain`), and the giver's town is known and
//! named, marked on the maps, its giver offering the quest with that step
//! done ("I've already dealt with it").

use bevy::prelude::*;
use shared::poi::ChunkPos;

use super::bosses::{BossQuest, Regard, boss_quest, regard};
use super::{cache, done_flags, log, tell_markers};
use crate::ServerWorld;
use crate::connections::Client;
use crate::world_map::learn_place;

/// The character shared in killing the boss of the poi in `site`. `false`
/// if the client has gone.
pub(super) fn slew_boss(
    client: &mut Client,
    world: &mut ServerWorld,
    site: ChunkPos,
    now: i64,
) -> bool {
    let Some(boss) = boss_quest(world, site) else {
        return true;
    };
    match regard(world, &client.character.quests, boss) {
        Regard::OnStep => step_done(client, world, boss, now),
        Regard::Stranger => remember(client, world, boss, now),
        Regard::Dealt => true,
    }
}

/// Their own quest's step is done, whether or not the steps before it are:
/// the boss is dead either way.
fn step_done(client: &mut Client, world: &mut ServerWorld, boss: BossQuest, now: i64) -> bool {
    let Some(quest) = cache::quest(world, boss.id) else {
        return true;
    };
    let Some(progress) = client.character.quests.taken.get_mut(&boss.id) else {
        return true;
    };
    progress.counts.resize(quest.steps.len(), 0);
    progress.counts[boss.step] = 1;
    progress.ready = quest.ready_to_turn_in(&done_flags(&quest, &progress.counts));
    client.unsaved = true;
    log::tell_changed(client, world, boss.id) && tell_markers(client, world, boss.id.town, now)
}

/// Killed without the quest: remembered until it is taken.
fn remember(client: &mut Client, world: &mut ServerWorld, boss: BossQuest, now: i64) -> bool {
    client.character.quests.slain.insert(boss.id);
    client.unsaved = true;
    info!(
        "{} killed the boss of quest {:?} before taking it",
        client.in_log(),
        boss.id
    );
    learn_place(client, world, boss.id.town)
        && super::ahead::tell_ahead(client, world)
        && tell_markers(client, world, boss.id.town, now)
}
