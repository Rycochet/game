//! A quest boss killed (owner): its step done for every quester on it who
//! shared in the kill; anyone else who shared in it, without the quest, has
//! it remembered (`QuestBook::slain`), and the giver's town is known and
//! named, marked on the maps, its giver offering the quest with that step
//! done ("I've already dealt with it").

use bevy::prelude::*;
use shared::poi::ChunkPos;
use shared::protocol::{QuestNews, ServerMessage, SlainBoss};

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
        && tell_slain(client, world)
        && tell_markers(client, world, boss.id.town, now)
}

/// The quests whose boss the character has dealt with, not taken yet.
pub(super) fn slain_news(client: &Client, world: &mut ServerWorld) -> QuestNews {
    let slain = (client.character.quests.slain.iter())
        .filter_map(|&id| {
            let quest = cache::quest(world, id)?;
            Some(SlainBoss {
                id,
                kind: quest.kind,
            })
        })
        .collect();
    QuestNews::Slain(slain)
}

/// Tells the client which bosses it has dealt with. `false` if it has gone.
pub(super) fn tell_slain(client: &mut Client, world: &mut ServerWorld) -> bool {
    let news = slain_news(client, world);
    client.link.send(&ServerMessage::Quest(news))
}

/// On taking a quest whose boss the character has already dealt with: that
/// step is done. Whether it was.
pub(super) fn already_dealt(client: &mut Client, quest: &shared::quest::Quest) -> bool {
    if !client.character.quests.slain.remove(&quest.id) {
        return false;
    }
    let boss = quest.steps.iter().position(|step| {
        matches!(
            step.target,
            shared::quest::Target::Poi {
                hook: shared::quest::Hook::Boss,
                ..
            }
        )
    });
    if let (Some(step), Some(progress)) = (boss, client.character.quests.taken.get_mut(&quest.id)) {
        progress.counts.resize(quest.steps.len(), 0);
        progress.counts[step] = 1;
        progress.ready = quest.ready_to_turn_in(&done_flags(quest, &progress.counts));
    }
    true
}
