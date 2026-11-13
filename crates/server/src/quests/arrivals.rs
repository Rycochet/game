//! Quest mobs on coming to a site level (owner): the poi's boss comes back if
//! it is down and the character is on its step or hasn't killed it, and a
//! "Slay" step brings extras of its kind until there are enough for what the
//! character still needs.

use shared::creature::CreatureKind;
use shared::poi::ChunkPos;
use shared::quest::{Hook, Target};

use super::bosses::{boss_quest, regard};
use super::{cache, done_flags};
use crate::ServerWorld;
use crate::connections::Client;

/// The boss and extras for the level the player has just come to.
pub(super) fn quest_mobs(client: &Client, world: &mut ServerWorld) {
    let Some(visit) = &client.visit else {
        return;
    };
    if visit.is_wilderness() {
        return;
    }
    let (Some(poi), Some(site)) = (visit.poi().copied(), visit.site()) else {
        return;
    };
    let levels = site.levels();
    let place = visit.population();
    let book = &client.character.quests;
    if let Some(boss) = boss_quest(world, poi.chunk)
        && regard(world, book, boss).brings_back()
    {
        world.bring_back_boss(place, &poi, levels);
    }
    for (kind, wanted) in kills_wanted(client, world, poi.chunk) {
        world.top_up(place, &poi, kind, wanted);
    }
}

/// How many more of each kind the character's open "Slay" and "Collect"
/// steps at the poi in `chunk` still need.
pub(super) fn kills_wanted(
    client: &Client,
    world: &mut ServerWorld,
    chunk: ChunkPos,
) -> Vec<(CreatureKind, u32)> {
    let mut wanted: Vec<(CreatureKind, u32)> = Vec::new();
    for (&id, progress) in &client.character.quests.taken {
        let Some(quest) = cache::quest(world, id) else {
            continue;
        };
        let done = done_flags(&quest, &progress.counts);
        for i in quest.open_steps(&done) {
            let step = &quest.steps[i];
            // Kills to count, or creatures to collect from.
            let (at, kind) = match step.target {
                Target::Poi {
                    chunk,
                    hook: Hook::Kills(kind) | Hook::Collect(kind),
                } => (chunk, kind),
                _ => continue,
            };
            let have = progress.counts.get(i).copied().unwrap_or(0);
            let still = step.count.unwrap_or(1).saturating_sub(have);
            if at != chunk || still == 0 {
                continue;
            }
            match wanted.iter_mut().find(|(k, _)| *k == kind) {
                // Two quests' kills of one kind both count each one slain.
                Some((_, most)) => *most = (*most).max(still),
                None => wanted.push((kind, still)),
            }
        }
    }
    wanted
}
