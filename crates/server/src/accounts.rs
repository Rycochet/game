//! Accounts: who has logged in, and which characters they have. See "Accounts"
//! in `docs/context/architecture.md`.
//!
//! Saved through `saves` (see `FileBackend` for where):
//!
//! - logins (`LoginSave`): which account a way of logging in belongs to, by a
//!   hash of the login's method and key, so no secret is kept, and more than one
//!   way in can lead to the same account.
//! - accounts (`AccountSave`): an account and its characters.

use std::collections::BTreeMap;

use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use shared::id::Id;
use shared::protocol::{CharacterId, CharacterSummary, GuestToken};

use crate::ids::new_id;
use crate::roles::Roles;
use crate::save_queue::SaveQueue;
use crate::saves::{Save, SaveKey};
use crate::storage::now;

/// How many characters an account may have.
pub(crate) const MAX_CHARACTERS: usize = 8;

/// The longest name a character may have, in characters.
const MAX_NAME: usize = 24;

/// What a character made without the client's screen is called.
const PLAIN_NAME: &str = "Guest";

/// Which account: made by the server, and the name of its file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct AccountId(pub Id);

/// An account's save.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct AccountSave {
    /// A guest may have only one character.
    pub guest: bool,
    /// When it was made (seconds since 1970).
    pub created: u64,
    pub characters: Vec<CharacterSummary>,
}

/// A login's save.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct LoginSave {
    account: AccountId,
    /// When it stops working (seconds since 1970): only a session's, put off
    /// each time it is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expires: Option<u64>,
}

/// Every account seen so far, and where they are kept (with no queue, only
/// here: tests).
#[derive(Default)]
pub struct Accounts {
    queue: Option<SaveQueue>,
    /// By login hash.
    logins: BTreeMap<String, AccountId>,
    loaded: BTreeMap<AccountId, AccountSave>,
    /// Sessions used since starting, by login hash (`session`).
    sessions: BTreeMap<String, LoginSave>,
    /// Which accounts hold which role.
    pub(crate) roles: Roles,
}

impl Accounts {
    /// The accounts saved through `queue` (if any), with these roles.
    pub(crate) fn new(queue: Option<SaveQueue>, roles: Roles) -> Self {
        Self {
            queue,
            roles,
            ..Self::default()
        }
    }

    /// The account this login belongs to: made now if it is the first time. An
    /// error if saves can't be reached: making another account would lose theirs.
    pub(crate) fn log_in(&mut self, key: &LoginKey) -> Result<AccountId, String> {
        let hash = key.hash();
        if let Some(id) = self.logins.get(&hash) {
            return Ok(*id);
        }
        let login = SaveKey::Login(hash.clone());
        let saved = match &self.queue {
            Some(queue) => queue.load(&login)?,
            None => None,
        };
        let id = match saved {
            Some(Save::Login(login)) => login.account,
            _ => {
                let id = key.account_id().unwrap_or_else(|| AccountId(new_id()));
                if let Some(queue) = &self.queue {
                    let save = LoginSave {
                        account: id,
                        expires: None,
                    };
                    queue.save(login, Save::Login(save));
                }
                self.loaded.insert(
                    id,
                    AccountSave {
                        guest: key.is_guest(),
                        created: now(),
                        characters: Vec::new(),
                    },
                );
                self.save(&id);
                id
            }
        };
        self.logins.insert(hash, id);
        Ok(id)
    }

    /// An account. One whose save has gone (or is broken) starts again, as a
    /// guest's so it can't be left with nothing to choose from. An error if saves
    /// can't be reached.
    pub(crate) fn account(&mut self, id: &AccountId) -> Result<&AccountSave, String> {
        if !self.loaded.contains_key(id) {
            let saved = match &self.queue {
                Some(queue) => queue.load(&SaveKey::Account(*id))?,
                None => None,
            };
            let save = match saved {
                Some(Save::Account(save)) => save,
                _ => AccountSave {
                    guest: true,
                    created: now(),
                    characters: Vec::new(),
                },
            };
            self.loaded.insert(*id, save);
            if self.date_characters(id) {
                self.save(id);
            }
        }
        Ok(&self.loaded[id])
    }

    fn save(&self, id: &AccountId) {
        if let (Some(queue), Some(save)) = (&self.queue, self.loaded.get(id)) {
            queue.save(SaveKey::Account(*id), Save::Account(save.clone()));
        }
    }

