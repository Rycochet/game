//! What the server says of givers: what one has (opening their panel at its
//! Quests tab, if there is anything), a quest handed in, and something asked
//! that couldn't be done.

use bevy::prelude::*;
use shared::protocol::{OfferState, QuestNews, ServerMessage};

use super::spawn::open_panel;
use super::words::refusal;
use super::{OpenTalks, Tab};
use crate::common::{CurrentDungeon, Wording};
use crate::connection::FromServer;
use crate::dom::Dom;

#[allow(clippy::too_many_arguments)]
pub(super) fn read_quest_news(
    mut from_server: MessageReader<FromServer>,
    dungeon: Res<CurrentDungeon>,
    wording: Res<Wording>,
    mut open: ResMut<OpenTalks>,
    mut dom: Dom,
    mut commands: Commands,
) {
    for FromServer(message) in from_server.read() {
        let ServerMessage::Quest(news) = message else {
            continue;
        };
        match news {
            QuestNews::Talk {
                town,
                building,
                offers,
            } => {
                let talk = match open.is_open(*building) || !offers.is_empty() {
                    true => {
                        let (open, dungeon) = (&mut open, &dungeon);
                        open_panel(
                            open,
                            *building,
                            Tab::Quests,
                            dungeon,
                            &mut dom,
                            &mut commands,
                        )
                    }
                    // Nothing to say, and no panel to say it on: a shop's
                    // stock opens it at its Trade tab.
                    false => None,
                };
                let Some(talk) = talk else {
                    continue;
                };
                let first = !talk.heard;
                talk.heard = true;
                talk.town = Some(*town);
                talk.offers.clone_from(offers);
                talk.dirty = true;
                if talk
                    .chosen
                    .is_some_and(|id| !offers.iter().any(|o| o.id == id))
                {
                    talk.chosen = None;
                }
                if first {
                    // Something to take or hand in shows first; one alone,
                    // in full.
                    let wanted = offers.iter().filter(|offer| {
                        matches!(
                            offer.state,
                            OfferState::Available { .. } | OfferState::Ready
                        )
                    });
                    if wanted.clone().next().is_some() {
                        talk.tab = Tab::Quests;
                    }
                    if let [only] = wanted.collect::<Vec<_>>()[..] {
                        talk.chosen = Some(only.id);
                    }
                }
            }
            QuestNews::TurnedIn { id, ending } => {
                for talk in open.0.iter_mut().filter(|talk| talk.chosen == Some(*id)) {
                    talk.ended = Some((*id, ending.clone()));
                    talk.chosen = None;
                    talk.refused = None;
                    talk.dirty = true;
                }
            }
            QuestNews::Refused { id, why } => {
                let mine =
                    |talk: &&mut super::OpenTalk| talk.offers.iter().any(|offer| offer.id == *id);
                for talk in open.0.iter_mut().filter(mine) {
                    talk.refused = Some(refusal(&wording.0, *why));
                    talk.dirty = true;
                }
            }
            // A quest taken moving on: shown as it is now.
            QuestNews::Changed(_) => {
                for talk in &mut open.0 {
                    talk.dirty = true;
                }
            }
            _ => {}
        }
    }
}
