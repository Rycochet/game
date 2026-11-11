//! Settings' Controls page: every action under its group's heading, in rows of
//! its name and two key boxes (owner: a table, so names' lengths don't matter,
//! with every other row tinted; no header). The groups and rows are copies of
//! templates in Settings' markup (`shell`), made and filled here. Esc's row is shown but can't be
//! changed. A binding chosen for one action leaves any other that had it, whose
//! box is then marked, with a tip saying who took it, until Settings closes.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::common::controls::SLOTS;
use crate::common::{Action, Binding, Controls, DebugTools, Group, IN_BROWSER, Preferences};
use crate::dom::Dom;
use crate::widgets::{ButtonPressed, KeyChosen, UiId};

const ESC: &str = "Esc always opens the menu and closes what is on top, so it can't be changed.";
/// A row's two key boxes, by slot. Each row's number is its action's place
/// in `Action::ALL`: `settings.controls/first.3`.
const BOXES: [&str; SLOTS] = ["first", "second"];
/// Esc's row: the number after the last action's.
const ESC_ROW: usize = Action::ALL.len();

/// Where the groups go, once the page is built.
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

/// What a built part of the page is known by, from its id.
pub(super) fn bind_controls(bound: &mut EntityCommands, id: &UiId) {
    let slot = BOXES.iter().position(|name| *name == id.parts().1);
    let action = id.index().and_then(|row| Action::ALL.get(row));
    match (id.parts().1, slot, action) {
        ("groups", ..) => bound.insert(ControlsPage),
        ("reset", ..) => bound.insert(ResetControls),
        (_, Some(slot), Some(&action)) => bound.insert(ControlBox(action, slot)),
        _ => bound,
    };
}

/// The actions with a row, in order: no debug menu without anything in it
/// (owner), and no full screen in a browser.
fn listed(debug_menu: bool) -> impl Iterator<Item = Action> {
    Action::ALL
        .into_iter()
        .filter(|&action| !(IN_BROWSER && action == Action::FullScreen))
        .filter(move |&action| debug_menu || action != Action::DebugMenu)
}

/// Makes the rows when which there are changes (and once the page is built),
/// then shows the bindings, and what was taken, when they change.
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn show_controls_page(
    preferences: Res<Preferences>,
    taken: Res<TakenKeys>,
    tools: Res<DebugTools>,
    role: Res<crate::common::AccountRole>,
    page: Query<(), With<ControlsPage>>,
    mut dom: Dom,
    mut rows: Local<Option<bool>>,
    mut shown: Local<Option<(Controls, TakenKeys)>>,
) {
    if page.is_empty() {
        return;
    }
    let debug_menu = tools.any() || role.has_accounts_panel();
    if *rows != Some(debug_menu) {
        *rows = Some(debug_menu);
        *shown = None;
        make_rows(&mut dom, debug_menu);
    }
    let now = (preferences.controls.clone(), taken.clone());
    if shown.as_ref() == Some(&now) {
        return;
    }
    for action in listed(debug_menu) {
        for (slot, binding) in preferences.controls.keys(action).into_iter().enumerate() {
            let id = format!("settings.controls/{}.{}", BOXES[slot], action.index());
            match binding {
                Some(binding) => dom.set(&id, "value", &binding.to_name()),
                None => dom.clear(&id, "value"),
            }
            // Marked, with who took it, until Settings closes.
            match taken.0.get(&(action, slot)) {
                Some((by, binding)) => {
                    let words = format!("{} is now used by {}.", binding.label(), by.name());
                    dom.set(&id, "class", "warned");
                    dom.set(&id, "title", &words);
                }
                None => {
                    dom.clear(&id, "class");
                    dom.clear(&id, "title");
                }
            }
        }
    }
    *shown = Some(now);
}

/// Every group under its heading, with a row for each of its actions, and
/// Esc's row, which nothing else can have, at the end of Interface's.
fn make_rows(dom: &mut Dom, debug_menu: bool) {
    const FORM: &str = "settings.controls";
    dom.empty("settings.controls/groups");
    for (at, group) in Group::ALL.into_iter().enumerate() {
        dom.copy_template("settings.controls/group", "settings.controls/groups", at);
        dom.set_text(&format!("{FORM}/heading.{at}"), group.name());
        let table = format!("{FORM}/keys.{at}");
        for action in listed(debug_menu).filter(|action| action.group() == group) {
            dom.copy_template("settings.controls/row", &table, action.index());
            dom.set_text(&format!("{FORM}/name.{}", action.index()), action.name());
        }
        if group == Group::Interface {
            dom.copy_template("settings.controls/row", &table, ESC_ROW);
            dom.set_text(&format!("{FORM}/name.{ESC_ROW}"), "Menu / Back");
            dom.set(&format!("{FORM}/first.{ESC_ROW}"), "value", "Escape");
            for part in ["name", "first", "second"] {
                dom.set(&format!("{FORM}/{part}.{ESC_ROW}"), "title", ESC);
            }
            for key_box in BOXES {
                dom.set_flag(&format!("{FORM}/{key_box}.{ESC_ROW}"), "disabled", true);
            }
        }
    }
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
