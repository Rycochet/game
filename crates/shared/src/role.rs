//! Account roles: what an account may do on a server beyond playing, and what
//! others are shown of it. The server keeps which account has which role (in
//! its data folder's `roles.ron`) and checks every permission; a client is only
//! told its own, to show the right menus. See "Roles" in
//! `docs/context/architecture.md`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// A role's id: one of the built-in ones (`OWNER` to `GUEST`) or a server's
/// own, in lower case letters, digits, `-` and `_`.
// Written as a plain string, so `roles.ron` reads `"admin"`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoleId(pub String);

/// The server's owner: everything, always. The embedded server's own player,
/// or whoever claimed a networked server.
pub const OWNER: &str = "owner";
/// Everything but making Owners and Admins.
pub const ADMIN: &str = "admin";
/// The debug tools, and nothing of moderation or accounts.
pub const DEVELOPER: &str = "developer";
/// Moderation, and nothing of the debug tools.
pub const MODERATOR: &str = "moderator";
/// Everyone with an account and no other role: never kept in the file.
pub const PLAYER: &str = "player";
/// A player under sanctions, usually for a time.
pub const MODERATED: &str = "moderated";
/// Can't log in, usually for a time.
pub const BANNED: &str = "banned";
/// Everyone playing as a guest: never given or taken.
pub const GUEST: &str = "guest";

/// Every built-in role, highest first.
pub const BUILT_IN: [&str; 8] = [
    OWNER, ADMIN, DEVELOPER, MODERATOR, PLAYER, MODERATED, BANNED, GUEST,
];

impl RoleId {
    pub fn new(id: &str) -> Self {
        Self(id.to_owned())
    }

    pub fn is(&self, id: &str) -> bool {
        self.0 == id
    }

    /// Whether this is a fit id for a role: 1 to 24 lower case letters,
    /// digits, `-` and `_`.
    pub fn is_valid(&self) -> bool {
        (1..=24).contains(&self.0.len())
            && self
                .0
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    }
}

/// Something a role may do, beyond playing. Granting roles and appearing as
/// them are in `Powers`, by role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// The debug menu's tools (one permission for now).
    DebugTools,
    /// The server's own settings, such as the difficulties allowed.
    ServerOptions,
    /// Stopping the clock, where the server can be paused.
    Pause,
    /// Never sent away for doing nothing.
    NeverIdle,
    /// Let in when the server is full.
    PastPlayerLimit,
    /// Finding characters and their accounts by name.
    LookUp,
    /// Reading the audit log of roles given and taken.
    ReadAudit,
    /// Changing roles' names and titles.
    RenameRoles,
}

impl Permission {
    pub const ALL: [Permission; 8] = [
        Self::DebugTools,
        Self::ServerOptions,
        Self::Pause,
        Self::NeverIdle,
        Self::PastPlayerLimit,
        Self::LookUp,
        Self::ReadAudit,
        Self::RenameRoles,
    ];
}

/// What a role lets its account do.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Powers {
    pub permissions: BTreeSet<Permission>,
    /// The roles it may give others (to accounts it may act on: those holding
    /// one of these). Player here means taking a role away.
    pub grants: BTreeSet<RoleId>,
    /// The roles its characters may be shown as instead of its own.
    pub appears_as: BTreeSet<RoleId>,
}

impl Powers {
    pub fn may(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }
}

/// What is shown of a role: its id, name and title, if it has one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleBadge {
    pub id: RoleId,
    pub name: String,
    pub title: Option<String>,
}

impl RoleBadge {
    /// What it is called: its title, or else its name.
    pub fn shown(&self) -> &str {
        self.title.as_deref().unwrap_or(&self.name)
    }
}

#[cfg(test)]
mod tests;
