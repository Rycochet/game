//! The words of the player's language (`shared::lang`), for text written from
//! what the server sends (quests' stories, givers' hints). English for now:
//! a setting will choose it.

use bevy::prelude::*;
use shared::lang::{Lang, Language};

/// The language files, parsed once.
#[derive(Resource)]
pub struct Wording(pub Lang);

impl Default for Wording {
    fn default() -> Self {
        Self(Lang::new(Language::English))
    }
}
