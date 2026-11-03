//! The player marker, its system order, and asking for and announcing steps and travel.

use bevy::prelude::*;
use shared::movement::Step;

/// Ordering for anything that moves the player. Put systems that send
/// `TakeStep` or `TravelTo` in `Input`; systems that draw the player's
/// position (or read `Stepped`) should run `.after(PlayerSystems::Travel)`.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PlayerSystems {
    Input,
    Travel,
}

/// Marks the entity the keyboard controls.
#[derive(Component)]
pub struct Player;

/// Ask the player to travel to a grid point, by the quickest way, as far
/// towards it as they can get: in a dungeon, through the cells they know; on
/// the world map, round water and easing off on slopes. A new request replaces
/// the current one, starting from wherever the player is.
#[derive(Message, Clone, Copy, Debug)]
pub struct TravelTo {
    pub x: i64,
    pub y: i64,
}

/// The player has finished travelling to a grid point (sent once, when it
/// gets there). Not sent for keyboard steps or other direct moves, nor for a
/// journey that stopped short of where it was asked to go.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arrived {
    pub x: i64,
    pub y: i64,
}

/// Ask the player to take a step (keyboard, auto walk, dev tools): taken
/// unless it leads into a wall in a site.
#[derive(Message, Clone, Copy, Debug)]
pub struct TakeStep(pub Step);

/// The player has taken a step, by `TakeStep` or travelling: for the server.
/// `moved` is how far it took them, which is two cells for a jump over liquid.
/// Not sent when the position is set from elsewhere (the server's word).
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stepped {
    pub step: Step,
    pub moved: (i64, i64),
}

impl Stepped {
    /// The step from `from` that ended at `to`.
    pub(super) fn between(step: Step, from: (i64, i64), to: (i64, i64)) -> Self {
        Self {
            step,
            moved: (to.0 - from.0, to.1 - from.1),
        }
    }
}
