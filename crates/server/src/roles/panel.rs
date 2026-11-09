//! What the Accounts panel asks of the roles beyond giving them: every role,
//! renaming one, reading the audit log, and which accounts changed.

use std::fs;

use shared::role::{RoleBadge, RoleId};

use super::Roles;
use crate::accounts::AccountId;

/// How many lines of the audit log go at a time.
pub(crate) const AUDIT_PAGE: usize = 50;

/// The longest a role's name or title may be, in characters.
const MOST: usize = 24;

impl Roles {
    /// Every role, as shown.
    pub(crate) fn badges(&self) -> Vec<RoleBadge> {
        let ids = self.file.roles.iter().map(|role| role.id.clone());
        ids.filter_map(|id| self.badge(&id)).collect()
    }

    /// Gives a role a new name and title (none to show the name): written down
    /// by `by` (an account id, for the log). What is wrong, if anything.
    pub(crate) fn rename(
        &mut self,
        by: AccountId,
        id: &RoleId,
        name: &str,
        title: Option<&str>,
        now: u64,
    ) -> Result<(), String> {
        let name = name.trim();
        let title = title.map(str::trim).filter(|title| !title.is_empty());
        let fits = |words: &str| (1..=MOST).contains(&words.chars().count());
        if !fits(name) || title.is_some_and(|title| !fits(title)) {
            return Err(format!("names and titles are 1 to {MOST} characters"));
        }
        let Some(role) = self.file.roles.iter_mut().find(|role| role.id == *id) else {
            return Err(format!("there is no role {:?}", id.0));
        };
        let before = (role.name.clone(), role.title.clone());
        role.name = name.to_owned();
        role.title = title.map(str::to_owned);
        if let Some(disk) = &self.disk
            && let Err(problem) = disk.write(&self.file, now)
        {
            let role = self.file.roles.iter_mut().find(|role| role.id == *id);
            let role = role.expect("found above");
            (role.name, role.title) = before;
            return Err(problem);
        }
        let shown = title.map_or(String::new(), |title| format!(", titled {title}"));
        let what = format!("{} named the role {} {name}{shown}", by.0, id.0);
        self.audit.record(now, &what);
        Ok(())
    }

    /// The audit log's lines, newest first, after skipping `skip`, and whether
    /// there are older ones.
    pub(crate) fn audit_page(&self, skip: usize) -> Result<(Vec<String>, bool), String> {
        let lines: Vec<String> = match &self.audit.path {
            Some(path) => match fs::read_to_string(path) {
                Ok(text) => text.lines().map(str::to_owned).collect(),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
                Err(error) => return Err(format!("can't read the audit log: {error}")),
            },
            None => self.audit.lines.clone(),
        };
        let page: Vec<String> = lines
            .iter()
            .rev()
            .skip(skip)
            .take(AUDIT_PAGE)
            .cloned()
            .collect();
        let more = lines.len() > skip + page.len();
        Ok((page, more))
    }

    /// The accounts whose role changed since last asked.
    pub(crate) fn take_changed(&mut self) -> Vec<AccountId> {
        std::mem::take(&mut self.changed)
    }
}
