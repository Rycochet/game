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

use shared::difficulty::{Difficulties, Difficulty};
use shared::protocol::OnlineSettings;

use super::{Controls, GraphicsPrefs, SavedServer, WindowPrefs};

/// The player's choices, loaded at start and saved whenever they change.
#[derive(Resource, Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Preferences {
    /// The page Item Pictures in the debug menu last showed, by its name.
    pub item_pictures: Option<String>,
    /// The debug menu's Dev Area as last set, for its panel and Regenerate.
    pub dev_area: Option<DevAreaPrefs>,
    /// The debug menu's Create Mob as last set.
    pub create_mob: Option<CreateMobPrefs>,
    /// How big the interface is drawn.
    pub interface_size: InterfaceSize,
    /// "Enable Touch": the interface is never drawn smaller than its own size,
    /// for fingers. A device with only a touch screen behaves as if this is on.
    pub touch: bool,
    /// The servers on other computers connected to, the latest first.
    pub servers: Vec<SavedServer>,
    /// "Online": the built-in server is opened to other players once playing on
    /// it (native only).
    pub online: bool,
    /// How it is opened (Server Options).
    pub online_settings: OnlineSettings,
    /// The difficulties the built-in server allows (Server Options' tickboxes),
    /// unless `HOW_DIFFICULTIES` says otherwise.
    pub server_difficulties: Difficulties,
    /// The player's own difficulty (Settings, Gameplay, before connecting): what
    /// a new character starts on, and a character with none of their own yet.
    pub difficulty: Difficulty,
    /// The window as last left (native): where, how big, and how shown.
    pub window: WindowPrefs,
    /// Graphics settings for speed and clutter.
    pub graphics: GraphicsPrefs,
    /// "Pause When Not in Focus": single player pauses while another window is
    /// in front.
    pub pause_unfocused: bool,
    /// Which keys do what (Settings, Controls); only changes are saved.
    pub controls: Controls,
    /// The movement pad as last left.
    pub step_pad: StepPadPrefs,
    /// "Don't Warn Again" on the warning that the game is drawn without a
    /// graphics card (`system_menu`).
    pub quiet_slow_drawing: bool,
    /// The servers this computer has a character on, as `connection` names them
    /// (the built-in one too), for Play.
    pub played_on: std::collections::BTreeSet<String>,
}

/// The movement pad as last left: open unless closed with its button, and
/// where it was dragged to, if it was.
#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq, Debug)]
#[serde(default)]
pub struct StepPadPrefs {
    pub closed: bool,
    /// Its top-left corner as a share of the window across and down, so it
    /// keeps its part of a window of another size.
    pub at: Option<(f32, f32)>,
}

/// Dev Area's choices, kept by name (a name no longer known is the default).
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, Debug)]
#[serde(default)]
pub struct DevAreaPrefs {
    pub size: u16,
    pub generator: String,
    /// "Default", "None" or a kind's name.
    pub mobs: String,
    pub count: u8,
    pub seed: String,
}

/// Create Mob's choices, kept by name.
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, Debug)]
#[serde(default)]
pub struct CreateMobPrefs {
    pub kind: String,
    /// "Default" or a feeling's name.
    pub attitude: String,
    pub gear: bool,
    pub still: bool,
    pub seed: u64,
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
        let mut preferences: Self = ron::from_str(text).unwrap_or_else(|error| {
            warn!("Can't read the saved preferences, so using the defaults: {error}");
            Self::default()
        });
        preferences.online_settings.move_off_old_default();
        preferences
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
