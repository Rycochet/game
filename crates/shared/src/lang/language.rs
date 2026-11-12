//! Which languages there are.

use serde::{Deserialize, Serialize};

use super::english::English;
use super::rules::Rules;

/// A language the game can write text in. English only for now.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Language {
    #[default]
    English,
    // New languages go at the end: settings carry one by its place.
}

impl Language {
    pub const ALL: [Self; 1] = [Self::English];

    /// Its code: the first part of a locale such as `en-GB`.
    pub const fn code(self) -> &'static str {
        match self {
            Self::English => "en",
        }
    }

    /// Its own name for itself, for a list of languages to choose from.
    pub const fn name(self) -> &'static str {
        match self {
            Self::English => "English",
        }
    }

    /// The language for a locale as the system gives it (`en-GB`, `en_US`,
    /// `fr`), by its first part, or English when there is none for it.
    pub fn from_locale(locale: &str) -> Self {
        let code = locale.split(['-', '_', '.']).next().unwrap_or("");
        Self::ALL
            .into_iter()
            .find(|language| language.code().eq_ignore_ascii_case(code))
            .unwrap_or_default()
    }

    /// The rules in Rust for what its files can't say.
    pub fn rules(self) -> &'static dyn Rules {
        match self {
            Self::English => &English,
        }
    }
}