    /// Gives characters saved before there were creation times the time now:
    /// whether any had none.
    fn date_characters(&mut self, id: &AccountId) -> bool {
        let Some(account) = self.loaded.get_mut(id) else {
            return false;
        };
        let mut any = false;
        for character in account.characters.iter_mut().filter(|c| c.created == 0) {
            character.created = now();
            any = true;
        }
        any
    }

    /// How many characters an account may have: a guest one.
    pub(crate) fn most_characters(&mut self, id: &AccountId) -> Result<usize, String> {
        Ok(if self.account(id)?.guest {
            1
        } else {
            MAX_CHARACTERS
        })
    }

    /// A character made without the client's screen, plainly named.
    pub(crate) fn plain_character(&mut self, id: &AccountId) -> Result<CharacterId, String> {
        self.add_character(id, PLAIN_NAME.into())
    }

    /// The embedded server's player's account, made the first time and saved like
    /// anyone else's. Not a guest's, as it was once: it may have as many
    /// characters as any. An error if saves can't be reached.
    pub(crate) fn single_player(&mut self) -> Result<AccountId, String> {
        let id = self.log_in(&LoginKey::SinglePlayer)?;
        self.account(&id)?;
        let account = self.loaded.get_mut(&id).expect("just loaded");
        if std::mem::take(&mut account.guest) {
            self.save(&id);
        }
        Ok(id)
    }

    /// Makes a character with the name the player chose. An error says what is
    /// wrong with asking for it.
    pub(crate) fn create_character(
        &mut self,
        id: &AccountId,
        name: &str,
    ) -> Result<CharacterId, String> {
        let name = checked_name(name)?;
        let most = self.most_characters(id)?;
        if self.account(id)?.characters.len() >= most {
            return Err(match most {
                1 => "a guest can have one character".into(),
                _ => format!("an account can have {most} characters"),
            });
        }
        self.add_character(id, name)
    }

    fn add_character(&mut self, id: &AccountId, name: String) -> Result<CharacterId, String> {
        let character = CharacterId(new_id());
        self.account(id)?;
        let account = self.loaded.get_mut(id).expect("just loaded");
        account.characters.push(CharacterSummary {
            id: character,
            name,
            last_played: 0,
            created: now(),
        });
        self.save(id);
        Ok(character)
    }

    /// A character's name, if its account has been loaded (as logging in does).
    pub(crate) fn character_name(&self, character: &CharacterId) -> Option<String> {
        self.loaded.values().find_map(|account| {
            let found = account.characters.iter().find(|c| c.id == *character)?;
            Some(found.name.clone())
        })
    }

    /// Takes a character off its account, which must have been loaded: the
    /// account's id, if it was on one.
    pub(crate) fn remove_character(&mut self, character: &CharacterId) -> Option<AccountId> {
        let account = self.loaded.iter_mut().find_map(|(id, account)| {
            let before = account.characters.len();
            account.characters.retain(|c| c.id != *character);
            (account.characters.len() < before).then_some(*id)
        })?;
        self.save(&account);
        Some(account)
    }

    /// Notes that a character is being played now.
    pub(crate) fn played(&mut self, id: &AccountId, character: &CharacterId) -> Result<(), String> {
        self.account(id)?;
        let account = self.loaded.get_mut(id).expect("just loaded");
        if let Some(found) = account.characters.iter_mut().find(|c| c.id == *character) {
            found.last_played = now();
            self.save(id);
        }
        Ok(())
    }
}

/// The name without the spaces around it, or what is wrong with it.
fn checked_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    let length = name.chars().count();
    if !(1..=MAX_NAME).contains(&length) {
        return Err(format!("a name is 1 to {MAX_NAME} characters long"));
    }
    if name.chars().any(char::is_control) {
        return Err("a name can't contain control characters".into());
    }
    Ok(name.to_owned())
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// A new guest's secret, for a native client. An error if the system's secure
/// randomness fails: a guessable token would give the character away.
pub fn new_guest_token() -> Result<GuestToken, String> {
    let mut token = [0; 32];
    SystemRandom::new()
        .fill(&mut token)
        .map_err(|_| "the system can't provide secure random numbers".to_owned())?;
    Ok(GuestToken(token))
}

// Split into files (`cargo xtask lint` keeps files small).
mod login_key;
mod session;

pub(crate) use login_key::{EmailKey, LoginKey};
#[cfg(test)]
use session::session_hash;

#[cfg(test)]
mod tests;
