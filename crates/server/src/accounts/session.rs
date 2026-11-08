//! Sessions: a token given on logging in through a provider, standing for that
//! account so the player needn't go through the provider's website every time.
//! Saved like any login, by a hash, with when it runs out, put off each use.

use ring::digest::{SHA256, digest};
use ring::rand::{SecureRandom, SystemRandom};
use shared::protocol::SessionToken;

use super::{AccountId, Accounts, LoginSave, hex};
use crate::saves::{Save, SaveKey};
use crate::storage::now;

/// How many days a session lasts unused.
pub(crate) const SESSION_DAYS: u64 = 90;

const SESSION_SECONDS: u64 = SESSION_DAYS * 24 * 60 * 60;

/// A session's file name, apart from every other kind of login's.
pub(super) fn session_hash(token: &SessionToken) -> String {
    let mut text = b"session:".to_vec();
    text.extend(token.0);
    hex(digest(&SHA256, &text).as_ref())
}

impl Accounts {
    /// A new session for an account: its token, which only its player has.
    /// An error if the system's secure randomness fails.
    pub(crate) fn new_session(&mut self, account: AccountId) -> Result<SessionToken, String> {
        let mut token = [0; 32];
        SystemRandom::new()
            .fill(&mut token)
            .map_err(|_| "the system can't provide secure random numbers".to_owned())?;
        let token = SessionToken(token);
        self.keep_session(&session_hash(&token), account);
        Ok(token)
    }

    /// The account a session token stands for, if it is known and hasn't run
    /// out; using it puts off when it does. An error if saves can't be reached.
    pub(crate) fn session(&mut self, token: &SessionToken) -> Result<Option<AccountId>, String> {
        let hash = session_hash(token);
        let saved = match (&self.queue, self.sessions.get(&hash)) {
            (_, Some(save)) => Some(save.clone()),
            (Some(queue), None) => match queue.load(&SaveKey::Login(hash.clone()))? {
                Some(Save::Login(save)) => Some(save),
                _ => None,
            },
            (None, None) => None,
        };
        let Some(save) = saved else {
            return Ok(None);
        };
        // A login of another kind (which this hash can't be) has none.
        if save.expires.is_none_or(|expires| expires <= now()) {
            self.sessions.remove(&hash);
            if let Some(queue) = &self.queue {
                queue.remove(SaveKey::Login(hash));
            }
            return Ok(None);
        }
        self.keep_session(&hash, save.account);
        Ok(Some(save.account))
    }

    fn keep_session(&mut self, hash: &str, account: AccountId) {
        let save = LoginSave {
            account,
            expires: Some(now() + SESSION_SECONDS),
        };
        if let Some(queue) = &self.queue {
            queue.save(SaveKey::Login(hash.to_owned()), Save::Login(save.clone()));
        }
        self.sessions.insert(hash.to_owned(), save);
    }
}
