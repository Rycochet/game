//! What a giver has for a character: the next quest of each of their chains
//! not yet handed in, their daily, and the character's quests from them under
//! way; and the marker over each giver's door that follows from it.

use std::collections::BTreeSet;

use shared::poi::ChunkPos;
use shared::protocol::{Marker, OfferState};
use shared::quest::{Quest, QuestId, SECONDS_PER_DAY, Slot, daily_givers, utc_day};

use super::needs::{Verdict, judge};
use super::{cache, done_flags};
use crate::ServerWorld;
use crate::connections::Client;

/// One quest a giver talks about, and where it stands for the character.
pub(crate) struct Offered {
    pub(crate) quest: Quest,
    pub(crate) state: OfferState,
}

/// Everything building `building` of the town in `town` has for `client`'s
/// character at `now` (Unix seconds).
pub(crate) fn giver_offers(
    client: &Client,
    world: &mut ServerWorld,
    town: ChunkPos,
    building: u16,
    now: i64,
) -> Vec<Offered> {
    let Some(found) = cache::town(world, town) else {
        return Vec::new();
    };
    let book = &client.character.quests;
    let mut offers = Vec::new();
    // Each chain (or the feeder) offers its first quest not handed in.
    let mut chains_seen = BTreeSet::new();
    for quest in found
        .quests
        .iter()
        .filter(|quest| quest.building == building)
    {
        let chain = match quest.id.slot {
            Slot::Giver { slot, .. } => Some(slot),
            _ => None,
        };
        if book.done.contains_key(&quest.id) || chains_seen.contains(&chain) {
            continue;
        }
        chains_seen.insert(chain);
        offers.extend(offered(client, world, quest, now));
    }
    if daily_givers(&found.poi, &found.buildings).contains(&building) {
        offers.extend(daily_offer(client, world, town, building, now));
    }
    offers
}

/// A giver's daily: the one taken, if any; else today's, a day after the last
/// was handed in (owner).
fn daily_offer(
    client: &Client,
    world: &mut ServerWorld,
    town: ChunkPos,
    building: u16,
    now: i64,
) -> Option<Offered> {
    let book = &client.character.quests;
    let taken = book.taken.keys().find(|id| {
        id.town == town && matches!(id.slot, Slot::Daily { building: b, .. } if b == building)
    });
    let quest = match taken {
        Some(&id) => cache::quest(world, id)?,
        None => {
            let last = book.dailies.get(&(town, building)).map(|daily| daily.last);
            if last.is_some_and(|last| now - last < SECONDS_PER_DAY) {
                return None;
            }
            (*cache::daily(world, town, building, utc_day(now))?).clone()
        }
    };
    offered(client, world, &quest, now)
}

/// Where one quest stands for the character, if they are told of it at all.
fn offered(client: &Client, world: &ServerWorld, quest: &Quest, now: i64) -> Option<Offered> {
    let book = &client.character.quests;
    let state = if let Some(progress) = book.taken.get(&quest.id) {
        match quest.ready_to_turn_in(&done_flags(quest, &progress.counts)) {
            true => OfferState::Ready,
            false => OfferState::Taken,
        }
    } else {
        if !book.may_retake(&quest.id, now) {
            return None;
        }
        match judge(quest, client, world) {
            Verdict::Hidden => return None,
            Verdict::Missing(hints) => OfferState::Locked(hints),
            Verdict::Met => OfferState::Available {
                story: Box::new(quest.story.clone()),
                reward: quest.reward.clone(),
            },
        }
    };
    Some(Offered {
        quest: quest.clone(),
        state,
    })
}

/// Whether `id` is on offer to the character from its giver now.
pub(crate) fn on_offer(
    client: &Client,
    world: &mut ServerWorld,
    id: QuestId,
    building: u16,
    now: i64,
) -> Option<Quest> {
    let offers = giver_offers(client, world, id.town, building, now);
    (offers.into_iter())
        .find(|offer| offer.quest.id == id && matches!(offer.state, OfferState::Available { .. }))
        .map(|offer| offer.quest)
}

/// The marker over each giver's door in a town that has one, by the owner's
/// order: "?" to hand in, a grey "?" for one under way, "!" on offer, then a
/// grey "!".
pub(crate) fn town_markers(
    client: &Client,
    world: &mut ServerWorld,
    town: ChunkPos,
    now: i64,
) -> Vec<(u16, Marker)> {
    let Some(found) = cache::town(world, town) else {
        return Vec::new();
    };
    let mut buildings: Vec<u16> = found.quests.iter().map(|quest| quest.building).collect();
    buildings.extend(daily_givers(&found.poi, &found.buildings));
    buildings.sort_unstable();
    buildings.dedup();
    let mut markers = Vec::new();
    for building in buildings {
        let offers = giver_offers(client, world, town, building, now);
        if let Some(marker) = offers.iter().map(marker_of).min_by_key(|m| m.0) {
            markers.push((building, marker.1));
        }
    }
    markers
}

/// The marker one quest would give its giver's door, with its rank (lowest
/// shows).
fn marker_of(offer: &Offered) -> (u8, Marker) {
    match offer.state {
        OfferState::Ready => (0, Marker::Ready(offer.quest.kind)),
        OfferState::Taken => (1, Marker::Taken),
        OfferState::Available { .. } => (2, Marker::Available(offer.quest.kind)),
        OfferState::Locked(_) => (3, Marker::Locked),
    }
}
