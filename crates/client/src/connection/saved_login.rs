//! The account kept for each server: the session token it gave on logging in
//! through a provider (`ServerMessage::LoggedIn`), to log in with next time,
//! and the email address, to show in the menus.
//!
//! - Native: `logins.ron` beside the preferences, by `host:port` (as guest
//!   tokens, `guest::storage_key`).
//! - Web: the browser's local storage, by the server's address.

use serde::{Deserialize, Serialize};
use shared::protocol::SessionToken;

use super::socket::Target;

/// An account kept for a server (no `Debug`, so the address can't reach a
/// log).
#[derive(Clone, PartialEq, Eq)]
pub(super) struct SavedLogin {
    pub session: SessionToken,
    pub email: String,
}

/// As kept: the token as text.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Kept {
    session: String,
    email: String,
}

impl Kept {
    fn from(login: &SavedLogin) -> Self {
        Self {
            session: login.session.to_hex(),
            email: login.email.clone(),
        }
    }

    fn login(self) -> Option<SavedLogin> {
        Some(SavedLogin {
            session: SessionToken::from_hex(&self.session)?,
            email: self.email,
        })
    }
}

/// The account kept for `target`, if any (none for a built-in server).
pub(super) fn saved(target: &Target) -> Option<SavedLogin> {
    let key = key(target)?;
    store::read(&key)?.login()
}

/// Keeps the account for `target`, in place of any other.
pub(super) fn keep(target: &Target, login: &SavedLogin) {
    if let Some(key) = key(target) {
        store::write(&key, Some(Kept::from(login)));
    }
}

/// Forgets the account kept for `target`.
pub(super) fn forget(target: &Target) {
    if let Some(key) = key(target) {
        store::write(&key, None);
    }
}

fn key(target: &Target) -> Option<String> {
    match target {
        #[cfg(not(target_arch = "wasm32"))]
        Target::Embedded => None,
        #[cfg(not(target_arch = "wasm32"))]
        Target::Url { url, .. } => Some(super::guest::storage_key(url)),
        #[cfg(target_arch = "wasm32")]
        Target::Url { url } => Some(format!("hall-of-worlds-login {url}")),
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod store {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use std::{fs, io};

    use bevy::log::warn;

    use super::Kept;

    /// Secrets, so not in the preferences, which people may share or edit.
    fn file() -> Option<PathBuf> {
        crate::common::client_file("logins.ron")
    }

    pub fn read(key: &str) -> Option<Kept> {
        read_from(&file()?).ok()?.remove(key)
    }

    pub fn write(key: &str, kept: Option<Kept>) {
        let Some(file) = file() else {
            return;
        };
        if let Err(error) = write_to(&file, key, kept) {
            warn!("Can't keep the login in {}: {error}", file.display());
        }
    }

    pub(super) fn read_from(file: &Path) -> io::Result<BTreeMap<String, Kept>> {
        match fs::read_to_string(file) {
            Ok(text) => ron::from_str(&text).map_err(io::Error::other),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(error) => Err(error),
        }
    }

    /// A file that can't be read is left alone rather than lose the rest.
    pub(super) fn write_to(file: &Path, key: &str, kept: Option<Kept>) -> io::Result<()> {
        let mut all = read_from(file)?;
        match kept {
            Some(kept) => all.insert(key.to_owned(), kept),
            None => all.remove(key),
        };
        let pretty = ron::ser::PrettyConfig::default();
        let text = ron::ser::to_string_pretty(&all, pretty).map_err(io::Error::other)?;
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder)?;
        }
        let new = file.with_extension("ron.new");
        fs::write(&new, text)?;
        fs::rename(new, file)?;
        // Only this user's, where a system can say so.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(file, fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod store {
    use super::Kept;

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    pub fn read(key: &str) -> Option<Kept> {
        let text = storage()?.get_item(key).ok().flatten()?;
        ron::from_str(&text).ok()
    }

    pub fn write(key: &str, kept: Option<Kept>) {
        let Some(storage) = storage() else {
            return;
        };
        let _ = match kept.and_then(|kept| ron::to_string(&kept).ok()) {
            Some(text) => storage.set_item(key, &text),
            None => storage.remove_item(key),
        };
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
