//! Taking in what the server says about quests.

use bevy::prelude::*;
use shared::protocol::{QuestNews, ServerMessage};

use super::{DoorMarkers, GateQuests, QuestBook};
use crate::connection::FromServer;

pub(super) fn read_server(
    mut from_server: MessageReader<FromServer>,
    mut book: ResMut<QuestBook>,
    mut doors: ResMut<DoorMarkers>,
    mut gates: ResMut<GateQuests>,
) {
    for FromServer(message) in from_server.read() {
        match message {
            // (A welcome may be from another server, or another world.)
            ServerMessage::Welcome { .. } => {
                *book = QuestBook::default();
                *doors = DoorMarkers::default();
                *gates = GateQuests::default();
            }
            ServerMessage::Quest(news) => take(news, &mut book, &mut doors, &mut gates),
            _ => {}
        }
    }
}

fn take(news: &QuestNews, book: &mut QuestBook, doors: &mut DoorMarkers, gates: &mut GateQuests) {
    match news {
        QuestNews::Log { quests, .. } => {
            book.entries = (quests.iter())
                .map(|entry| (entry.id, entry.clone()))
                .collect();
        }
        QuestNews::Changed(entry) => {
            book.entries.insert(entry.id, entry.clone());
        }
        QuestNews::Markers { town, markers } => {
            doors.town = Some(*town);
            doors.markers = markers.iter().copied().collect();
        }
        QuestNews::Planets(planets) => {
            gates.0 = (planets.iter())
                .map(|planet| (planet.world, planet.ready))
                .collect();
        }
        _ => {}
    }
}
