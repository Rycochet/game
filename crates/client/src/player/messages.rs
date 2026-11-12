//! The player marker, its system order, and asking for and announcing steps and travel.

use bevy::prelude::*;
use shared::movement::Step;
use shared::spot::Spot;

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

/// The player has taken a step on the world map, by `TakeStep` or
/// travelling: for the server. `moved` is how far it took them, and `to`
/// where it ended. (On levels and in the Hall, `Moved`.) Not sent when the position is set from elsewhere (the server's
/// word).
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stepped {
    pub step: Step,
    pub moved: (i64, i64),
    pub to: (i64, i64),
}

impl Stepped {
    /// The step from `from` that ended at `to`.
    pub(super) fn between(step: Step, from: (i64, i64), to: (i64, i64)) -> Self {
        Self {
            step,
            moved: (to.0 - from.0, to.1 - from.1),
            to,
        }
    }
}

/// The player walks freely this way this frame, on a level or in the Hall
/// (keys or the movement pad held): a grid direction (`x` east, `y` south),
/// one long. Replaces any journey.
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct Walking {
    pub x: f64,
    pub y: f64,
}

/// The player has moved straight from one spot to another on a level or in
/// the Hall (units): for the server, which checks it by the same rule. Not
/// sent when the position is set from elsewhere (the server's word).
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Moved {
    pub from: Spot,
    pub to: Spot,
}
