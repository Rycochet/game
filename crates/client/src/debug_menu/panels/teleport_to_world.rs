//! The Teleport to World panel (owner's design, 2026-11-09): a world's seed
//! (blank for the one the player is in, or in the Hall the one they came
//! from), a choice of its World Gates, and buttons going there, to the Hall of
//! Worlds beside that gate's way in, or showing that planet. It takes the place
//! of Go to World, Teleport to Gate and Teleport to Hall of Worlds.

use bevy::prelude::*;
use shared::world_gate::WORLD_GATES;

use super::{CHECK_ROW, FONT, SEED_WIDTH, row, words};
use crate::widgets::{
    BUTTON_PADDING, ButtonState, ButtonStyle, Pressable, TextBox, TextSize, UiId, spawn_button,
    spawn_new_seed_button, spawn_text_box,
};

/// The panel's form, for the debug API.
pub(super) const FORM: &str = "debug.teleport_to_world";

const GATE_PADDING: UiRect = UiRect {
    left: Val::Px(6.0),
    right: Val::Px(6.0),
    top: Val::Px(2.0),
    bottom: Val::Px(2.0),
};

/// What the panel has chosen: the gate (none: Any), and a seed to put in its
/// box once it is there (when View Planet opened it).
#[derive(Resource, Default)]
pub(in crate::debug_menu) struct TeleportForm {
    pub(super) gate: Option<u8>,
    pub(super) seed_to_show: Option<String>,
}

/// The box with the world's seed.
#[derive(Component)]
pub(in crate::debug_menu) struct WorldSeedBox;

/// What the panel's other buttons do.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(in crate::debug_menu) enum TeleportButton {
    RandomSeed,
    /// A World Gate (from 0; it says one more), or none for Any.
    Gate(Option<u8>),
    ViewPlanet,
    Hall,
    Go,
}

pub(super) fn spawn_teleport_to_world(content: &mut ChildSpawnerCommands) {
    content.spawn(row()).with_children(|row| {
        row.spawn(words("Seed"));
        let text_box = TextBox {
            digits_only: true,
            takes_keys: true,
            ..default()
        };
        let node = Node {
            width: px(SEED_WIDTH),
            height: px(CHECK_ROW),
            ..default()
        };
        spawn_text_box(row, text_box, FONT, node).insert((WorldSeedBox, UiId::new(FORM, "seed")));
        spawn_new_seed_button(row, CHECK_ROW)
            .insert((TeleportButton::RandomSeed, UiId::new(FORM, "new_seed")));
        row.spawn(words("Blank: this world"));
    });
    content.spawn(row()).with_children(|row| {
        row.spawn(words("Gate"));
        let node = Node {
            height: px(CHECK_ROW),
            padding: GATE_PADDING,
            ..default()
        };
        spawn_button(
            row,
            ButtonState::Normal,
            "Any",
            TextSize::Small,
            node.clone(),
        )
        .insert((
            TeleportButton::Gate(None),
            Pressable,
            UiId::new(FORM, "any"),
        ));
        for gate in 0..WORLD_GATES as u8 {
            let label = (gate + 1).to_string();
            spawn_button(
                row,
                ButtonState::Normal,
                label,
                TextSize::Small,
                node.clone(),
            )
            .insert((
                TeleportButton::Gate(Some(gate)),
                Pressable,
                UiId::new(FORM, "gate").at(usize::from(gate)),
            ));
        }
    });
    content.spawn(row()).with_children(|row| {
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        for (label, button, name) in [
            ("View Planet...", TeleportButton::ViewPlanet, "view_planet"),
            ("Hall of Worlds", TeleportButton::Hall, "hall"),
            ("Go", TeleportButton::Go, "go"),
        ] {
            let mut spawned = spawn_button(row, ButtonState::Normal, label, FONT, node.clone());
            spawned.insert((button, Pressable, UiId::new(FORM, name)));
            if button == TeleportButton::Go {
                spawned.insert(ButtonStyle::Main);
            }
        }
    });
}

/// The seed in the box: none when it is blank, and `Err` when it isn't a
/// number.
pub(super) fn seed_in(text: &str) -> Result<Option<u64>, ()> {
    let text = text.trim();
    if text.is_empty() {
        Ok(None)
    } else {
        text.parse().map(Some).map_err(|_| ())
    }
}
