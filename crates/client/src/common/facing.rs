//! Which way creatures face (`shared::facing`), and what the player sees by
//! it: `player` turns the player, `entities` others, `view3d` turns their
//! models, `dungeon_map` draws the player's cone.

use bevy::prelude::*;
use shared::facing::{Cone, Field, Heading};
use shared::spot::Spot;

/// The way a creature faces: the player's as they turn and move, others' as
/// the server tells. Models turn smoothly towards it.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Facing(pub Heading);

/// Turn the player to face this point on their level (units), without
/// moving: a right click or long press on the map (where a menu of choices
/// will open), or using an ability on someone.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaceTowards(pub Spot);

/// A right click, or a finger held still, on the map at this place on the
/// screen (logical pixels): where a menu of choices will open. `target` sends
/// it; `dungeon_map` finds the ground there and turns the player to it.
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct PointedAtMap(pub Vec2);

/// What the player sees round them, by kind and Observation (`character`
/// keeps it).
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerField(pub Field);

impl Default for PlayerField {
    fn default() -> Self {
        Self(Field::ALL_ROUND)
    }
}

/// The player's field of view where they stand on a level or in the Hall;
/// `None` on the world map, which is seen all round. `player` keeps it.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayerCone(pub Option<Cone>);
