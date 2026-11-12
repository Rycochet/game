//! The server's answers: the account's role (for `common::AccountRole`), and
//! what the panel asked.

use bevy::prelude::*;
use shared::protocol::{RoleAnswer, ServerMessage};

use super::state::Panel;
use crate::common::AccountRole;
use crate::connection::FromServer;

pub(super) fn read_answers(
    mut from_server: MessageReader<FromServer>,
    mut role: ResMut<AccountRole>,
    mut panel: ResMut<Panel>,
) {
    for FromServer(message) in from_server.read() {
        let answer = match message {
            ServerMessage::Welcome {
                role: badge,
                powers,
                ..
            }
            | ServerMessage::Role(RoleAnswer::Changed {
                role: badge,
                powers,
                ..
            }) => {
                role.set_if_neq(AccountRole {
                    role: Some(badge.clone()),
                    powers: powers.clone(),
                });
                continue;
            }
            ServerMessage::Role(answer) => answer,
            _ => continue,
        };
        let panel = &mut *panel;
        panel.dirty = true;
        panel.refused = None;
        match answer {
            RoleAnswer::Found(found) => {
                // The role it has now, if it may be given, so Apply does nothing new.
                let now = found.may_give.iter().position(|b| b.id == found.role.id);
                panel.give_role = now.unwrap_or(0);
                panel.found = Some(found.clone());
                panel.new_found = true;
            }
            RoleAnswer::Audit { lines, more } => {
                panel.audit.clone_from(lines);
                panel.audit_more = *more;
            }
            RoleAnswer::Roles(roles) => {
                panel.roles.clone_from(roles);
                panel.new_roles = true;
                panel.role = panel.role.min(roles.len().saturating_sub(1));
                panel.take_role_words();
            }
            RoleAnswer::Refused(why) => panel.refused = Some(why.clone()),
            RoleAnswer::Shown(_) | RoleAnswer::Changed { .. } => panel.dirty = false,
        }
    }
}
