//! What the player chose last time, kept between runs.
//!
//! - Native: in `client.ron` in the user's data folder, or where
//!   `HOW_PREFERENCES` says.
//! - Web: in the browser's local storage.
//!
//! Anything missing or unreadable is the default, so a new field or an old file
//! is never an error.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use shared::protocol::OnlineSettings;

use super::SavedServer;

/// The player's choices, loaded at start and saved whenever they change.
#[derive(Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Preferences {
    /// The page Item Pictures in the debug menu last showed, by its name.
    pub item_pictures: Option<String>,
    /// How big the interface is drawn.
    pub interface_size: InterfaceSize,
    /// "Enable touch": the interface is never drawn smaller than its own size,
    /// for fingers. A device with only a touch screen behaves as if this is on.
    pub touch: bool,
    /// The servers on other computers connected to, the latest first.
    pub servers: Vec<SavedServer>,
    /// The address of the one Play connects to, if it isn't the usual one
    /// (single player, or in a browser the page's server).
    pub last_server: Option<String>,
    /// "Online": the built-in server is opened to other players once playing on
    /// it (native only).
    pub online: bool,
    /// How it is opened (Server Options).
    pub online_settings: OnlineSettings,
}

/// How big the interface is drawn: its own size is 100%.
#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum InterfaceSize {
    /// Keeps its share of the window: 100% in a window 1920 × 1080 (logical
    /// pixels), more in a larger one.
    #[default]
    Auto,
    /// Always this percentage.
    Percent(u16),
}

impl Preferences {
    /// As saved, or the default.
    fn from_text(text: &str) -> Self {
        ron::from_str(text).unwrap_or_else(|error| {
            warn!("Can't read the saved preferences, so using the defaults: {error}");
            Self::default()
        })
    }

    fn to_text(&self) -> String {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()).unwrap_or_default()
    }
}

/// Keeps `Preferences` saved.
pub(super) fn save_preferences(preferences: Res<Preferences>) {
    if preferences.is_changed() && !preferences.is_added() {
        store::save(&preferences.to_text());
    }
}

/// Loads `Preferences` at start.
pub(super) fn load_preferences() -> Preferences {
    store::load().map_or_else(Preferences::default, |text| Preferences::from_text(&text))
}

#[cfg(target_arch = "wasm32")]
mod store {
    const KEY: &str = "hall-of-worlds-preferences";

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    pub(super) fn load() -> Option<String> {
        storage()?.get_item(KEY).ok().flatten()
    }

    pub(super) fn save(text: &str) {
        if let Some(storage) = storage() {
            let _ = storage.set_item(KEY, text);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod store {
    use std::path::PathBuf;
    use std::{env, fs, io};

    use bevy::prelude::*;

    /// `HOW_PREFERENCES`, or `Hall of Worlds/client/client.ron` in the user's data
    /// folder.
    pub(in crate::common) fn path() -> Option<PathBuf> {
        env::var_os("HOW_PREFERENCES").map(Into::into).or_else(|| {
            dirs::data_dir().map(|folder| {
                folder
                    .join("Hall of Worlds")
                    .join("client")
                    .join("client.ron")
            })
        })
    }

    pub(super) fn load() -> Option<String> {
        let path = path()?;
        match fs::read_to_string(&path) {
            Ok(text) => Some(text),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => {
                warn!("Can't read {}: {error}", path.display());
                None
            }
        }
    }

    pub(super) fn save(text: &str) {
        let Some(path) = path() else {
            return;
        };
        let written = path
            .parent()
            .map_or(Ok(()), fs::create_dir_all)
            .and_then(|()| fs::write(&path, text));
        if let Err(error) = written {
            warn!("Can't save preferences in {}: {error}", path.display());
        }
    }
}

/// Native: where the client keeps a file of its own, such as guest tokens:
/// beside the preferences.
#[cfg(not(target_arch = "wasm32"))]
pub fn client_file(name: &str) -> Option<std::path::PathBuf> {
    Some(store::path()?.with_file_name(name))
}

#[cfg(test)]
mod tests;
