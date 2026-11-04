//! Other creatures on the map, the Character panel's words, the player's speed, the
//! most they can carry and spent energy.

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
