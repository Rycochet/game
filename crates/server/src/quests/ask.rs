//! Answering what a client asks about quests (`QuestAsk`), in turn with the
//! player's steps, as talking and handing in go by where they stand.

use bevy::prelude::*;
use shared::protocol::{Offer, QuestAsk, QuestNews, ServerMessage};

use super::clock::unix_now;
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
        QuestAsk::Log => {
            let whole = log::whole_log(client, world);
            client.link.send(&ServerMessage::Quest(whole))
        }
    }
}
