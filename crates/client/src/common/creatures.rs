//! Other creatures on the map, the Character panel's words, the player's speed, the
//! most they can carry, spent energy and making a character.

use bevy::prelude::*;

/// Another creature on the map: a mob or another player, as the server has told
/// this player of it.
#[derive(Component, Clone, Copy, Debug)]
pub struct Other {
    pub id: shared::id::Id,
    pub kind: shared::creature::CreatureKind,
    pub feeling: shared::sheet::Feeling,
}

/// The words on the Character panel saying what the character is like, which
/// `character` fills in.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SheetText;

/// The Character panel's Abilities page, which `abilities` fills in.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct AbilitiesPage;

/// The Character panel's Skills page, which `skills_page` fills in.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SkillsPage;

/// How much energy the player gains per tick (`shared::energy`), from what they are
/// and what they carry. The server keeps its own.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerSpeed(pub u32);

impl Default for PlayerSpeed {
    fn default() -> Self {
        Self(shared::energy::NORMAL_SPEED)
    }
}

/// The most the player can carry, in grams, from what they are (`Sheet::capacity`),
/// which `character` keeps up to date.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CarryLimit(pub u32);

/// The player has done something that takes this much energy (`shared::energy`).
/// The server works out the same, and keeps its own.
#[derive(Message, Clone, Copy, Debug)]
pub struct SpentEnergy(pub u32);

/// What the player spent energy on was called off before it was done: all
/// they still owe comes back but for this much (`Energy::cancel`).
#[derive(Message, Clone, Copy, Debug)]
pub struct CancelledEnergy(pub u32);

/// The server asked for a new character to be made (`ServerMessage::Characters`
/// with none to choose): `character_create` shows its screen until it is sent,
/// and the loading screen leaves it the input. `connection` sets it, and clears it
/// on hanging up.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MakingCharacter(pub bool);

/// Whether a new character is made with `character_create`'s screen, which
/// `connection` tells the server in `Hello`. Off, the server makes a plain one
/// of the usual kind: `dev_tools` turns it off for runs that skip the menu
/// (`HOW_DEBUG_SKIP_CREATION`).
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MakesCharacters(pub bool);

impl Default for MakesCharacters {
    fn default() -> Self {
        Self(true)
    }
}
