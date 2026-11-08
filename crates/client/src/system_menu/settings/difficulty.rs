//! Settings' Gameplay page: the Difficulty dropdown. Before playing, it is the
//! player's own default (`Preferences::difficulty`), any of the four, asking
//! first for Ironman (`ironman`). While playing, it is the character's, from
//! those the server allows, asked of the server (`SetDifficulty`); it is
//! disabled when there is nothing else to change to, and always on Ironman,
//! which is only chosen when making a character.

use bevy::prelude::*;
use shared::difficulty::Difficulty;
use shared::protocol::ClientMessage;

use super::super::{Action, SystemMenu};
use super::rows::{spawn_dropdown_row, tip};
use crate::common::{PlayDifficulty, Preferences, difficulty_words};
use crate::connection::ToServer;
use crate::widgets::{ButtonPressed, ButtonState, Dropdown, DropdownChosen, UiId};

const WIDTH: f32 = 200.0;

/// The dropdown, with the difficulty each option is.
#[derive(Component)]
pub(in crate::system_menu) struct DifficultyList(Vec<Difficulty>);

/// What the dropdown offers, which is chosen, and why it can't be changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::system_menu) struct Choices {
    pub options: Vec<Difficulty>,
    pub selected: Difficulty,
    pub why_not: Option<&'static str>,
}

/// What to offer: with no character, the player's own default (any); with
/// one, those the server allows that it may change to.
pub(in crate::system_menu) fn choices(play: PlayDifficulty, own: Difficulty) -> Choices {
    let Some(current) = play.current else {
        return Choices {
            options: Difficulty::ALL.to_vec(),
            selected: own,
            why_not: None,
        };
    };
    let fixed = |why_not| Choices {
        options: vec![current],
        selected: current,
        why_not: Some(why_not),
    };
    if current == Difficulty::Ironman {
        return fixed("Ironman is chosen when making a character, and can't be changed.");
    }
    let options: Vec<_> = play
        .allowed
        .iter()
        .filter(|&to| current.may_change_to(to))
        .collect();
    if options.len() < 2 {
        return fixed("This server allows no other difficulty to change to.");
    }
    Choices {
        options,
        selected: current,
        why_not: None,
    }
}

/// The row's tip: what each difficulty means, and what the choice is for.
fn words(playing: bool) -> String {
    let what = if playing {
        "How hard the game is for this character."
    } else {
        "Your own difficulty: what new characters start on, and characters who have none yet."
    };
    let each: Vec<_> = Difficulty::ALL.into_iter().map(difficulty_words).collect();
    format!("{what}\n\n{}", each.join("\n"))
}

/// Adds the row to the Gameplay page.
pub(in crate::system_menu) fn spawn_difficulty_row(
    page: &mut ChildSpawnerCommands,
    choices: &Choices,
    playing: bool,
) {
    let dropdown = Dropdown {
        options: choices
            .options
            .iter()
            .map(|d| d.name().to_owned())
            .collect(),
        selected: choices
            .options
            .iter()
            .position(|&d| d == choices.selected)
            .unwrap_or(0),
        ..default()
    };
    let tip = tip(&words(playing), choices.why_not);
    let made = spawn_dropdown_row(page, "Difficulty", tip, dropdown, WIDTH);
    let mut commands = page.commands();
    let mut made = commands.entity(made);
    made.insert((
        DifficultyList(choices.options.clone()),
        UiId::new("settings.gameplay", "difficulty"),
    ));
    if choices.why_not.is_some() {
        made.insert(ButtonState::Disabled);
    }
}

/// A choice: asked of the server while playing; otherwise the player's own,
/// asking first for Ironman (whose yes is `Action::ChooseIronman`).
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn use_difficulty(
    mut chosen: MessageReader<DropdownChosen>,
    mut pressed: MessageReader<ButtonPressed>,
    lists: Query<&DifficultyList>,
    actions: Query<&Action>,
    play: Res<PlayDifficulty>,
    mut menu: ResMut<SystemMenu>,
    mut preferences: ResMut<Preferences>,
    mut to_server: MessageWriter<ToServer>,
) {
    for &DropdownChosen(list, index) in chosen.read() {
        let Some(&difficulty) = lists.get(list).ok().and_then(|list| list.0.get(index)) else {
            continue;
        };
        if play.current.is_some() {
            to_server.write(ToServer(ClientMessage::SetDifficulty(difficulty)));
        } else if difficulty == Difficulty::Ironman && preferences.difficulty != difficulty {
            *menu = SystemMenu::Ironman;
        } else if preferences.difficulty != difficulty {
            preferences.difficulty = difficulty;
        }
    }
    let yes = pressed
        .read()
        .filter_map(|press| actions.get(press.0).ok())
        .any(|&action| action == Action::ChooseIronman);
    if yes {
        preferences.difficulty = Difficulty::Ironman;
    }
}

#[cfg(test)]
mod tests;
