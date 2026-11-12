//! What the Accounts panel shows and has been typed into it, kept while its
//! body is built again after each answer.

use bevy::prelude::*;
use shared::protocol::FoundAccount;
use shared::role::{Permission, Powers, RoleBadge};

/// The form of the widgets still made by code (`widgets::UiId`), the same as
/// the panel's markup.
pub(super) const FORM: &str = "accounts";

/// How long a role is given for: the dropdown's words and minutes (none: for good).
pub(super) const FOR: [(&str, Option<u32>); 5] = [
    ("Always", None),
    ("An hour", Some(60)),
    ("A day", Some(24 * 60)),
    ("A week", Some(7 * 24 * 60)),
    ("30 days", Some(30 * 24 * 60)),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tab {
    /// Finding a character and giving their account a role.
    Find,
    /// Renaming roles.
    Roles,
    /// The audit log.
    Audit,
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Self::Find => "Find",
            Self::Roles => "Roles",
            Self::Audit => "Audit Log",
        }
    }
}

/// The tabs a role has something on.
pub(super) fn tabs_for(powers: &Powers) -> Vec<Tab> {
    [
        (Tab::Find, Permission::LookUp),
        (Tab::Roles, Permission::RenameRoles),
        (Tab::Audit, Permission::ReadAudit),
    ]
    .into_iter()
    .filter(|(_, permission)| powers.may(*permission))
    .map(|(tab, _)| tab)
    .collect()
}

/// The open panel, if any, and everything it shows.
#[derive(Resource, Default)]
pub(super) struct Panel {
    /// Its document, from when it is opened.
    pub document: Option<Entity>,
    /// The panel itself and its page, once built.
    pub root: Option<Entity>,
    pub body: Option<Entity>,
    pub tabs: Vec<Tab>,
    pub tab: usize,
    /// Build the body again.
    pub dirty: bool,
    /// Why the last thing asked was refused.
    pub refused: Option<String>,
    // Find.
    pub name: String,
    pub found: Option<FoundAccount>,
    pub give_role: usize,
    pub give_for: usize,
    pub reason: String,
    // Roles.
    pub roles: Vec<RoleBadge>,
    pub role: usize,
    pub role_name: String,
    pub role_title: String,
    // Audit log.
    pub audit: Vec<String>,
    pub audit_more: bool,
    pub audit_skip: u32,
}

impl Panel {
    pub fn tab(&self) -> Option<Tab> {
        self.tabs.get(self.tab).copied()
    }

    /// The chosen role's name and title into the boxes.
    pub fn take_role_words(&mut self) {
        if let Some(role) = self.roles.get(self.role) {
            self.role_name.clone_from(&role.name);
            self.role_title = role.title.clone().unwrap_or_default();
        }
    }
}
