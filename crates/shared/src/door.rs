//! Doors: more than a cell. Every `Cell::Door` and `Cell::SecretDoor` of a site
//! level has a `Door` (rolled from the poi's seed where it stands, then changed
//! by whoever opens, closes or breaks it), and the level keeps its `Swing` for
//! the rules (`dungeon::Level::swing`): a closed door can't be walked through
//! and blocks sight and light, but not sound or smell. See
//! `docs/context/shared/door.md`.

use serde::{Deserialize, Serialize};

use crate::item::Quality;
use crate::material::Material;

// Split into files (`cargo xtask lint` keeps files small).
mod keys;
mod lock;
mod look;
mod placing;
mod start;
mod swing;

pub use keys::carries_key;
pub use lock::Lock;
pub use look::DoorLook;
pub use placing::{place_locks, reach_without};
pub use start::{door_is_outside, starting_door};
pub use swing::{Side, Swing};

/// How tough a Common wooden door is, in thousandths of a point.
const BASE_TOUGHNESS: u32 = 20_000;

/// What opening or closing a door costs: a straight step's time.
pub fn door_cost() -> u32 {
    crate::energy::site_step_cost((0, 0), (1, 0))
}

/// One door: how it hangs, and what it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Door {
    pub swing: Swing,
    /// Nearly always Common; better doors are for future generators.
    pub quality: Quality,
    /// What it is made of: wood for now.
    pub material: Material,
    pub style: DoorStyle,
    /// Its lock, if it has one: without one it can't be locked.
    #[serde(default)]
    pub lock: Option<Lock>,
    /// How much it has been harmed, kicked and the like, in thousandths of a
    /// point (`vitals::Points`): at its `toughness` it breaks.
    #[serde(default)]
    pub damage: u32,
}

impl Door {
    /// How much harm breaks it, in thousandths of a point: by what it is made
    /// of and how well. First guesses: a Common wooden door takes about ten kicks.
    pub fn toughness(&self) -> u32 {
        let material = match self.material.category() {
            crate::material::Category::Metal => 3,
            _ => 1,
        };
        // Poor half as tough as Common, each better one half as much again.
        let rank = Quality::ALL.iter().position(|&q| q == self.quality);
        let rank = rank.unwrap_or(1) as u32 + 1;
        BASE_TOUGHNESS * material * rank / 2
    }

    /// Whether it is locked: closed, with its lock turned.
    pub fn is_locked(&self) -> bool {
        self.lock.is_some_and(|lock| lock.locked)
    }

    /// What players are told of it at `(x, y)`.
    pub fn look(&self, x: i64, y: i64) -> DoorLook {
        DoorLook {
            x,
            y,
            swing: self.swing,
            material: self.material,
            style: self.style,
            lock: self.lock.is_some(),
        }
    }
}

/// How a door looks, for its model. Only one so far: styles are a future
/// research task.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoorStyle {
    #[default]
    Plain,
}

#[cfg(test)]
mod tests;
