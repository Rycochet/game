//! What the server tells the log: another planet's quests, and that the
//! planets, or this planet's quests, have changed.

use bevy::prelude::*;
use shared::protocol::{QuestNews, ServerMessage};

use super::state::{LogPanel, Told};
use crate::connection::FromServer;
use crate::quests::{QuestBook, QuestPlanets};

pub(super) fn read_server(
    mut from_server: MessageReader<FromServer>,
    mut panel: ResMut<LogPanel>,
    book: Res<QuestBook>,
    planets: Res<QuestPlanets>,
) {
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::Welcome { .. } => {
                panel.others.clear();
                panel.runes.clear();
                panel.planet = None;
                panel.chosen = None;
            }
            ServerMessage::Quest(QuestNews::PlanetLog {
                world,
                quests,
                dailies,
            }) => {
                let told = Told {
                    quests: quests.clone(),
                    dailies: dailies.clone(),
                };
                panel.others.insert(*world, told);
                panel.dirty = true;
            }
            _ => {}
        }
    }
    if planets.is_changed() {
        panel.planets_dirty = true;
        panel.dirty = true;
    }
    if book.is_changed() && panel.planet.is_none() {
        panel.dirty = true;
    }
}
