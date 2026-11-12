//! Whether a blow (or a spell) reaches: a distance between creatures in 3D,
//! in units (`spot::Spot`, 240 a cell), with a clear line between them (free
//! positions, PR 7).

use crate::creature::Size;
use crate::dungeon::{Level, spots_are_clear};
use crate::relief::Relief;
use crate::spot::{Spot, root};

/// How far past the edge of its circle a creature's arms reach, in units, for
/// each quarter of its size (`Size::quarters`): 180 for anyone Medium. Generous,
/// as circles are wider than bodies: bare hands just reach a Medium neighbour
/// standing in the next cell diagonally (339 units between their middles).
/// First guess.
pub const ARM_PER_QUARTER: u64 = 45;

/// Millimetres of a weapon to a unit of reach: a cell is about 1.9 metres,
/// as creatures' models are drawn.
pub const MM_PER_UNIT: u32 = 8;

/// A creature where it stands, as far as reaching it goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Body {
    /// The middle of its circle.
    pub at: Spot,
    /// The ground's height under it (`relief::Relief::at`), in units: 0 on
    /// flat ground.
    pub height: i32,
    pub size: Size,
}

impl Body {
    /// Someone of `size` standing at `at` on ground shaped as `relief` (`None`
    /// where it is flat).
    pub fn on(relief: Option<&Relief>, at: Spot, size: Size) -> Self {
        let height = relief.map_or(0, |relief| relief.at(at.x, at.y));
        Self { at, height, size }
    }
}

/// How far past its circle's edge someone of this size reaches with no weapon.
pub fn arm_reach(size: Size) -> u64 {
    u64::from(size.quarters()) * ARM_PER_QUARTER
}

/// How far apart two creatures' middles may be for a blow to land: the
/// attacker's radius, its arm's reach, its weapon's length (`Weapon::reach`)
/// and the target's radius.
pub fn reach_between(attacker: Size, weapon: u64, target: Size) -> u64 {
    let radius = |size: Size| size.radius().unsigned_abs();
    radius(attacker) + arm_reach(attacker) + weapon + radius(target)
}

/// The distance between two creatures' middles in 3D, rounded down to a unit.
pub fn distance(from: &Body, to: &Body) -> u64 {
    let dz = i128::from(to.height) - i128::from(from.height);
    root(from.at.distance_squared(to.at) + (dz * dz) as u128)
}

/// Whether `to` is within `range` of `from` (between their middles, in 3D),
/// with nothing between them a blow or a spell couldn't pass: the line sight
/// goes by.
pub fn within(level: &Level, from: &Body, to: &Body, range: u64) -> bool {
    distance(from, to) <= range && spots_are_clear(level, from.at, to.at)
}

/// Whether `attacker`, with a weapon reaching `weapon` units (`Weapon::reach`),
/// can strike `target`.
pub fn in_reach(level: &Level, attacker: &Body, weapon: u64, target: &Body) -> bool {
    within(
        level,
        attacker,
        target,
        reach_between(attacker.size, weapon, target.size),
    )
}
