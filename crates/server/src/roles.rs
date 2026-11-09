//! Account roles (see `shared::role` and "Roles" in
//! `docs/context/architecture.md`): which account holds which role, kept in
//! the data folder's `roles.ron` (`file`, `disk`), with every change written to
//! the audit log (`audit`), and a networked server with no Owner claimed by a
//! code (`claim`).

use std::path::Path;

use shared::role::{GUEST, OWNER, PLAYER, Powers, RoleBadge, RoleId};

use crate::accounts::AccountId;

mod audit;
mod claim;
mod defaults;
mod disk;
mod file;
mod panel;
mod set_role;
mod stamp;

use audit::Audit;
use disk::Disk;
use file::{HeldRole, RolesFile};
pub(crate) use set_role::{Actor, Change};
pub(crate) use stamp::log_stamp;

/// Every role and who holds one. Without a data folder (tests), only in memory.
pub(crate) struct Roles {
    file: RolesFile,
    disk: Option<Disk>,
    audit: Audit,
    /// The code that makes its account the Owner, while there is none.
    claim: Option<String>,
    /// Accounts whose role changed, for anyone playing on them (`take_changed`).
    changed: Vec<AccountId>,
}

impl Default for Roles {
    fn default() -> Self {
        Self {
            file: RolesFile::new(),
            disk: None,
            audit: Audit::default(),
            claim: None,
            changed: Vec::new(),
        }
    }
}

/// An account's role, as it is now.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Held {
    pub badge: RoleBadge,
    pub powers: Powers,
    /// When it ends, if ever.
    pub until: Option<u64>,
    pub reason: String,
}

impl Roles {
    /// The roles kept in `data`: made if there are none, and written back if a
    /// newer version adds to them. An error if they can't be read or make no
    /// sense, as the server mustn't run without them (or write over them).
    pub(crate) fn open(data: &Path, now: u64) -> Result<Self, String> {
        let disk = Disk {
            data: data.to_owned(),
        };
        let (mut file, mut changed) = match disk.read()? {
            Some(file) => (file, false),
            None => (RolesFile::new(), true),
        };
        changed |= file.fill_in();
        file.check()
            .map_err(|problem| format!("{}: {problem}", disk.file().display()))?;
        if changed {
            disk.write(&file, now)?;
        }
        Ok(Self {
            file,
            audit: Audit {
                path: Some(data.join("audit.log")),
                lines: Vec::new(),
            },
            disk: Some(disk),
            claim: None,
            changed: Vec::new(),
        })
    }

    /// With no Owner yet, makes a claim code (written beside the roles, if
    /// they are kept) and returns it to be logged. `None` if there is an Owner.
    pub(crate) fn start_claim(&mut self) -> Result<Option<String>, String> {
        if self.file.has_owner() {
            return Ok(None);
        }
        let code = claim::new_code()?;
        if let Some(disk) = &self.disk {
            claim::write_file(&disk.data, &code)?;
        }
        self.claim = Some(code.clone());
        Ok(Some(code))
    }

    /// The badge and powers of a role.
    fn role(&self, id: &RoleId, until: Option<u64>, reason: String) -> Held {
        let def = self.file.role(id).expect("checked to exist");
        Held {
            badge: RoleBadge {
                id: def.id.clone(),
                name: def.name.clone(),
                title: def.title.clone(),
            },
            powers: Powers {
                permissions: def
                    .permissions
                    .iter()
                    .filter(|(_, on)| **on)
                    .map(|(permission, _)| *permission)
                    .collect(),
                grants: def.grants.clone(),
                appears_as: def.appears_as.clone(),
            },
            until,
            reason,
        }
    }

    /// What is shown of a role, if there is one by this id.
    pub(crate) fn badge(&self, id: &RoleId) -> Option<RoleBadge> {
        self.file.role(id).map(|def| RoleBadge {
            id: def.id.clone(),
            name: def.name.clone(),
            title: def.title.clone(),
        })
    }

    /// The roles an account holding `held` may be shown as: its own first.
    pub(crate) fn choices(&self, held: &Held) -> Vec<RoleBadge> {
        let others = held
            .powers
            .appears_as
            .iter()
            .filter_map(|id| self.badge(id));
        std::iter::once(held.badge.clone()).chain(others).collect()
    }

    /// The Owner's role: the embedded server's own player's.
    pub(crate) fn owner(&self) -> Held {
        self.role(&RoleId::new(OWNER), None, String::new())
    }

    /// An account's role now: a guest's is Guest, and one that has run out
    /// goes back to Player (and is written down).
    pub(crate) fn held(&mut self, account: AccountId, guest: bool, now: u64) -> Held {
        if guest {
            return self.role(&RoleId::new(GUEST), None, String::new());
        }
        let key = account.0.to_string();
        let until = self.file.accounts.get(&key).and_then(|held| held.until);
        if until.is_some_and(|until| until <= now) {
            let what = format!("{key}'s role ran out");
            if let Err(problem) = self.change(&key, None, now, &what) {
                bevy::log::error!("{problem}");
            }
        }
        match self.file.accounts.get(&key) {
            Some(held) => self.role(&held.role, held.until, held.reason.clone()),
            None => self.role(&RoleId::new(PLAYER), None, String::new()),
        }
    }

    /// The role id an account holds (Player for none).
    fn holds(&self, key: &str) -> RoleId {
        self.file
            .accounts
            .get(key)
            .map_or_else(|| RoleId::new(PLAYER), |held| held.role.clone())
    }

    /// Gives `key` this role (or none, for Player), writes the file and the
    /// audit log; nothing changes if the file can't be written.
    fn change(
        &mut self,
        key: &str,
        to: Option<HeldRole>,
        now: u64,
        what: &str,
    ) -> Result<(), String> {
        let before = self.file.accounts.clone();
        match to {
            Some(held) => self.file.accounts.insert(key.to_owned(), held),
            None => self.file.accounts.remove(key),
        };
        if let Some(disk) = &self.disk
            && let Err(problem) = disk.write(&self.file, now)
        {
            self.file.accounts = before;
            return Err(problem);
        }
        self.audit.record(now, what);
        Ok(())
    }

    /// The audit log's lines, when kept in memory (tests).
    #[cfg(test)]
    pub(crate) fn audit_lines(&self) -> &[String] {
        &self.audit.lines
    }
}

#[cfg(test)]
mod tests;
