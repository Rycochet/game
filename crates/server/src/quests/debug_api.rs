//! `server.quests` and `server.quest`: reading a player's quests and a town's
//! givers, and driving quests for testing (taking, handing in and giving up
//! without walking to the door, finishing steps, failing, standing).

use bevy::prelude::*;
use serde::Deserialize;
use shared::poi::ChunkPos;
use shared::quest::QuestId;

use super::clock::unix_now;
use super::{accept, cache, give_up, log, offers, progress, standing, tell_markers, turn_in};
use crate::debug_api::{BrpResult, Value, json, params, refused};
use crate::{Connections, ServerWorlds};

#[derive(Deserialize)]
pub(crate) struct QuestsParams {
    /// Which connected player, in `server.state`'s order.
    #[serde(default)]
    player: usize,
    /// A town whose givers to list: the one the player is in if missing.
    town: Option<ChunkPos>,
}

/// `server.quests { "player": 0, "town": {"x": 3, "y": 4} }`: the player's
/// quest book (as saved, standing with tiers), and what each giver of the town
/// offers them now, with every quest's id to pass to `server.quest`.
pub(crate) fn quests(
    In(request): In<Option<Value>>,
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
) -> BrpResult {
    let QuestsParams { player, town } = params(request)?;
    let client = (connections.clients_mut().get_mut(player))
        .ok_or_else(|| refused(format!("no player {player}")))?;
    let world = worlds.of(client.world);
    let book = &client.character.quests;
    let standing: Vec<Value> = (book.standing.iter())
        .map(|(town, points)| json!({"town": town, "points": points, "tier": format!("{:?}", standing::tier(*points))}))
        .collect();
    let log = serde_json::to_value(log::whole_log(client, world)).unwrap_or_default();
    let town = town.or(client.quest_seen.town());
    let mut givers = Vec::new();
    if let Some(town) = town
        && let Some(found) = cache::town(world, town)
    {
        let mut buildings: Vec<u16> = found.quests.iter().map(|quest| quest.building).collect();
        buildings.extend(shared::quest::daily_givers(&found.poi, &found.buildings));
        buildings.sort_unstable();
        buildings.dedup();
        for building in buildings {
            for offer in offers::giver_offers(client, world, town, building, unix_now()) {
                givers.push(json!({
                    "building": building,
                    "id": offer.quest.id,
                    "kind": format!("{:?}", offer.quest.kind),
                    "state": serde_json::to_value(&offer.state).unwrap_or_default(),
                    "steps": offer.quest.steps,
                    "requirements": format!("{:?}", offer.quest.requirements),
                }));
            }
        }
    }
    Ok(json!({
        "character": client.character_id,
        "town": town,
        "book": book_json(book),
        "standing": standing,
        "log": log,
        "givers": givers,
    }))
}

#[derive(Deserialize)]
pub(crate) struct QuestParams {
    #[serde(default)]
    player: usize,
    /// `accept`, `turn_in`, `abandon`, `fail`, `step` (finish `step`, or every
    /// step if missing), or `standing` (add `points` in `town`).
    act: String,
    id: Option<QuestId>,
    step: Option<usize>,
    town: Option<ChunkPos>,
    #[serde(default)]
    points: u32,
}

/// `server.quest { "act": "accept", "id": {...} }`: drives a quest as if the
/// player were at its giver's door; answers `ok`, or why not.
pub(crate) fn quest(
    In(request): In<Option<Value>>,
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
) -> BrpResult {
    let QuestParams {
        player,
        act,
        id,
        step,
        town,
        points,
    } = params(request)?;
    let client = (connections.clients_mut().get_mut(player))
        .ok_or_else(|| refused(format!("no player {player}")))?;
    let world = worlds.of(client.world);
    let now = unix_now();
    let need_id = || id.ok_or_else(|| refused("give the quest's `id`"));
    match act.as_str() {
        "accept" => accept::accept(client, world, need_id()?, now, false),
        "turn_in" => turn_in::turn_in(client, world, need_id()?, now, false),
        "abandon" => give_up::end(client, world, need_id()?, false, now),
        "fail" => give_up::end(client, world, need_id()?, true, now),
        "step" => {
            let id = need_id()?;
            let steps = cache::quest(world, id).map_or(0, |quest| quest.steps.len());
            let chosen: Vec<usize> = step.map_or_else(|| (0..steps).collect(), |step| vec![step]);
            for step in chosen {
                if !progress::complete_step(client, world, id, step) {
                    return Err(refused("not a quest the player has taken"));
                }
            }
            log::tell_changed(client, world, id) && tell_markers(client, world, id.town, now)
        }
        "standing" => {
            let town = town.ok_or_else(|| refused("give the `town`"))?;
            standing::add(&mut client.character.quests, town, points);
            client.unsaved = true;
            tell_markers(client, world, town, now)
        }
        _ => {
            return Err(refused(
                "act: accept, turn_in, abandon, fail, step or standing",
            ));
        }
    };
    Ok(json!("ok"))
}

/// The book as saved, with its maps as lists (JSON keys must be words).
fn book_json(book: &crate::storage::QuestBook) -> Value {
    let taken: Vec<Value> = (book.taken.iter())
        .map(|(id, progress)| json!({"id": id, "at": progress.at, "counts": progress.counts}))
        .collect();
    let done: Vec<Value> = (book.done.iter())
        .map(|(id, at)| json!({"id": id, "at": at}))
        .collect();
    let ended: Vec<Value> = (book.ended.iter())
        .map(|(id, ended)| json!({"id": id, "at": ended.at, "failed": ended.failed}))
        .collect();
    let dailies: Vec<Value> = (book.dailies.iter())
        .map(|((town, building), daily)| {
            json!({"town": town, "building": building, "count": daily.count, "last": daily.last})
        })
        .collect();
    json!({"taken": taken, "done": done, "ended": ended, "dailies": dailies})
}
