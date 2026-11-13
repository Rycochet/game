//! Taking in what the server says about quests.

use bevy::prelude::*;
use shared::protocol::{QuestNews, ServerMessage};

use super::{BossMarks, DoorMarkers, GateQuests, LevelDots, QuestBook, QuestPlanets};
use crate::connection::FromServer;

pub(super) fn read_server(
    mut from_server: MessageReader<FromServer>,
    mut book: ResMut<QuestBook>,
    mut doors: ResMut<DoorMarkers>,
    mut gates: ResMut<GateQuests>,
    mut planets: ResMut<QuestPlanets>,
    mut dots: ResMut<LevelDots>,
    mut bosses: ResMut<BossMarks>,
) {
    for FromServer(message) in from_server.read() {
        match message {
            // (A welcome may be from another server, or another world.)
            ServerMessage::Welcome { .. } => {
                *book = QuestBook::default();
                *doors = DoorMarkers::default();
                *gates = GateQuests::default();
                *planets = QuestPlanets::default();
                *dots = LevelDots::default();
                *bosses = BossMarks::default();
            }
            ServerMessage::Quest(QuestNews::Bosses(told)) => {
                bosses.0 = told.iter().map(|mark| (mark.id, mark.sign)).collect();
            }
            ServerMessage::Quest(QuestNews::Dots(told)) => {
                dots.0 = (told.iter())
                    .map(|quest| (quest.id, quest.cells.clone()))
                    .collect();
            }
            ServerMessage::Quest(QuestNews::Planets(told)) => {
                gates.0 = (told.iter())
                    .filter(|planet| planet.taken)
                    .map(|planet| (planet.world, planet.ready))
                    .collect();
                planets.0.clone_from(told);
            }
            ServerMessage::Quest(news) => take(news, &mut book, &mut doors),
            _ => {}
        }
    }
}

fn take(news: &QuestNews, book: &mut QuestBook, doors: &mut DoorMarkers) {
    match news {
        QuestNews::Log {
            quests,
            dailies,
            tracked,
        } => {
            book.entries = (quests.iter())
                .map(|entry| (entry.id, entry.clone()))
                .collect();
            book.dailies.clone_from(dailies);
            book.tracked = *tracked;
        }
        QuestNews::Tracked(tracked) => book.tracked = *tracked,
        QuestNews::Slain(slain) => book.slain.clone_from(slain),
        QuestNews::Changed(entry) => {
            book.entries.insert(entry.id, entry.clone());
        }
        QuestNews::Markers { town, markers } => {
            doors.town = Some(*town);
            doors.markers = markers.iter().copied().collect();
        }
        _ => {}
    }
}
