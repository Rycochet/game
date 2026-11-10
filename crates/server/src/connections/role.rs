//! A client's role: what its account may do here, checked on everything asked
//! of the server beyond playing, and the role its character is shown as.

use std::collections::BTreeSet;

use shared::role::{
    DebugTool, PLAYER, Permission, Powers, RoleBadge, RoleId, ShownRole, is_marked,
};

use super::Client;
use crate::ServerOptions;
use crate::roles::{Held, Roles};

/// Its account's role, once logged in, and what its character is shown as.
#[derive(Default)]
pub(crate) struct RoleState {
    held: Option<Held>,
    shown: ShownRole,
}

impl Client {
    /// Its account's role now (on logging in, or the host's on `Hello`), shown
    /// as itself until the character's choice is loaded (`show_saved`).
    pub(crate) fn take_role(&mut self, held: Held, roles: &Roles) {
        self.role.shown = ShownRole {
            current: held.badge.id.clone(),
            choices: roles.choices(&held),
        };
        self.role.held = Some(held);
    }

    /// Whether it may do this: the host (the embedded server's own player, its
    /// Owner) always, anyone else by their role.
    pub(crate) fn may(&self, permission: Permission) -> bool {
        self.host
            || self
                .role
                .held
                .as_ref()
                .is_some_and(|held| held.powers.may(permission))
    }

    /// Whether it may use this debug tool: by its role, or on a server that
    /// gives them to everyone (`HOW_DEBUG_TOOLS`), all but the Admin-only ones.
    pub(super) fn may_debug(&self, tool: DebugTool, options: &ServerOptions) -> bool {
        (options.dev_tools && !tool.admin_only()) || self.may(Permission::Debug(tool))
    }

    /// Every debug tool it may use, as `Welcome` tells it.
    pub(super) fn debug_tools(&self, options: &ServerOptions) -> BTreeSet<DebugTool> {
        DebugTool::ALL
            .into_iter()
            .filter(|&tool| self.may_debug(tool, options))
            .collect()
    }

    /// What it is told of its role.
    pub(super) fn badge(&self) -> RoleBadge {
        match &self.role.held {
            Some(held) => held.badge.clone(),
            None => RoleBadge {
                id: RoleId::new(PLAYER),
                name: "Player".into(),
                title: None,
            },
        }
    }

    pub(super) fn powers(&self) -> Powers {
        self.role
            .held
            .as_ref()
            .map(|held| held.powers.clone())
            .unwrap_or_default()
    }

    /// What its character is shown as, and may be.
    pub(super) fn shown(&self) -> ShownRole {
        self.role.shown.clone()
    }

    /// The character's saved choice, if it may still be shown as that (its
    /// role may have changed since); otherwise its own role.
    pub(super) fn show_saved(&mut self, saved: Option<RoleId>) {
        let own = self.role.shown.choices.first().map(|own| own.id.clone());
        let allowed = saved.filter(|id| self.role.shown.choices.iter().any(|b| b.id == *id));
        if let Some(current) = allowed.or(own) {
            self.role.shown.current = current;
        }
    }

    /// Shows the character as this role, if it may be: whether it was.
    pub(super) fn show_as(&mut self, id: RoleId) -> bool {
        let allowed = self.role.shown.choices.iter().any(|badge| badge.id == id);
        if allowed {
            self.role.shown.current = id;
            self.unsaved = true;
        }
        allowed
    }

    /// The role saved with the character, once it has one to choose from.
    pub(super) fn shown_to_save(&self) -> Option<RoleId> {
        let chosen = !self.role.shown.choices.is_empty();
        chosen.then(|| self.role.shown.current.clone())
    }

    /// The badge others see before its character's name, if it is shown as a
    /// marked role.
    pub(crate) fn mark(&self) -> Option<RoleBadge> {
        let badge = self.role.shown.badge()?;
        is_marked(&badge.id).then(|| badge.clone())
    }
}
