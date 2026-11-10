//! `roles.ron`: every role and which accounts hold one. Hand-edited only
//! while the server is off; read once at start and written on each change.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use shared::id::Id;
use shared::role::{
    ADMIN, BANNED, DEVELOPER, DebugTool, GUEST, MODERATED, OWNER, PLAYER, Permission, RoleId,
};
use uuid::Uuid;

use super::defaults::default_roles;
use crate::accounts::AccountId;

/// The file's layout: raised when an older file needs changing to be read.
pub(super) const FORMAT: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct RolesFile {
    pub format: u32,
    pub roles: Vec<RoleDef>,
    /// By account id (as written in the log). Players and guests are never here.
    #[serde(default)]
    pub accounts: BTreeMap<String, HeldRole>,
}

/// One role. The Owner's permissions are always all of them, whatever is written.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct RoleDef {
    pub id: RoleId,
    pub name: String,
    /// Shown in place of the name, if set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Every permission, on or off: any left out are filled in when read.
    #[serde(default)]
    pub permissions: BTreeMap<Permission, bool>,
    #[serde(default)]
    pub grants: BTreeSet<RoleId>,
    #[serde(default)]
    pub appears_as: BTreeSet<RoleId>,
}

/// An account's role.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct HeldRole {
    pub role: RoleId,
    /// When it was given (seconds since 1970).
    pub since: u64,
    /// When it ends, going back to Player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<u64>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reason: String,
    /// Who gave it: nobody for the server itself (a claim) or the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
}

impl RolesFile {
    pub fn new() -> Self {
        let mut file = Self {
            format: FORMAT,
            roles: Vec::new(),
            accounts: BTreeMap::new(),
        };
        file.fill_in();
        file
    }

    pub fn role(&self, id: &RoleId) -> Option<&RoleDef> {
        self.roles.iter().find(|role| role.id == *id)
    }

    /// Adds built-in roles and permissions the file doesn't have yet (a new
    /// version's), and gives the Owner everything: whether anything changed.
    pub fn fill_in(&mut self) -> bool {
        let before = self.clone();
        for role in &mut self.roles {
            take_old_debug_tools(role);
        }
        for default in default_roles() {
            match self.roles.iter_mut().find(|role| role.id == default.id) {
                Some(role) => {
                    for (permission, on) in default.permissions {
                        role.permissions.entry(permission).or_insert(on);
                    }
                }
                None => self.roles.push(default),
            }
        }
        for role in &mut self.roles {
            for permission in Permission::all() {
                role.permissions.entry(permission).or_insert(false);
            }
        }
        let others: BTreeSet<RoleId> = self
            .roles
            .iter()
            .map(|role| role.id.clone())
            .filter(|id| !id.is(OWNER) && !id.is(GUEST))
            .collect();
        let owner = self.roles.iter_mut().find(|role| role.id.is(OWNER));
        let owner = owner.expect("filled in above");
        owner.permissions = Permission::all().map(|p| (p, true)).collect();
        owner.appears_as = others
            .iter()
            .filter(|id| can_appear_as(id))
            .cloned()
            .collect();
        owner.grants = others;
        *self != before
    }

    /// What is wrong with the file, if anything (after `fill_in`).
    pub fn check(&self) -> Result<(), String> {
        if self.format != FORMAT {
            return Err(format!("format {} isn't {FORMAT}", self.format));
        }
        let mut seen = BTreeSet::new();
        for role in &self.roles {
            let id = &role.id.0;
            if !role.id.is_valid() {
                return Err(format!(
                    "the role id {id:?} isn't 1 to 24 lower case letters, digits, - and _"
                ));
            }
            if !seen.insert(&role.id) {
                return Err(format!("there are two roles with the id {id:?}"));
            }
            if role.name.trim().is_empty() {
                return Err(format!("the role {id:?} has no name"));
            }
        }
        for role in &self.roles {
            let given = role.grants.iter().filter(|id| id.is(OWNER) || id.is(GUEST));
            let shown = role.appears_as.iter().filter(|id| !can_appear_as(id));
            let unknown = role.grants.iter().chain(&role.appears_as);
            let unknown = unknown.filter(|id| self.role(id).is_none());
            if let Some(other) = given.chain(shown).chain(unknown).next() {
                return Err(format!(
                    "the role {:?} names {:?}, which isn't a role it can give or appear as",
                    role.id.0, other.0
                ));
            }
        }
        for (account, held) in &self.accounts {
            account_id(account).ok_or_else(|| format!("{account:?} isn't an account id"))?;
            let role = &held.role;
            if self.role(role).is_none() || role.is(PLAYER) || role.is(GUEST) {
                return Err(format!(
                    "the account {account} has the role {:?}, which no account can hold",
                    role.0
                ));
            }
        }
        Ok(())
    }

    /// Whether any account is the Owner.
    pub fn has_owner(&self) -> bool {
        self.accounts.values().any(|held| held.role.is(OWNER))
    }
}

/// Whether characters may be shown as this role: not as the Owner, a guest or
/// someone under sanctions.
/// A `roles.ron` from before each debug tool had its own permission has one,
/// `DebugTools`, for all of them. Where it differs from what that role had by
/// default (Owner, Admin and Developer had it), it is kept: on, every tool but
/// the Admin-only ones; off, none. Otherwise the new defaults follow.
fn take_old_debug_tools(role: &mut RoleDef) {
    let Some(on) = role.permissions.remove(&Permission::DebugTools) else {
        return;
    };
    let had_it = [OWNER, ADMIN, DEVELOPER].iter().any(|id| role.id.is(id));
    if on == had_it {
        return;
    }
    for tool in DebugTool::ALL {
        let given = on && !tool.admin_only();
        role.permissions.insert(Permission::Debug(tool), given);
    }
}

fn can_appear_as(id: &RoleId) -> bool {
    ![OWNER, GUEST, MODERATED, BANNED]
        .iter()
        .any(|not| id.is(not))
}

/// The account an id written in the file means.
pub(super) fn account_id(written: &str) -> Option<AccountId> {
    Uuid::parse_str(written.trim())
        .ok()
        .map(|uuid| AccountId(Id(uuid)))
}
