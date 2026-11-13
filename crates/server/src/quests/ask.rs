//! Answering what a client asks about quests (`QuestAsk`), in turn with the
//! player's steps, as talking and handing in go by where they stand.

use bevy::prelude::*;
use shared::protocol::{Offer, QuestAsk, QuestNews, ServerMessage};
use shared::quest::QuestId;

use super::clock::unix_now;
use super::planets::tell_planets;
use super::{accept, give_up, log, offers, send_markers, turn_in};
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::beside_giver;

/// Answers one ask. `false` if the client has gone.
pub(crate) fn answer_quest(client: &mut Client, world: &mut ServerWorld, ask: QuestAsk) -> bool {
    let now = unix_now();
    match ask {
        QuestAsk::Talk { town, building } => {
            if !beside_giver(client, world, town, building) {
                warn!("{} talked to a giver they aren't beside", client.in_log());
                return true;
            }
            let offers = (offers::giver_offers(client, world, town, building, now).into_iter())
                .map(|offered| Offer {
                    id: offered.quest.id,
                    kind: offered.quest.kind,
                    giver: offered.quest.story.cast.giver().person.clone(),
                    state: offered.state,
                })
                .collect();
            let talk = QuestNews::Talk {
                town,
                building,
                offers,
            };
            client.link.send(&ServerMessage::Quest(talk))
        }
        QuestAsk::Markers { town } => {
            if client.quest_seen.town() != Some(town) {
                warn!(
                    "{} asked for the markers of a town they aren't in",
                    client.in_log()
                );
                return true;
            }
            send_markers(client, world, town, now)
        }
        QuestAsk::Accept(id) => accept::accept(client, world, id, now, true),
        QuestAsk::Abandon(id) => give_up::end(client, world, id, false, now),
        QuestAsk::TurnIn(id) => turn_in::turn_in(client, world, id, now, true),
        QuestAsk::Enter(id) => super::dungeon::enter(client, world, id, true),
        QuestAsk::Log => {
            let whole = log::whole_log(client, world);
            client.link.send(&ServerMessage::Quest(whole))
        }
        QuestAsk::PlanetLog(planet) => {
            client.quest_seen.ask_planet_log(planet);
            true
        }
        QuestAsk::Planets => tell_planets(client),
        QuestAsk::Track(id) => track(client, id),
    }
}

/// Follows a quest taken on this planet (or none). One not taken is refused
/// by answering with what is followed now.
fn track(client: &mut Client, id: Option<QuestId>) -> bool {
    let book = &mut client.character.quests;
    if id.is_none_or(|id| book.taken.contains_key(&id)) {
        book.tracked = id;
    }
    let news = QuestNews::Tracked(book.tracked);
    client.link.send(&ServerMessage::Quest(news))
}
