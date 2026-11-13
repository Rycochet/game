//! The sign over a quest boss the player can see (owner): a red "!" for a
//! quest they don't have and haven't dealt with, the quest's colour while on
//! its step, none once dealt with. Sent as `QuestNews::Bosses` when it changes.

use shared::id::Id;
use shared::protocol::{BossMark, QuestNews, ServerMessage};

use super::bosses::{boss_quest, regard};
use crate::ServerWorld;
use crate::connections::Client;

/// Sends the signs over the bosses in sight, if they have changed. `false` if
/// the client has gone.
pub(super) fn tell_bosses(client: &mut Client, world: &mut ServerWorld) -> bool {
    let marks = bosses_seen(client, world);
    if marks == client.quest_seen.bosses {
        return true;
    }
    client.quest_seen.bosses = marks.clone();
    client
        .link
        .send(&ServerMessage::Quest(QuestNews::Bosses(marks)))
}

/// The boss on the player's level, if they can see it, and its sign for them.
fn bosses_seen(client: &Client, world: &mut ServerWorld) -> Vec<BossMark> {
    let Some((id, chunk)) = boss_in_sight(client, world) else {
        return Vec::new();
    };
    let Some(boss) = boss_quest(world, chunk) else {
        return Vec::new();
    };
    let sign = regard(world, &client.character.quests, boss).sign(boss.kind);
    sign.map(|sign| BossMark { id, sign }).into_iter().collect()
}

/// The quest boss on the player's level they have been told of, with its
/// site's poi.
pub(super) fn boss_in_sight(
    client: &Client,
    world: &ServerWorld,
) -> Option<(Id, shared::poi::ChunkPos)> {
    let visit = client.visit.as_ref()?;
    let chunk = visit.poi()?.chunk;
    let population = world.populations.0.get(&visit.population())?;
    let boss = population.mobs.iter().find(|mob| mob.is_boss())?;
    client
        .seen_entities
        .contains_key(&boss.id)
        .then_some((boss.id, chunk))
}

/// The cell of the quest boss on the player's level, while in sight: its dot.
pub(super) fn boss_cell(client: &Client, world: &ServerWorld) -> Option<(i64, i64)> {
    let (id, _) = boss_in_sight(client, world)?;
    let visit = client.visit.as_ref()?;
    let population = world.populations.0.get(&visit.population())?;
    let boss = population.mobs.iter().find(|mob| mob.id == id)?;
    Some(boss.position)
}
