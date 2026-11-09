//! What the player does on the panel: tabs, typing, choosing and the buttons,
//! asking the server (`RoleAsk`) where it should.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, RoleAsk};

use super::audit::{NewerButton, OlderButton, PAGE};
use super::find::{ApplyButton, FindButton, GiveForList, GiveRoleList, NameBox, ReasonBox};
use super::roles::{RoleList, RoleNameBox, RoleTitleBox, SaveButton};
use super::state::{FOR, Panel, Tab};
use crate::connection::ToServer;
use crate::widgets::{ButtonPressed, DropdownChosen, TabChosen, TextBox, TextEdited};

/// Another tab: its page is built.
pub(super) fn choose_tab(
    mut chosen: MessageReader<TabChosen>,
    mut panel: ResMut<Panel>,
    mut to_server: MessageWriter<ToServer>,
) {
    for &TabChosen(row, index) in chosen.read() {
        if Some(row) == panel.tabs_row && panel.tab != index {
            panel.tab = index;
            panel.refused = None;
            panel.dirty = true;
            // The log may have grown since it was last asked for.
            if panel.tab() == Some(Tab::Audit) {
                let skip = panel.audit_skip;
                to_server.write(ToServer(ClientMessage::Role(RoleAsk::Audit { skip })));
            }
        }
    }
}

/// What is typed is kept, for the page being built again.
/// A text box, and which of the panel's it is.
type Typed<'a> = (
    &'a TextBox,
    Has<NameBox>,
    Has<ReasonBox>,
    Has<RoleNameBox>,
    Has<RoleTitleBox>,
);

pub(super) fn keep_typing(
    mut edited: MessageReader<TextEdited>,
    boxes: Query<Typed>,
    mut panel: ResMut<Panel>,
) {
    for edit in edited.read() {
        let Ok((text_box, name, reason, role_name, role_title)) = boxes.get(edit.0) else {
            continue;
        };
        let kept = match (name, reason, role_name, role_title) {
            (true, ..) => &mut panel.name,
            (_, true, ..) => &mut panel.reason,
            (_, _, true, _) => &mut panel.role_name,
            (.., true) => &mut panel.role_title,
            _ => continue,
        };
        kept.clone_from(&text_box.text);
    }
}

/// Choices from the dropdowns.
pub(super) fn keep_choices(
    mut chosen: MessageReader<DropdownChosen>,
    lists: Query<(Has<GiveRoleList>, Has<GiveForList>, Has<RoleList>)>,
    mut panel: ResMut<Panel>,
) {
    for &DropdownChosen(list, index) in chosen.read() {
        match lists.get(list) {
            Ok((true, _, _)) => panel.give_role = index,
            Ok((_, true, _)) => panel.give_for = index,
            Ok((_, _, true)) => {
                panel.role = index;
                panel.take_role_words();
                panel.dirty = true;
            }
            _ => {}
        }
    }
}

/// The buttons: Find, Apply, Save, Newer and Older.
#[allow(clippy::type_complexity)]
pub(super) fn press_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<(
        Has<FindButton>,
        Has<ApplyButton>,
        Has<SaveButton>,
        Has<NewerButton>,
        Has<OlderButton>,
    )>,
    mut panel: ResMut<Panel>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        let Ok(which) = buttons.get(press.0) else {
            continue;
        };
        let ask = match which {
            (true, ..) => RoleAsk::LookUp(panel.name.clone()),
            (_, true, ..) => {
                let Some(found) = &panel.found else {
                    continue;
                };
                let Some(role) = found.may_give.get(panel.give_role) else {
                    continue;
                };
                RoleAsk::Give {
                    character: found.character,
                    role: role.id.clone(),
                    minutes: FOR[panel.give_for].1,
                    reason: panel.reason.clone(),
                }
            }
            (_, _, true, ..) => {
                let Some(role) = panel.roles.get(panel.role) else {
                    continue;
                };
                let title = Some(panel.role_title.clone()).filter(|t| !t.trim().is_empty());
                RoleAsk::Rename {
                    role: role.id.clone(),
                    name: panel.role_name.clone(),
                    title,
                }
            }
            (.., true, _) => {
                panel.audit_skip = panel.audit_skip.saturating_sub(PAGE);
                RoleAsk::Audit {
                    skip: panel.audit_skip,
                }
            }
            (.., true) => {
                panel.audit_skip += PAGE;
                RoleAsk::Audit {
                    skip: panel.audit_skip,
                }
            }
            _ => continue,
        };
        to_server.write(ToServer(ClientMessage::Role(ask)));
    }
}
