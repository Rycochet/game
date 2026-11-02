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
use shared::protocol::{CharacterId, CharacterSummary, Credentials, GuestToken};

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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct AccountId(String);

impl AccountId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[cfg(test)]
    pub fn for_test(id: &str) -> Self {
        Self(id.to_owned())
    }
}

/// An account's save.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct AccountSave {
    /// A guest has one character, with nothing to choose.
    #[serde(default)]
    pub guest: bool,
    /// When it was made (seconds since 1970).
    #[serde(default)]
    pub created: u64,
    #[serde(default)]
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
        matches!(self, Self::Guest(_))
    }

    /// Its file name. A method's name comes first, so the same key from another
    /// method is another login.
    fn hash(&self) -> String {
        let mut text = match self {
            Self::Guest(_) => b"guest:".to_vec(),
            #[cfg(test)]
            Self::Test(_) => b"test:".to_vec(),
        };
        match self {
            Self::Guest(token) => text.extend(token.0),
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
            return Ok(id.clone());
        }
        let login = SaveKey::Login(hash.clone());
        let saved = match &self.queue {
            Some(queue) => queue.load(&login)?,
            None => None,
        };
        let id = match saved {
            Some(Save::Login(login)) => login.account,
            _ => {
                let id = AccountId(random_id());
                if let Some(queue) = &self.queue {
                    let account = id.clone();
                    queue.save(login, Save::Login(LoginSave { account }));
                }
                self.loaded.insert(
                    id.clone(),
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
        self.logins.insert(hash, id.clone());
        Ok(id)
    }

    /// An account. One whose save has gone (or is broken) starts again, as a
    /// guest's so it can't be left with nothing to choose from. An error if saves
    /// can't be reached.
    pub(crate) fn account(&mut self, id: &AccountId) -> Result<&AccountSave, String> {
        if !self.loaded.contains_key(id) {
            let saved = match &self.queue {
                Some(queue) => queue.load(&SaveKey::Account(id.clone()))?,
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
            self.loaded.insert(id.clone(), save);
        }
        Ok(&self.loaded[id])
    }

    fn save(&self, id: &AccountId) {
        if let (Some(queue), Some(save)) = (&self.queue, self.loaded.get(id)) {
            queue.save(SaveKey::Account(id.clone()), Save::Account(save.clone()));
        }
    }

    /// The character of a guest's account: their one, made now if need be.
    pub(crate) fn guest_character(&mut self, id: &AccountId) -> Result<CharacterId, String> {
        if let Some(character) = self.account(id)?.characters.first() {
            return Ok(character.id.clone());
        }
        self.add_character(id, GUEST_NAME.into())
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
        let character = CharacterId::new(&random_id()).expect("hex digits are a fine id");
        self.account(id)?;
        let account = self.loaded.get_mut(id).expect("just loaded");
        account.characters.push(CharacterSummary {
            id: character.clone(),
            name,
            last_played: 0,
        });
        self.save(id);
        Ok(character)
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

/// 16 random hex digits, for the ids the server makes.
fn random_id() -> String {
    let mut bytes = [0; 8];
    SystemRandom::new()
        .fill(&mut bytes)
        .expect("the system has randomness");
    hex(&bytes)
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
mod tests {
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use super::*;
    use crate::save_queue::tests::MemoryBackend;
    use crate::storage::tests::TestFolder;

    /// Every file under a folder, as text: its path and contents.
    fn everything_in(folder: &Path) -> String {
        let mut text = String::new();
        for entry in fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            text += &path.display().to_string();
            if path.is_dir() {
                text += &everything_in(&path);
            } else {
                text += &fs::read_to_string(&path).unwrap();
            }
        }
        text
    }

    #[test]
    fn a_login_finds_its_account_again_after_a_restart() {
        let folder = TestFolder::new();
        let token = LoginKey::Guest(GuestToken([3; 32]));
        let mut accounts = Accounts::new(SaveQueue::in_folder(&folder.0));
        let id = accounts.log_in(&token).unwrap();
        let character = accounts.guest_character(&id).unwrap();
        assert!(accounts.account(&id).unwrap().guest);
        // Dropping it finishes writing, as stopping does.
        drop(accounts);

        let mut restarted = Accounts::new(SaveQueue::in_folder(&folder.0));
        assert_eq!(restarted.log_in(&token).unwrap(), id);
        assert_eq!(restarted.guest_character(&id).unwrap(), character);
        assert_eq!(restarted.account(&id).unwrap().characters.len(), 1);
        // Someone else is someone else.
        let other = LoginKey::Guest(GuestToken([4; 32]));
        assert_ne!(restarted.log_in(&other).unwrap(), id);
    }

    #[test]
    fn a_guest_token_is_never_saved() {
        let folder = TestFolder::new();
        let token = GuestToken([0xcd; 32]);
        let mut accounts = Accounts::new(SaveQueue::in_folder(&folder.0));
        let id = accounts.log_in(&LoginKey::Guest(token)).unwrap();
        accounts.guest_character(&id).unwrap();
        drop(accounts);
        assert!(!everything_in(&folder.0).contains(&token.to_hex()));
    }

    #[test]
    fn the_same_key_from_another_method_is_another_login() {
        let guest = LoginKey::Guest(GuestToken([0; 32])).hash();
        let test = LoginKey::Test("").hash();
        assert_ne!(guest, test);
    }

    #[test]
    fn names_are_checked_and_characters_limited() {
        let mut accounts = Accounts::default();
        let id = accounts.log_in(&LoginKey::Test("someone")).unwrap();
        assert!(!accounts.account(&id).unwrap().guest);
        for bad in ["", "   ", "a\nb", &"x".repeat(MAX_NAME + 1)] {
            assert!(accounts.create_character(&id, bad).is_err(), "{bad}");
        }
        for _ in 0..MAX_CHARACTERS {
            accounts
                .create_character(&id, " Long Name Ünder 24 ")
                .unwrap();
        }
        assert_eq!(
            accounts.account(&id).unwrap().characters[0].name,
            "Long Name Ünder 24"
        );
        assert!(accounts.create_character(&id, "One More").is_err());
    }

    #[test]
    fn saves_that_cant_be_reached_dont_make_a_new_account() {
        let backend = Arc::new(MemoryBackend::default());
        let queue = SaveQueue::new(backend.clone());
        let token = LoginKey::Guest(GuestToken([3; 32]));
        let id = Accounts::new(queue.clone()).log_in(&token).unwrap();
        assert!(queue.flush(Duration::from_secs(5)));

        backend.cant_read.store(true, Ordering::Relaxed);
        let mut restarted = Accounts::new(queue.clone());
        assert!(restarted.log_in(&token).is_err());
        assert!(restarted.account(&id).is_err());

        backend.cant_read.store(false, Ordering::Relaxed);
        assert_eq!(restarted.log_in(&token).unwrap(), id);
    }
}
