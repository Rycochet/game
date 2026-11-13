//! Server Options' difficulties (native only): a tickbox for each that the
//! built-in server allows. Kept at once (`Preferences::server_difficulties`),
//! and `connection` asks the server to follow while playing on it. One is
//! always ticked: the last can't be unticked. `HOW_DIFFICULTIES`, if set when
//! the game started, decides instead: the boxes show it, disabled, with a tip
//! saying why.

use bevy::prelude::*;
use shared::difficulty::{Difficulties, Difficulty};

use super::super::{BUTTON_ROW, FONT, GAP};
use crate::common::{Preferences, difficulty_words};
use crate::dom::HelpAfter;
use crate::widgets::{
    ButtonPressed, ButtonState, ButtonTooltip, Checked, HoverTip, Pressable, UiId, field_label,
    spawn_check_button,
};

/// A tickbox, for this difficulty.
#[derive(Component)]
pub(in crate::system_menu) struct DifficultyBox(Difficulty);

const SET_BY_ENVIRONMENT: &str = "Set by HOW_DIFFICULTIES when the game was started.";
const LAST_ONE: &str = "At least one difficulty must be allowed.";

pub(super) fn spawn_difficulty_row(panel: &mut ChildSpawnerCommands) {
    // In a row of its own, so a "?" can follow the words when there is a
    // reason the boxes can't be changed.
    let mut heading = panel.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    });
    let mut label = None;
    heading.with_children(|heading| {
        label = Some(
            heading
                .spawn(field_label("Difficulties players may choose"))
                .id(),
        );
    });
    if server::difficulties_from_environment().is_some()
        && let Some(label) = label
    {
        heading.insert((HoverTip(SET_BY_ENVIRONMENT.into()), HelpAfter(label)));
    }
    let row = Node {
        column_gap: px(GAP),
        flex_wrap: FlexWrap::Wrap,
        ..default()
    };
    panel.spawn(row).with_children(|row| {
        for difficulty in Difficulty::ALL {
            let node = Node {
                height: px(BUTTON_ROW),
                ..default()
            };
            let name = difficulty.name();
            spawn_check_button(row, ButtonState::Normal, Some(true), name, FONT, node).insert((
                DifficultyBox(difficulty),
                Pressable,
                ButtonTooltip(difficulty_words(difficulty).into()),
                UiId::new("server_options", "difficulty").at(difficulty as usize),
            ));
        }
    });
}

/// How a tickbox for `difficulty` is shown, of `allowed`: disabled when the
/// environment decides, or when it is the last one ticked.
pub(super) fn box_state(difficulty: Difficulty, allowed: Difficulties, fixed: bool) -> ButtonState {
    let ticked = allowed.contains(difficulty);
    let last = ticked && allowed.len() == 1;
    match (fixed || last, ticked) {
        (false, _) => ButtonState::Normal,
        (true, false) => ButtonState::Disabled,
        (true, true) => ButtonState::DisabledSelected,
    }
}

/// A press ticks or unticks one (never the last); the boxes show what is kept.
pub(in crate::system_menu) fn run_difficulty_boxes(
    mut pressed: MessageReader<ButtonPressed>,
    mut preferences: ResMut<Preferences>,
    mut boxes: Query<(
        &DifficultyBox,
        &mut Checked,
        &mut ButtonState,
        Option<&mut ButtonTooltip>,
    )>,
    mut environment: Local<Option<Option<Difficulties>>>,
) {
    // Read once: an unknown name in it is warned about each time.
    let fixed = *environment.get_or_insert_with(server::difficulties_from_environment);
    for press in pressed.read() {
        let Ok((&DifficultyBox(difficulty), ..)) = boxes.get(press.0) else {
            continue;
        };
        let allowed = preferences.server_difficulties.or_all();
        // (Disabled boxes aren't pressed, but a press may come before they are.)
        if fixed.is_some() || box_state(difficulty, allowed, false) != ButtonState::Normal {
            continue;
        }
        let allowed = if allowed.contains(difficulty) {
            allowed.without(difficulty)
        } else {
            allowed.with(difficulty)
        };
        preferences.server_difficulties = allowed;
    }
    let allowed = fixed.unwrap_or(preferences.server_difficulties.or_all());
    for (&DifficultyBox(difficulty), mut checked, mut state, tooltip) in &mut boxes {
        let ticked = allowed.contains(difficulty);
        if checked.0 != ticked {
            checked.0 = ticked;
        }
        state.set_if_neq(box_state(difficulty, allowed, fixed.is_some()));
        let words = match () {
            _ if fixed.is_some() => SET_BY_ENVIRONMENT,
            _ if ticked && allowed.len() == 1 => LAST_ONE,
            _ => difficulty_words(difficulty),
        };
        if let Some(mut tooltip) = tooltip
            && tooltip.0 != words
        {
            tooltip.0 = words.into();
        }
    }
}

#[cfg(test)]
mod tests;
