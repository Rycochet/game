//! The Create Mob panel (owner's design): what the mob will be, and Create, a
//! toggle that turns Mob Mode on: clicks then put that mob on the cell clicked.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::sheet::Feeling;

use super::{CHECK_ROW, FONT, SEED_WIDTH, row, words};
use crate::widgets::{
    BUTTON_PADDING, ButtonState, Dropdown, Pressable, TextBox, UiId, WARNING_TEXT, spawn_button,
    spawn_check_button, spawn_dropdown, spawn_new_seed_button, spawn_text_box,
};

const ARROW_ROOM: f32 = 24.0;

/// The attitudes to choose from, by the dropdown's order: the kind's own first.
pub(super) const ATTITUDES: [(&str, Option<Feeling>); 4] = [
    ("Default", None),
    ("Friendly", Some(Feeling::Friendly)),
    ("Neutral", Some(Feeling::Neutral)),
    ("Unfriendly", Some(Feeling::Unfriendly)),
];

/// Which of the panel's dropdowns it is.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(in crate::debug_menu) enum MobChoice {
    Kind,
    Attitude,
    Gear,
}

#[derive(Component)]
pub(in crate::debug_menu) struct MobStill;

#[derive(Component)]
pub(in crate::debug_menu) struct MobSeed;

#[derive(Component)]
pub(in crate::debug_menu) struct MobNewSeed;

/// Create: on while Mob Mode is.
#[derive(Component)]
pub(in crate::debug_menu) struct MobCreate;

/// "Click to place", shown while Mob Mode is on.
#[derive(Component)]
pub(in crate::debug_menu) struct MobClickToPlace;

/// The panel; `start_create_mob_panel` sets it to `common::CreateMobOrder`.
pub(super) fn spawn_create_mob_panel(content: &mut ChildSpawnerCommands) {
    content.spawn(row()).with_children(|line| {
        line.spawn(words("Kind"));
        let kinds = CreatureKind::ALL
            .iter()
            .map(|kind| kind.def().name.to_owned());
        dropdown(line, MobChoice::Kind, kinds, true);
        line.spawn(words("Attitude"));
        dropdown(
            line,
            MobChoice::Attitude,
            ATTITUDES.map(|(name, _)| name),
            false,
        );
        line.spawn(words("Gear"));
        dropdown(line, MobChoice::Gear, ["Rolled", "None"], false);
        let node = Node {
            height: px(CHECK_ROW),
            ..default()
        };
        spawn_check_button(line, ButtonState::Normal, Some(false), "Still", FONT, node).insert((
            MobStill,
            Pressable,
            id("still"),
        ));
    });
    content.spawn(row()).with_children(|line| {
        line.spawn(words("Seed"));
        let seed = TextBox {
            digits_only: true,
            takes_keys: true,
            ..default()
        };
        let node = Node {
            width: px(SEED_WIDTH),
            height: px(CHECK_ROW),
            ..default()
        };
        spawn_text_box(line, seed, FONT, node).insert((MobSeed, id("seed")));
        spawn_new_seed_button(line, CHECK_ROW).insert((MobNewSeed, id("new_seed")));
    });
    let bottom = Node {
        justify_content: JustifyContent::End,
        align_items: AlignItems::Center,
        ..row()
    };
    content.spawn(bottom).with_children(|line| {
        let words = (
            Text::new("Click to place"),
            FONT.font(),
            TextColor(WARNING_TEXT),
        );
        line.spawn((words, Visibility::Hidden, MobClickToPlace));
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(line, ButtonState::Normal, "Create", FONT, node).insert((
            MobCreate,
            Pressable,
            id("create"),
        ));
    });
}

/// A widget of the Create Mob panel.
fn id(name: &'static str) -> UiId {
    UiId::new("debug.create_mob", name)
}

fn dropdown<S: Into<String>>(
    line: &mut ChildSpawnerCommands,
    choice: MobChoice,
    options: impl IntoIterator<Item = S>,
    searchable: bool,
) {
    let name = match choice {
        MobChoice::Kind => "kind",
        MobChoice::Attitude => "attitude",
        MobChoice::Gear => "gear",
    };
    let dropdown = Dropdown {
        options: options.into_iter().map(Into::into).collect(),
        searchable,
        ..default()
    };
    let node = Node {
        padding: UiRect {
            right: px(ARROW_ROOM),
            ..BUTTON_PADDING
        },
        ..default()
    };
    spawn_dropdown(line, dropdown, FONT, node).insert((choice, id(name)));
}
