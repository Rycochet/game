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

use ring::digest::{SHA256, digest};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use shared::id::Id;
use shared::protocol::{CharacterId, CharacterSummary, Credentials, GuestToken};

use crate::ids::new_id;
use crate::save_queue::SaveQueue;
use crate::saves::{Save, SaveKey};
use crate::storage::now;

/// How many characters an account may have.
pub(crate) const MAX_CHARACTERS: usize = 8;

/// The longest name a character may have, in characters.
const MAX_NAME: usize = 24;

/// What a guest's one character is called.
const GUEST_NAME: &str = "Guest";

/// Which account: made by the server, and the name of its file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct AccountId(pub Id);

/// An account's save.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct AccountSave {
    /// A guest has one character, with nothing to choose.
    pub guest: bool,
    /// When it was made (seconds since 1970).
    pub created: u64,
    pub characters: Vec<CharacterSummary>,
}

/// A login's save.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct LoginSave {
    account: AccountId,
}

/// A way of logging in, proved by whoever makes one.
pub(crate) enum LoginKey {
    Guest(GuestToken),
    /// The embedded server's one player, who has nothing to log in with.
    SinglePlayer,
    /// The only way to an account that isn't a guest's, until there are logins.
    #[cfg(test)]
    Test(&'static str),
}

impl From<&Credentials> for LoginKey {
    fn from(credentials: &Credentials) -> Self {
        match credentials {
            Credentials::Guest(token) => Self::Guest(*token),
        }
    }
}

impl LoginKey {
    fn is_guest(&self) -> bool {
        matches!(self, Self::Guest(_) | Self::SinglePlayer)
    }

    /// Its file name. A method's name comes first, so the same key from another
    /// method is another login.
    fn hash(&self) -> String {
        let mut text = match self {
            Self::Guest(_) => b"guest:".to_vec(),
            Self::SinglePlayer => b"single player:".to_vec(),
            #[cfg(test)]
            Self::Test(_) => b"test:".to_vec(),
        };
        match self {
            Self::Guest(token) => text.extend(token.0),
            Self::SinglePlayer => {}
            #[cfg(test)]
            Self::Test(key) => text.extend(key.as_bytes()),
        }
        hex(digest(&SHA256, &text).as_ref())
    }
}

/// Every account seen so far, and where they are kept (with no queue, only
/// here: tests).
#[derive(Default)]
pub struct Accounts {
    queue: Option<SaveQueue>,
    /// By login hash.
    logins: BTreeMap<String, AccountId>,
    loaded: BTreeMap<AccountId, AccountSave>,
}

impl Accounts {
    /// The accounts saved through `queue`.
    pub(crate) fn new(queue: SaveQueue) -> Self {
        Self {
            queue: Some(queue),
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
                let id = AccountId(new_id());
                if let Some(queue) = &self.queue {
                    queue.save(login, Save::Login(LoginSave { account: id }));
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
        }
        Ok(&self.loaded[id])
    }

    fn save(&self, id: &AccountId) {
        if let (Some(queue), Some(save)) = (&self.queue, self.loaded.get(id)) {
            queue.save(SaveKey::Account(*id), Save::Account(save.clone()));
        }
    }

    /// The character of a guest's account: their one, made now if need be.
    pub(crate) fn guest_character(&mut self, id: &AccountId) -> Result<CharacterId, String> {
        if let Some(character) = self.account(id)?.characters.first() {
            return Ok(character.id);
        }
        self.add_character(id, GUEST_NAME.into())
    }

    /// The embedded server's one character, made the first time and saved like
    /// anyone else's. An error if saves can't be reached.
    pub(crate) fn single_player(&mut self) -> Result<CharacterId, String> {
        let id = self.log_in(&LoginKey::SinglePlayer)?;
        self.guest_character(&id)
    }

    /// Makes a character with the name the player chose. An error says what is
    /// wrong with asking for it.
    pub(crate) fn create_character(
        &mut self,
        id: &AccountId,
        name: &str,
    ) -> Result<CharacterId, String> {
        let name = checked_name(name)?;
        if self.account(id)?.characters.len() >= MAX_CHARACTERS {
            return Err(format!("an account can have {MAX_CHARACTERS} characters"));
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
        });
        self.save(id);
        Ok(character)
    }

    /// Gives a character already made (a guest's, the single player's) the name
    /// its player chose. Its account must have been loaded, as logging in does.
    pub(crate) fn rename_character(
        &mut self,
        character: &CharacterId,
        name: &str,
    ) -> Result<(), String> {
        let name = checked_name(name)?;
        let account = self.loaded.iter_mut().find_map(|(id, account)| {
            let found = account.characters.iter_mut().find(|c| c.id == *character)?;
            found.name = name.clone();
            Some(*id)
        });
        let account = account.ok_or("no such character")?;
        self.save(&account);
        Ok(())
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

    /// Whether an account is a guest's (the single player's is one).
    pub(crate) fn is_guest(&mut self, id: &AccountId) -> Result<bool, String> {
        Ok(self.account(id)?.guest)
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

fn hex(bytes: &[u8]) -> String {
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

#[cfg(test)]
mod tests;
