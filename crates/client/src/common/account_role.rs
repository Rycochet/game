//! The account's role on the server played on (`shared::role`), and the wish
//! to open the Accounts panel.

use bevy::prelude::*;
use shared::role::{Permission, Powers, RoleBadge};

/// The account's role, as the server last said (`Welcome`, or a change while
/// playing): kept by `accounts_panel`, read by `system_menu` for its Accounts
/// entry. Only for showing the right menus: the server checks everything.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountRole {
    pub role: Option<RoleBadge>,
    pub powers: Powers,
}

impl AccountRole {
    /// Whether the Accounts panel has anything for it: finding characters,
    /// giving roles, renaming them or reading the audit log.
    pub fn has_accounts_panel(&self) -> bool {
        let powers = &self.powers;
        !powers.grants.is_empty()
            || [
                Permission::LookUp,
                Permission::ReadAudit,
                Permission::RenameRoles,
            ]
            .into_iter()
            .any(|permission| powers.may(permission))
    }
}

/// Opens the Accounts panel (the system menu's Accounts), or brings it to the front.
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct OpenAccounts;
