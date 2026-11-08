//! The login secret: the key email addresses are hashed with (HMAC-SHA256), so
//! a server keeps no address, only a key no one can work back without the
//! secret. Servers sharing players must share it (`HOW_LOGIN_SECRET`); without
//! that, one is made on the first run and kept in the data folder.

use std::fs;
use std::path::Path;

use bevy::log::info;
use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};

use crate::accounts::{EmailKey, hex};

/// Where the secret is kept, in the data folder.
const SECRET_FILE: &str = "login-secret.txt";

/// The key email addresses are hashed with.
pub(crate) struct LoginSecret(hmac::Key);

impl LoginSecret {
    /// From the secret's text.
    pub(crate) fn new(text: &str) -> Self {
        Self(hmac::Key::new(hmac::HMAC_SHA256, text.as_bytes()))
    }

    /// The server's secret: `given` (`HOW_LOGIN_SECRET`), kept in `data` the
    /// first time; otherwise the one kept there, made now if there is none. An
    /// error if `given` isn't the one kept: every account would be lost.
    pub(crate) fn load(data: &Path, given: Option<&str>) -> Result<Self, String> {
        let file = data.join(SECRET_FILE);
        let kept = match fs::read_to_string(&file) {
            Ok(text) => Some(text.trim().to_owned()).filter(|text| !text.is_empty()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(format!("Can't read {}: {error}", file.display())),
        };
        let given = given.map(str::trim).filter(|text| !text.is_empty());
        let secret = match (given, kept) {
            (Some(given), Some(kept)) if given != kept => {
                return Err(format!(
                    "HOW_LOGIN_SECRET isn't the secret in {}, which this server's accounts \
                     were made with: set it to that one, or remove the file to start again \
                     with no accounts found",
                    file.display()
                ));
            }
            (_, Some(kept)) => return Ok(Self::new(&kept)),
            (Some(given), None) => given.to_owned(),
            (None, None) => {
                let mut bytes = [0; 32];
                SystemRandom::new()
                    .fill(&mut bytes)
                    .map_err(|_| "the system can't provide secure random numbers".to_owned())?;
                info!(
                    "Made a login secret in {}: servers sharing players need the same one \
                     (HOW_LOGIN_SECRET)",
                    file.display()
                );
                hex(&bytes)
            }
        };
        fs::create_dir_all(data)
            .and_then(|()| fs::write(&file, format!("{secret}\n")))
            .map_err(|error| {
                format!("Can't keep the login secret in {}: {error}", file.display())
            })?;
        Ok(Self::new(&secret))
    }

    /// The key an email address is known by: the same however it is written
    /// (case, spaces around it).
    pub(crate) fn email_key(&self, email: &str) -> EmailKey {
        let address = email.trim().to_lowercase();
        let tag = hmac::sign(&self.0, address.as_bytes());
        let mut key = [0; 32];
        key.copy_from_slice(tag.as_ref());
        EmailKey(key)
    }
}
