//! The account's characters: what the server offers (`Characters`), played at
//! once for Play or shown for choosing (`common::CharacterSelect`), what the
//! player chooses there (`CharacterChoice`), and which servers this computer
//! has characters on, for Play (`Preferences::played_on`).

use bevy::prelude::*;
use shared::protocol::{ClientMessage, ServerMessage};

use super::socket::Target;
use super::{Connecting, FromServer, ServerConnection};
use crate::common::{
    CharacterChoice, CharacterSelect, ChosenServer, MakingCharacter, PlayDifficulty, PlayFor,
    PlayableHere, Preferences, Session,
};

/// What `Preferences::played_on` calls a server.
fn played_key(target: &Target) -> String {
    match target {
        #[cfg(not(target_arch = "wasm32"))]
        Target::Embedded => "built-in".into(),
        Target::Url { url, .. } => url.clone(),
    }
}

/// Notes whether this computer has a character on the server, changing the
/// preferences (and so saving them) only if that changes.
fn note_played(preferences: &mut Preferences, target: &Target, any: bool) {
    let key = played_key(target);
    if preferences.played_on.contains(&key) != any {
        if any {
            preferences.played_on.insert(key);
        } else {
            preferences.played_on.remove(&key);
        }
    }
}

/// Takes what the server offers. The first offer after connecting for Play goes
/// straight to the character played last; anything else (Characters, none to
/// play, or an offer after one was deleted) shows the screen.
#[allow(clippy::too_many_arguments)]
pub(super) fn answer_characters(
    mut commands: Commands,
    mut from_server: MessageReader<FromServer>,
    connection: Res<ServerConnection>,
    mut connecting: ResMut<Connecting>,
    mut select: ResMut<CharacterSelect>,
    play_for: Res<PlayFor>,
    mut preferences: ResMut<Preferences>,
) {
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::Characters {
                list,
                can_create,
                difficulties,
            } => {
                // What a new character may be.
                commands.insert_resource(PlayDifficulty {
                    current: None,
                    allowed: *difficulties,
                });
                let any = !list.is_empty();
                if preferences
                    .played_on
                    .contains(&played_key(&connecting.target))
                    != any
                {
                    note_played(&mut preferences, &connecting.target, any);
                }
                let first = !std::mem::replace(&mut connecting.offered, true);
                select.list = list.clone();
                select.can_create = *can_create;
                let latest = select.sorted().first().map(|c| c.summary.id);
                match latest {
                    Some(id) if first && *play_for == PlayFor::Latest => {
                        connection.link.send(&ClientMessage::ChooseCharacter { id });
                        select.open = false;
                    }
                    _ => {
                        select.open = true;
                        select.reached = true;
                    }
                }
            }
            ServerMessage::Welcome { .. } => {
                if !preferences
                    .played_on
                    .contains(&played_key(&connecting.target))
                {
                    note_played(&mut preferences, &connecting.target, true);
                }
                select.open = false;
            }
            _ => {}
        }
    }
}

/// Acts on what the player chose on the screens for choosing and making
/// characters.
pub(super) fn choose_character(
    mut choices: MessageReader<CharacterChoice>,
    connection: Option<Res<ServerConnection>>,
    mut select: ResMut<CharacterSelect>,
    mut making: ResMut<MakingCharacter>,
    mut session: ResMut<Session>,
) {
    for choice in choices.read() {
        let send = |message: ClientMessage| {
            if let Some(connection) = &connection {
                connection.link.send(&message);
            }
        };
        match *choice {
            CharacterChoice::Play(id) => {
                send(ClientMessage::ChooseCharacter { id });
                select.open = false;
            }
            CharacterChoice::Delete(id) => send(ClientMessage::DeleteCharacter { id }),
            CharacterChoice::New => {
                select.open = false;
                making.0 = true;
            }
            CharacterChoice::CancelNew if select.reached => {
                making.0 = false;
                select.open = true;
            }
            CharacterChoice::CancelNew | CharacterChoice::Cancel => {
                // Logging out hangs up, and starts these afresh.
                making.0 = false;
                select.open = false;
                *session = Session::AtStart;
            }
        }
    }
}

/// Whether Play's server has a character of this player's, for the startup
/// menu: again whenever that may have changed.
pub(super) fn show_playable(
    chosen: Res<ChosenServer>,
    connecting: Res<Connecting>,
    preferences: Res<Preferences>,
    mut playable: ResMut<PlayableHere>,
) {
    if !chosen.is_changed() && !preferences.is_changed() && !connecting.is_changed() {
        return;
    }
    let target = Target::choose(&chosen, connecting.environment);
    let here = preferences.played_on.contains(&played_key(&target));
    playable.set_if_neq(PlayableHere(here));
}
