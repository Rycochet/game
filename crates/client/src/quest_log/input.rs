//! What the player does on the log: choosing a planet or a quest, Track,
//! Abandon (pressed twice) and the places a quest sends them to, which open
//! the full map there. A tap arrives as a press, so all of it works by touch.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, QuestAsk};

use super::state::LogPanel;
use crate::common::CurrentWorld;
use crate::connection::ToServer;
use crate::dom::{UiEvent, UiValue};
use crate::minimap::FullMap;
use crate::quests::QuestBook;

pub(super) fn read_events(
    mut events: MessageReader<UiEvent>,
    mut panel: ResMut<LogPanel>,
    book: Res<QuestBook>,
    world: Option<Res<CurrentWorld>>,
    mut full_map: ResMut<FullMap>,
    mut to_server: MessageWriter<ToServer>,
) {
    for event in events.read() {
        let ("quest_log", name) = event.id.parts() else {
            continue;
        };
        let at = event.id.index();
        let ask = match (&event.value, name) {
            (UiValue::Choice { index, .. }, "planet") => choose_planet(&mut panel, *index),
            (UiValue::Pressed, "quest") => {
                panel.chosen = at.and_then(|at| panel.rows.get(at).copied());
                panel.abandoning = false;
                None
            }
            (UiValue::Pressed, "track") => panel.chosen.map(|id| {
                // Pressing the one chosen to follow stops following it.
                let again = book.tracked == Some(id);
                QuestAsk::Track((!again).then_some(id))
            }),
            (UiValue::Pressed, "abandon") => {
                let sure = panel.abandoning;
                panel.abandoning = !sure;
                panel.chosen.filter(|_| sure).map(QuestAsk::Abandon)
            }
            (UiValue::Pressed, "target") => {
                let chunk = at.and_then(|at| panel.targets.get(at).copied());
                let poi = chunk.and_then(|chunk| world.as_ref()?.pois.poi_in(chunk));
                if let Some(poi) = poi {
                    full_map.open = true;
                    full_map.centre = Some((poi.x as f64, poi.y as f64));
                }
                None
            }
            _ => continue,
        };
        panel.dirty = true;
        if let Some(ask) = ask {
            to_server.write(ToServer(ClientMessage::Quest(ask)));
        }
    }
}

/// Shows another planet (asking the server for its quests) or this one.
fn choose_planet(panel: &mut LogPanel, index: usize) -> Option<QuestAsk> {
    let planet = panel.choices.get(index).copied().flatten();
    panel.planet = planet;
    panel.chosen = None;
    panel.abandoning = false;
    planet.map(QuestAsk::PlanetLog)
}
