//! Settings' Controls page: every action under its group's heading, in rows of
//! its name and two key boxes (owner: a table, so names' lengths don't matter,
//! with every other row tinted; no header). Esc's row is shown but can't be
//! changed. A binding chosen for one action leaves any other that had it, whose
//! box is then marked, with a tip saying who took it, until Settings closes.

use std::collections::HashMap;

use bevy::prelude::*;

use super::rows::{spawn_section, spawn_sections, tip};
use super::{FONT, LABEL_GAP, ROW_HEIGHT};
use crate::common::controls::SLOTS;
use crate::common::{Action, Binding, Controls, DebugTools, Group, IN_BROWSER, Mods, Preferences};
use crate::widgets::{
    ButtonPressed, ButtonState, HoverTip, INK, KeyBox, KeyChosen, ROW_TINT, UiId, spawn_button,
    spawn_key_box,
};

const FORM: &str = "settings.controls";
const KEY_WIDTH: f32 = 200.0;
const ROW_PADDING: f32 = 4.0;
const ESC: &str = "Esc always opens the menu and closes what is on top, so it can't be changed.";

#[derive(Component)]
pub(in crate::system_menu) struct ControlsPage;

/// A key box: whose, and which of its two.
#[derive(Component, Clone, Copy)]
pub(in crate::system_menu) struct ControlBox(Action, usize);

#[derive(Component)]
pub(in crate::system_menu) struct ResetControls;

/// Boxes whose binding was just given to another action, and to which.
#[derive(Resource, Default, Clone, PartialEq)]
pub(in crate::system_menu) struct TakenKeys(HashMap<(Action, usize), (Action, Binding)>);

/// Builds the page again when the bindings, or what was taken, change (and at
/// the start).
pub(in crate::system_menu) fn fill_controls_page(
    mut commands: Commands,
    preferences: Res<Preferences>,
    taken: Res<TakenKeys>,
    page: Single<Entity, With<ControlsPage>>,
    tools: Res<DebugTools>,
    role: Res<crate::common::AccountRole>,
    mut shown: Local<Option<(Controls, TakenKeys, bool)>>,
) {
    // No debug menu without anything in it (owner).
    let debug_menu = tools.any() || role.has_accounts_panel();
    let now = (preferences.controls.clone(), taken.clone(), debug_menu);
    if shown.as_ref() == Some(&now) {
        return;
    }
    *shown = Some(now);
    let page = *page;
    commands.entity(page).despawn_related::<Children>();
    commands.entity(page).with_children(|page| {
        spawn_sections(page, UiId::new(FORM, "scroll"), |sections| {
            for group in Group::ALL {
                spawn_section(sections, group.name(), |section| {
                    let actions: Vec<Action> = Action::ALL
                        .into_iter()
                        .filter(|action| action.group() == group)
                        .filter(|&action| !(IN_BROWSER && action == Action::FullScreen))
                        .filter(|&action| debug_menu || action != Action::DebugMenu)
                        .collect();
                    for (row, &action) in actions.iter().enumerate() {
                        spawn_action_row(section, row, action, &preferences.controls, &taken);
                    }
                    if group == Group::Interface {
                        spawn_escape_row(section, actions.len());
                    }
                });
            }
            sections
                .spawn(Node {
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                })
                .with_children(|bar| {
                    spawn_button(
                        bar,
                        ButtonState::Normal,
                        "Reset to Defaults",
                        FONT,
                        default(),
                    )
                    .insert((ResetControls, UiId::new(FORM, "reset")));
                });
        });
    });
}

/// A row's node: the whole width, tinted if it is an odd one.
fn row_node(row: usize) -> impl Bundle {
    (
        Node {
            column_gap: px(LABEL_GAP),
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(ROW_PADDING * 2.0), px(ROW_PADDING)),
            ..default()
        },
        BackgroundColor(if row % 2 == 1 { ROW_TINT } else { Color::NONE }),
    )
}

/// The name, taking what is left, so the boxes line up as columns.
fn name(label: &str) -> impl Bundle {
    (
        Text::new(label),
        FONT.font(),
        TextColor(INK),
        Node {
            flex_grow: 1.0,
            ..default()
        },
    )
}

fn box_node() -> Node {
    Node {
        width: px(KEY_WIDTH),
        height: px(ROW_HEIGHT),
        flex_shrink: 0.0,
        ..default()
    }
}

fn spawn_action_row(
    section: &mut ChildSpawnerCommands,
    row: usize,
    action: Action,
    controls: &Controls,
    taken: &TakenKeys,
) {
    section.spawn(row_node(row)).with_children(|row| {
        row.spawn(name(action.name()));
        for (slot, binding) in controls.keys(action).into_iter().enumerate() {
            let lost = taken.0.get(&(action, slot));
            let key_box = KeyBox {
                binding,
                warned: lost.is_some(),
            };
            let mut made = spawn_key_box(row, key_box, ButtonState::Normal, FONT, box_node());
            made.insert((
                ControlBox(action, slot),
                UiId::new(FORM, action.id()).at(slot),
            ));
            if let Some((by, binding)) = lost {
                let words = format!("{} is now used by {}.", binding.label(), by.name());
                made.insert(HoverTip(words));
            }
        }
    });
}

/// Esc, which nothing else can have.
fn spawn_escape_row(section: &mut ChildSpawnerCommands, row: usize) {
    section
        .spawn((row_node(row), tip(ESC, None)))
        .with_children(|row| {
            row.spawn(name("Menu / Back"));
            let esc = KeyBox {
                binding: Some(Binding::new(KeyCode::Escape, Mods::NONE)),
                warned: false,
            };
            for (slot, key_box) in [esc, KeyBox::default()].into_iter().enumerate() {
                spawn_key_box(row, key_box, ButtonState::Disabled, FONT, box_node())
                    .insert(UiId::new(FORM, "menu").at(slot));
            }
        });
    debug_assert_eq!(SLOTS, 2, "Esc's row has two boxes");
}

/// Puts chosen bindings into the preferences, and Reset to Defaults.
pub(in crate::system_menu) fn use_controls_page(
    mut chosen: MessageReader<KeyChosen>,
    mut pressed: MessageReader<ButtonPressed>,
    boxes: Query<&ControlBox>,
    resets: Query<(), With<ResetControls>>,
    mut preferences: ResMut<Preferences>,
    mut taken: ResMut<TakenKeys>,
) {
    for &KeyChosen(entity, binding) in chosen.read() {
        let Ok(&ControlBox(action, slot)) = boxes.get(entity) else {
            continue;
        };
        if preferences.controls.keys(action)[slot] == binding {
            taken.0.remove(&(action, slot));
            continue;
        }
        taken.0.remove(&(action, slot));
        for lost in preferences.controls.set(action, slot, binding) {
            if let Some(binding) = binding
                && lost.0 != action
            {
                taken.0.insert(lost, (action, binding));
            }
        }
    }
    if pressed.read().any(|press| resets.contains(press.0)) {
        preferences.controls = Controls::default();
        taken.0.clear();
    }
}

/// Forgets what was taken once Settings closes.
pub(in crate::system_menu) fn forget_taken(
    menu: Res<super::super::SystemMenu>,
    mut taken: ResMut<TakenKeys>,
) {
    let in_settings = matches!(
        *menu,
        super::super::SystemMenu::Settings
            | super::super::SystemMenu::KeepGraphics
            | super::super::SystemMenu::Ironman
    );
    if !in_settings && !taken.0.is_empty() {
        taken.0.clear();
    }
}
