//! Giving and taking roles, by someone allowed to or by the server itself
//! (a claim), and the rules for who may.

use shared::role::{GUEST, OWNER, PLAYER, Powers, RoleId};

use super::Roles;
use super::file::HeldRole;
use crate::accounts::AccountId;

/// Who is changing a role.
pub(crate) enum Actor<'a> {
    /// An account, which may only give roles in its `grants`, to accounts
    /// holding one of them, and never to itself.
    Account(AccountId, &'a Powers),
    /// The server: a claim.
    Server,
}

/// What is asked of a role change.
pub(crate) struct Change<'a> {
    pub to: &'a RoleId,
    pub until: Option<u64>,
    pub reason: &'a str,
}

impl Roles {
    /// Gives a (non-guest) account a role, or takes its role away (Player), as
    /// `by` may: what is wrong, if anything.
    pub(crate) fn set_role(
        &mut self,
        by: Actor,
        target: AccountId,
        guest: bool,
        change: Change,
        now: u64,
    ) -> Result<(), String> {
        let key = target.0.to_string();
        let to = change.to;
        if guest {
            return Err("a guest can't hold a role".into());
        }
        if self.file.role(to).is_none() || to.is(GUEST) {
            return Err(format!("there is no role {:?} to give", to.0));
        }
        let from = self.holds(&key);
        let by_name = match by {
            Actor::Account(actor, powers) => {
                if actor == target {
                    return Err("nobody can change their own role".into());
                }
                if !powers.grants.contains(&from) {
                    return Err("that account's role isn't one you can change".into());
                }
                if !powers.grants.contains(to) {
                    return Err("that isn't a role you can give".into());
                }
                Some(actor.0.to_string())
            }
            Actor::Server => None,
        };
        let until = change
            .until
            .map(|u| format!(" until {}", super::stamp::log_stamp(u)));
        let what = format!(
            "{} gave {key} {} (was {}){}{}",
            by_name.as_deref().unwrap_or("the server"),
            to.0,
            from.0,
            until.unwrap_or_default(),
            if change.reason.is_empty() {
                String::new()
            } else {
                format!(": {}", change.reason)
            },
        );
        let held = (!to.is(PLAYER)).then(|| HeldRole {
            role: to.clone(),
            since: now,
            until: change.until,
            reason: change.reason.to_owned(),
            by: by_name,
        });
        self.change(&key, held, now, &what)?;
        self.changed.push(target);
        Ok(())
    }

    /// If `name` is the claim code: the account becomes the Owner (`Ok`), or
    /// can't, being a guest's (`Err`). `None` for any other name.
    pub(crate) fn try_claim(
        &mut self,
        account: AccountId,
        guest: bool,
        name: &str,
        now: u64,
    ) -> Option<Result<(), String>> {
        if self.claim.as_deref() != Some(name.trim()) {
            return None;
        }
        if guest {
            return Some(Err(
                "a guest can't claim this server: log in with an account".into(),
            ));
        }
        let owner = RoleId::new(OWNER);
        let change = Change {
            to: &owner,
            until: None,
            reason: "claimed with the claim code",
        };
        let claimed = self.set_role(Actor::Server, account, false, change, now);
        if claimed.is_ok() {
            self.claim = None;
            if let Some(disk) = &self.disk {
                super::claim::remove_file(&disk.data);
            }
        }
        Some(claimed)
    }
}
