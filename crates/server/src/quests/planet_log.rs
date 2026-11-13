//! Another planet's quests for the Quest Log (`QuestAsk::PlanetLog`), made
//! again from that planet's towns: answered once every client's messages this
//! tick have been read, as it may need the planet's world opened (it closes
//! again when nobody is there, `WORLD_KEPT_FOR`).

use bevy::prelude::*;
use shared::protocol::{QuestNews, ServerMessage};

use super::log::book_log;
use crate::ServerWorlds;
use crate::connections::{Client, Phase};

/// Answers each client that asked for a planet's log this tick.
pub(crate) fn answer_planet_logs(clients: &mut [Client], worlds: &mut ServerWorlds) {
    for client in clients.iter_mut() {
        let Some(id) = client.quest_seen.take_planet_log() else {
            continue;
        };
        if client.is_gone() || client.phase != Phase::Playing {
            continue;
        }
        let here = client.world == Some(id);
        let book = match here {
            true => &client.character.quests,
            false => match client.other_worlds().get(&id) {
                Some(save) => &save.quests,
                None => continue,
            },
        };
        if !book.has_history() {
            continue;
        }
        let world = match here {
            true => Some(worlds.of(client.world)),
            false => match worlds.open_saved(id) {
                Ok(Some(open)) => worlds.get_mut(open),
                Ok(None) => None,
                Err(error) => {
                    warn!("{} asked for a planet's quests: {error}", client.in_log());
                    None
                }
            },
        };
        let Some(world) = world else {
            continue;
        };
        let (quests, dailies) = book_log(book, world);
        let news = QuestNews::PlanetLog {
            world: id,
            quests,
            dailies,
        };
        // A client gone is noticed when its messages are next read.
        let _ = client.link.send(&ServerMessage::Quest(news));
    }
}
