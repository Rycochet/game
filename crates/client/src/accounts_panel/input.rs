//! What the player does on the panel: tabs, typing, choosing and the buttons,
//! asking the server (`RoleAsk`) where it should.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, RoleAsk};

use super::audit::PAGE;
use super::state::{FOR, Panel, Tab};
use crate::connection::ToServer;
use crate::dom::{UiEvent, UiValue};

pub(super) fn read_events(
    mut events: MessageReader<UiEvent>,
    mut panel: ResMut<Panel>,
    mut to_server: MessageWriter<ToServer>,
) {
    for event in events.read() {
        let ("accounts", name) = event.id.parts() else {
            continue;
        };
        let ask = match &event.value {
            UiValue::Text(words) => {
                typed(&mut panel, name, words);
                None
            }
            UiValue::Choice { index, .. } => chosen(&mut panel, name, *index),
            UiValue::Pressed => pressed(&mut panel, name),
            _ => None,
        };
        if let Some(ask) = ask {
            to_server.write(ToServer(ClientMessage::Role(ask)));
        }
    }
}

/// What is typed is kept, for asking with.
fn typed(panel: &mut Panel, name: &str, words: &str) {
    let kept = match name {
        "name" => &mut panel.name,
        "reason" => &mut panel.reason,
        "role_name" => &mut panel.role_name,
        "role_title" => &mut panel.role_title,
        _ => return,
    };
    words.clone_into(kept);
}

/// Another tab, or a choice from a dropdown.
fn chosen(panel: &mut Panel, name: &str, index: usize) -> Option<RoleAsk> {
    match name {
        "tabs" if panel.tab != index => {
            panel.tab = index;
            panel.refused = None;
            panel.dirty = true;
            // The log may have grown since it was last asked for.
            let skip = panel.audit_skip;
            return (panel.tab() == Some(Tab::Audit)).then_some(RoleAsk::Audit { skip });
        }
        "give_role" => panel.give_role = index,
        "give_for" => panel.give_for = index,
        "role" => {
            panel.role = index;
            panel.take_role_words();
            panel.dirty = true;
        }
        _ => {}
    }
    None
}

/// The buttons: Find, Apply, Save, Newer and Older.
fn pressed(panel: &mut Panel, name: &str) -> Option<RoleAsk> {
    Some(match name {
        "find" => RoleAsk::LookUp(panel.name.clone()),
        "apply" => {
            let found = panel.found.as_ref()?;
            let role = found.may_give.get(panel.give_role)?;
            RoleAsk::Give {
                character: found.character,
                role: role.id.clone(),
                minutes: FOR.get(panel.give_for)?.1,
                reason: panel.reason.clone(),
            }
        }
        "save" => {
            let role = panel.roles.get(panel.role)?;
            let title = Some(panel.role_title.clone()).filter(|t| !t.trim().is_empty());
            RoleAsk::Rename {
                role: role.id.clone(),
                name: panel.role_name.clone(),
                title,
            }
        }
        "newer" => {
            panel.audit_skip = panel.audit_skip.saturating_sub(PAGE);
            RoleAsk::Audit {
                skip: panel.audit_skip,
            }
        }
        "older" => {
            panel.audit_skip += PAGE;
            RoleAsk::Audit {
                skip: panel.audit_skip,
            }
        }
        _ => return None,
    })
}
