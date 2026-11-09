//! Messages about roles (`crate::role`): what a character is shown as, and the
//! Accounts panel's: finding characters, giving roles, renaming them and
//! reading the audit log. The server checks every one against the asker's
//! role.

use serde::{Deserialize, Serialize};

use super::CharacterId;
use crate::role::{Powers, RoleBadge, RoleId, ShownRole};

/// From a client (`ClientMessage::Role`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleAsk {
    /// Show the character as this role, one it may be (answered with `Shown`).
    ShowAs(RoleId),
    /// Find a character by name, whatever its capitals (`Permission::LookUp`):
    /// answered with `Found` or `Refused`.
    LookUp(String),
    /// Give this character's account a role (Player takes its role away), for
    /// `minutes` or for good, with a reason for the audit log.
    Give {
        character: CharacterId,
        role: RoleId,
        minutes: Option<u32>,
        reason: String,
    },
    /// The audit log, newest first, skipping `skip` lines
    /// (`Permission::ReadAudit`).
    Audit { skip: u32 },
    /// Every role, to rename (`Permission::RenameRoles`).
    Roles,
    /// A role's new name and title (`Permission::RenameRoles`).
    Rename {
        role: RoleId,
        name: String,
        title: Option<String>,
    },
}

/// To a client (`ServerMessage::Role`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleAnswer {
    /// What the character is now shown as (after `ShowAs`).
    Shown(ShownRole),
    /// The account's role changed while playing: what it now is, may do, and
    /// shows as.
    Changed {
        role: RoleBadge,
        powers: Powers,
        shown: ShownRole,
    },
    Found(FoundAccount),
    /// Lines of the audit log, newest first, and whether there are older.
    Audit {
        lines: Vec<String>,
        more: bool,
    },
    Roles(Vec<RoleBadge>),
    /// Why it couldn't be.
    Refused(String),
}

/// A character found by name, and its account's role.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoundAccount {
    pub character: CharacterId,
    pub name: String,
    /// The account's other characters' names.
    pub others: Vec<String>,
    pub guest: bool,
    /// The asker's own account (whose role it can't change).
    pub yours: bool,
    pub role: RoleBadge,
    /// When it ends, as the server writes times (UTC).
    pub until: Option<String>,
    pub reason: String,
    /// The roles the asker may give this account, if it may change its role
    /// at all (empty if not).
    pub may_give: Vec<RoleBadge>,
}
