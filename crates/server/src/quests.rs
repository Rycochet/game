//! Quests on the server (`shared::quest` makes them from each town's seed):
//! what givers offer each character, taking, giving up, failing and handing
//! in quests, steps moving on from kills and arrivals, standing in towns, and
//! what clients are told (`shared::protocol::QuestAsk`, `QuestNews`). A
//! character's quests are kept per planet in `storage::QuestBook`.

mod accept;
mod ask;
mod cache;
mod clock;
#[cfg(feature = "debug-api")]
pub(crate) mod debug_api;
mod dots;
mod follow;
mod give_up;
mod log;
mod needs;
mod offers;
mod planet_log;
mod planets;
mod progress;
mod standing;
mod turn_in;

pub(crate) use ask::answer_quest;
pub(crate) use cache::TownQuests;
pub(crate) use follow::{Seen, follow, killed};
pub(crate) use planet_log::answer_planet_logs;

use shared::poi::ChunkPos;
use shared::protocol::{QuestNews, ServerMessage};
use shared::quest::Quest;

use crate::ServerWorld;
use crate::connections::Client;

/// The most quests a character may have taken on one planet, dailies too
/// (owner: about 20).
pub(crate) const MOST_TAKEN: usize = 20;

/// Which of a quest's steps are done, from how far along each is.
pub(crate) fn done_flags(quest: &Quest, counts: &[u32]) -> Vec<bool> {
    (quest.steps.iter().enumerate())
        .map(|(i, step)| counts.get(i).copied().unwrap_or(0) >= step.count.unwrap_or(1))
        .collect()
}

/// Tells the client the markers over a town's doors, if the player is in that
/// town. `false` if it has gone.
pub(crate) fn tell_markers(
    client: &mut Client,
    world: &mut ServerWorld,
    town: ChunkPos,
    now: i64,
) -> bool {
    if client.quest_seen.town() != Some(town) {
        return true;
    }
    send_markers(client, world, town, now)
}

/// Sends the markers over a town's doors. `false` if the client has gone.
fn send_markers(client: &mut Client, world: &mut ServerWorld, town: ChunkPos, now: i64) -> bool {
    let markers = offers::town_markers(client, world, town, now);
    client
        .link
        .send(&ServerMessage::Quest(QuestNews::Markers { town, markers }))
}

#[cfg(test)]
mod dots_tests;
#[cfg(test)]
mod log_tests;
#[cfg(test)]
mod planets_tests;
#[cfg(test)]
mod tests;
