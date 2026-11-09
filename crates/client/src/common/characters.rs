//! Choosing a character: what the server offered (`CharacterSelect`), what Play
//! and Characters connect for (`PlayFor`), and what the player chose
//! (`CharacterChoice`). `connection` fills and acts on them; `character_select`
//! shows them.

use bevy::prelude::*;
use shared::protocol::{CharacterId, ListedCharacter};

/// What connecting is for: the startup menu's Play (the character played last
/// on that server) or Characters (the screen for choosing).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayFor {
    #[default]
    Latest,
    Choose,
}

/// The account's characters as the server last offered them, and whether the
/// screen for choosing between them is up. `reached`: it has been shown since
/// connecting, so making a new character goes back to it when called off.
#[derive(Resource, Clone, Debug, Default)]
pub struct CharacterSelect {
    pub open: bool,
    pub reached: bool,
    pub list: Vec<ListedCharacter>,
    pub can_create: bool,
}

impl CharacterSelect {
    /// The list, the latest played (or else made) first.
    pub fn sorted(&self) -> Vec<&ListedCharacter> {
        let mut sorted: Vec<&ListedCharacter> = self.list.iter().collect();
        sorted.sort_by_key(|c| std::cmp::Reverse((c.summary.last_played, c.summary.created)));
        sorted
    }
}

/// Whether Play's server has a character of this player's to go back to, as far
/// as this computer knows (it has played there and not deleted them all): Play
/// is only enabled if so.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayableHere(pub bool);

/// What the player chose on the screens for choosing and making characters.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterChoice {
    Play(CharacterId),
    Delete(CharacterId),
    /// On to making a new one.
    New,
    /// Making one called off: back to choosing, or the startup menu.
    CancelNew,
    /// Back to the startup menu.
    Cancel,
}
