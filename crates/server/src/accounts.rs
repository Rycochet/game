//! Accounts: who has logged in, and which characters they have. See "Accounts"
//! in `docs/context/architecture.md`.
//!
//! Saved in the server's data folder:
//!
//! - `logins/<hash>.ron`: which account a way of logging in belongs to. The
//!   hash is of the login's method and key, so no secret is kept, and more than
//!   one way in can lead to the same account.
//! - `accounts/<id>.ron`: an account and its characters (`AccountSave`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ring::digest::{SHA256, digest};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use shared::protocol::{CharacterId, CharacterSummary, Credentials, GuestToken};

use crate::storage::{now, read_or_put_aside, write};

/// How many characters an account may have.
pub(crate) const MAX_CHARACTERS: usize = 8;

/// The longest name a character may have, in characters.
const MAX_NAME: usize = 24;

/// What a guest's one character is called.
const GUEST_NAME: &str = "Guest";

/// Which account: made by the server, and the name of its file.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct AccountId(String);

/// `accounts/<id>.ron`.
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

/// `logins/<hash>.ron`.
#[derive(Serialize, Deserialize)]
struct LoginSave {
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

/// Every account seen so far, and where they are kept (with no folder, only
/// here: tests).
#[derive(Default)]
pub struct Accounts {
    folder: Option<PathBuf>,
    /// By login hash.
    logins: BTreeMap<String, AccountId>,
    loaded: BTreeMap<AccountId, AccountSave>,
}

impl Accounts {
    /// The accounts in the data folder `data`.
    pub(crate) fn in_folder(data: &Path) -> Self {
        Self {
            folder: Some(data.to_owned()),
            ..Self::default()
        }
    }

    fn login_file(&self, hash: &str) -> Option<PathBuf> {
        let file = self.folder.as_ref()?.join("logins");
        Some(file.join(format!("{hash}.ron")))
    }

    fn account_file(&self, id: &AccountId) -> Option<PathBuf> {
        let file = self.folder.as_ref()?.join("accounts");
        Some(file.join(format!("{}.ron", id.0)))
    }

    /// The account this login belongs to: made now if it is the first time.
    pub(crate) fn log_in(&mut self, key: &LoginKey) -> AccountId {
        let hash = key.hash();
        if let Some(id) = self.logins.get(&hash) {
            return id.clone();
        }
        let file = self.login_file(&hash);
        let saved = file.as_deref().and_then(read_or_put_aside::<LoginSave>);
        let id = match saved {
            Some(login) => login.account,
            None => {
                let id = AccountId(random_id());
                if let Some(file) = &file {
                    let account = id.clone();
                    write(file, &LoginSave { account });
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
        id
    }

    /// An account. One whose file has gone (or is broken) starts again, as a
    /// guest's so it can't be left with nothing to choose from.
    pub(crate) fn account(&mut self, id: &AccountId) -> &AccountSave {
        if !self.loaded.contains_key(id) {
            let file = self.account_file(id);
            let save = file.as_deref().and_then(read_or_put_aside);
            let save = save.unwrap_or_else(|| AccountSave {
                guest: true,
                created: now(),
                characters: Vec::new(),
            });
            self.loaded.insert(id.clone(), save);
        }
        &self.loaded[id]
    }

    fn save(&self, id: &AccountId) {
        if let (Some(file), Some(save)) = (self.account_file(id), self.loaded.get(id)) {
            write(&file, save);
        }
    }

    /// The character of a guest's account: their one, made now if need be.
    pub(crate) fn guest_character(&mut self, id: &AccountId) -> CharacterId {
        if let Some(character) = self.account(id).characters.first() {
            return character.id.clone();
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
        if self.account(id).characters.len() >= MAX_CHARACTERS {
            return Err(format!("an account can have {MAX_CHARACTERS} characters"));
        }
        Ok(self.add_character(id, name))
    }

    fn add_character(&mut self, id: &AccountId, name: String) -> CharacterId {
        let character = CharacterId::new(&random_id()).expect("hex digits are a fine id");
        self.account(id);
        let account = self.loaded.get_mut(id).expect("just loaded");
        account.characters.push(CharacterSummary {
            id: character.clone(),
            name,
            last_played: 0,
        });
        self.save(id);
        character
    }

    /// Notes that a character is being played now.
    pub(crate) fn played(&mut self, id: &AccountId, character: &CharacterId) {
        self.account(id);
        let account = self.loaded.get_mut(id).expect("just loaded");
        if let Some(found) = account.characters.iter_mut().find(|c| c.id == *character) {
            found.last_played = now();
            self.save(id);
        }
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

    use super::*;
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
        let mut accounts = Accounts::in_folder(&folder.0);
        let id = accounts.log_in(&token);
        let character = accounts.guest_character(&id);
        assert!(accounts.account(&id).guest);

        let mut restarted = Accounts::in_folder(&folder.0);
        assert_eq!(restarted.log_in(&token), id);
        assert_eq!(restarted.guest_character(&id), character);
        assert_eq!(restarted.account(&id).characters.len(), 1);
        // Someone else is someone else.
        assert_ne!(restarted.log_in(&LoginKey::Guest(GuestToken([4; 32]))), id);
    }

    #[test]
    fn a_guest_token_is_never_saved() {
        let folder = TestFolder::new();
        let token = GuestToken([0xcd; 32]);
        let mut accounts = Accounts::in_folder(&folder.0);
        let id = accounts.log_in(&LoginKey::Guest(token));
        accounts.guest_character(&id);
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
        let id = accounts.log_in(&LoginKey::Test("someone"));
        assert!(!accounts.account(&id).guest);
        for bad in ["", "   ", "a\nb", &"x".repeat(MAX_NAME + 1)] {
            assert!(accounts.create_character(&id, bad).is_err(), "{bad}");
        }
        for _ in 0..MAX_CHARACTERS {
            accounts
                .create_character(&id, " Long Name Ünder 24 ")
                .unwrap();
        }
        assert_eq!(
            accounts.account(&id).characters[0].name,
            "Long Name Ünder 24"
        );
        assert!(accounts.create_character(&id, "One More").is_err());
    }
}
