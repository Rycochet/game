//! Who can reach a bag on the ground: dropping joins the nearest bag in reach,
//! and anything in a bag in reach can be picked up.

use crate::creature::Size;
use crate::spot::Spot;

/// How far past the edge of their circle anyone can reach down to the ground,
/// in units (240 a cell). Enough for the smallest, standing in a cell's
/// middle, to reach a bag anywhere in that cell (its corners are 170 away),
/// and short of the next cell's middle for anyone Medium or smaller.
pub const ARM_REACH: u64 = 150;

/// How far from their middle someone of this size reaches on the ground.
pub fn reach(size: Size) -> u64 {
    size.radius().unsigned_abs() + ARM_REACH
}

/// Whether someone of this size standing at `at` can reach a bag at `bag`.
pub fn in_reach(size: Size, at: Spot, bag: Spot) -> bool {
    at.distance_squared(bag) <= u128::from(reach(size)).pow(2)
}

/// The bag in reach nearest `at`, of `bags`; the first of those as near.
pub fn nearest_in_reach(
    size: Size,
    at: Spot,
    bags: impl IntoIterator<Item = Spot>,
) -> Option<Spot> {
    bags.into_iter()
        .filter(|&bag| in_reach(size, at, bag))
        .min_by_key(|&bag| (at.distance_squared(bag), bag))
}
