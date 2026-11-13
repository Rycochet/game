//! Taking a quest: from beside its giver's door, if it is on offer and the
//! character has room on this planet's list; its targets become known and
//! named (owner).

use bevy::prelude::*;
use shared::protocol::{QuestNews, QuestRefusal, ServerMessage};
use shared::quest::{Quest, QuestId};

use super::{MOST_TAKEN, log, offers, tell_markers};
use crate::ServerWorld;
use crate::connections::Client;
use crate::ids::new_id;
use crate::storage::Progress;
use crate::world_map::learn_place;

/// Takes `id`, if the player may (`reach`: they must stand at the giver's
/// door; the debug API leaves that out). `false` if the client has gone.
pub(crate) fn accept(
    client: &mut Client,
    world: &mut ServerWorld,
    id: QuestId,
    now: i64,
    reach: bool,
) -> bool {
    let Some(building) = giver_of(world, id) else {
        warn!(
            "{} asked for a quest that doesn't exist: {id:?}",
            client.in_log()
        );
        return true;
    };
    if reach && !crate::sites::beside_giver(client, world, id.town, building) {
        warn!("{} asked for a quest away from its giver", client.in_log());
        return true;
    }
    let Some(quest) = offers::on_offer(client, world, id, building, now) else {
        return refuse(client, id, QuestRefusal::NotOffered);
    };
    if client.character.quests.taken.len() >= MOST_TAKEN {
        return refuse(client, id, QuestRefusal::TooMany);
    }
    // A delivery's parcels, all or none (owner).
    let mut inventory = client.inventory.clone();
    let parcels = super::parcels::parcels_for(world, &quest);
    let handed = !parcels.is_empty();
    for parcel in parcels {
        if inventory.give(parcel, world.stacks, &mut new_id).is_err() {
            return refuse(client, id, QuestRefusal::NoRoom);
        }
    }
    client.inventory = inventory;
    let counts = vec![0; quest.steps.len()];
    client.character.quests.taken.insert(
        id,
        Progress {
            at: now,
            counts,
            ready: false,
        },
    );
    client.unsaved = true;
    info!("{} took quest {id:?}", client.in_log());
    if handed {
        client.update_speed(world.action_clock.ticks());
        if !client.send_inventory() {
            return false;
        }
    }
    let ahead = super::ahead::already_done(client, &quest);
    (!ahead || super::ahead::tell_ahead(client, world))
        && learn_targets(client, world, &quest)
        && log::tell_changed(client, world, id)
        && tell_markers(client, world, id.town, now)
        // A cellar of the giver's own: straight down into it (owner).
        && (quest.dungeon.is_none() || super::dungeon::enter(client, world, id, reach))
}

/// The building of a quest's giver.
pub(crate) fn giver_of(world: &mut ServerWorld, id: QuestId) -> Option<u16> {
    super::cache::quest(world, id).map(|quest| quest.building)
}

/// Every poi a quest sends the player to is known and named from now on.
fn learn_targets(client: &mut Client, world: &ServerWorld, quest: &Quest) -> bool {
    let mut chunks: Vec<_> = quest.steps.iter().map(|step| step.target.chunk()).collect();
    chunks.sort_unstable();
    chunks.dedup();
    chunks
        .into_iter()
        .all(|chunk| learn_place(client, world, chunk))
}

/// Tells the client why something asked of `id` wasn't done.
pub(crate) fn refuse(client: &mut Client, id: QuestId, why: QuestRefusal) -> bool {
    client
        .link
        .send(&ServerMessage::Quest(QuestNews::Refused { id, why }))
}
