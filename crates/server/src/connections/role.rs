//! A client's role: what its account may do here, checked on everything asked
//! of the server beyond playing.

use shared::role::{PLAYER, Permission, Powers, RoleBadge, RoleId};

use super::Client;
use crate::ServerOptions;
use crate::roles::Held;

impl Client {
    /// Its account's role now (on logging in, or the host's on `Hello`).
    pub(super) fn take_role(&mut self, held: Held) {
        self.held = Some(held);
    }

    /// Whether it may do this: the host (the embedded server's own player, its
    /// Owner) always, anyone else by their role.
    pub(crate) fn may(&self, permission: Permission) -> bool {
        self.host
            || self
                .held
                .as_ref()
                .is_some_and(|held| held.powers.may(permission))
    }

    /// Whether it may use the debug tools: by its role, or on a server that
    /// gives them to everyone (`HOW_DEBUG_TOOLS`).
    pub(super) fn may_debug(&self, options: &ServerOptions) -> bool {
        options.dev_tools || self.may(Permission::DebugTools)
    }

    /// What it is told of its role.
    pub(super) fn badge(&self) -> RoleBadge {
        match &self.held {
            Some(held) => held.badge.clone(),
            None => RoleBadge {
                id: RoleId::new(PLAYER),
                name: "Player".into(),
                title: None,
            },
        }
    }

    pub(super) fn powers(&self) -> Powers {
        self.held
            .as_ref()
            .map(|held| held.powers.clone())
            .unwrap_or_default()
    }
}
