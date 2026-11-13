//! What the player does on a talk panel: its tabs, choosing a quest from the
//! list, and Accept, Decline, Complete and Back. Taking or handing in a quest
//! asks the giver again after, so the panel shows what they have now. A tap
//! arrives as a press, so all of it works by touch.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, QuestAsk};

use super::{OpenTalk, OpenTalks, Tab};
use crate::connection::ToServer;
use crate::dom::{UiEvent, UiValue};

pub(super) fn read_events(
    mut events: MessageReader<UiEvent>,
    mut open: ResMut<OpenTalks>,
    mut to_server: MessageWriter<ToServer>,
) {
    for event in events.read() {
        let (("talk", name), Some(key)) = (event.id.parts(), event.id.which()) else {
            continue;
        };
        let Some(talk) = open.by_key(key) else {
            continue;
        };
        let asks = match &event.value {
            UiValue::Choice { index, .. } if name == "tabs" => {
                talk.tab = if *index == 0 { Tab::Quests } else { Tab::Trade };
                talk.dirty = true;
                Vec::new()
            }
            UiValue::Pressed => pressed(talk, name, event.id.index()),
            _ => Vec::new(),
        };
        for ask in asks {
            to_server.write(ToServer(ClientMessage::Quest(ask)));
        }
    }
}

/// A button: what to ask the server, if anything.
fn pressed(talk: &mut OpenTalk, name: &str, at: Option<usize>) -> Vec<QuestAsk> {
    talk.dirty = true;
    talk.refused = None;
    let chosen = talk.chosen;
    // Asking again after, so the list is what the giver has now.
    let and_again = |ask: QuestAsk| match talk.town {
        Some(town) => vec![
            ask,
            QuestAsk::Talk {
                town,
                building: talk.building,
            },
        ],
        None => vec![ask],
    };
    match name {
        "offer" => {
            let offer = at.and_then(|at| talk.offers.get(at));
            talk.chosen = offer.map(|offer| offer.id);
            Vec::new()
        }
        "accept" => match chosen {
            Some(id) => and_again(QuestAsk::Accept(id)),
            None => Vec::new(),
        },
        "turn_in" => match chosen {
            Some(id) => and_again(QuestAsk::TurnIn(id)),
            None => Vec::new(),
        },
        // Going in closes the panel, as any new level does.
        "enter" => chosen.map(QuestAsk::Enter).into_iter().collect(),
        "decline" | "back" => {
            talk.chosen = None;
            talk.ended = None;
            Vec::new()
        }
        _ => Vec::new(),
    }
}
