//! A quest given up or failed: its progress is lost and it can be taken again
//! an hour later (owner). Quest items vanishing with it come with them (Quest
//! system PR 7).

use bevy::prelude::*;
use shared::quest::QuestId;

use super::{log, tell_markers};
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::Ended;

/// Ends a quest taken, given up or `failed`, at `now`. Nothing if it isn't
/// taken. `false` if the client has gone.
pub(crate) fn end(
    client: &mut Client,
    world: &mut ServerWorld,
    id: QuestId,
    failed: bool,
    now: i64,
) -> bool {
    let book = &mut client.character.quests;
    if book.taken.remove(&id).is_none() {
        warn!("{} gave up a quest they hadn't taken", client.in_log());
        return true;
    }
    book.ended.insert(id, Ended { at: now, failed });
    client.unsaved = true;
    info!(
        "{} {} quest {id:?}",
        client.in_log(),
        if failed { "failed" } else { "gave up" }
    );
    log::tell_changed(client, world, id) && tell_markers(client, world, id.town, now)
}
