//! What town shops have in stock: how good it tends to be, how long it takes to
//! make, what sells, and catching up on all that when a shop is next opened. See
//! `docs/context/shared/shop.md`.
//!
//! All the numbers here are first guesses. Only the server calls this: the seed
//! is secret. It makes no items, as ids are the server's: it says what to keep
//! and what to add.

// Split into files (`cargo xtask lint` keeps files small).
mod catch_up;
mod sells;
#[allow(clippy::module_inception)]
mod shop;

pub use catch_up::{NewStock, Restocked, catch_up};
pub use sells::sells;
use sells::{base_minutes, sell_weight, steps_from_common};
pub use shop::Shop;

/// Marks these hashes as being for shops, apart from anything else from the seed.
const PURPOSE: i64 = 0x0073_686f;

const MINUTES_PER_DAY: u64 = 24 * 60;

/// How much time is caught up on. Past this, a shop with one of a kind in town
/// would have replaced everything it had.
pub const REPLACED_AFTER: u64 = 90 * MINUTES_PER_DAY;

/// How long a shop must have been left for it to be topped up on opening, so that
/// buying everything and coming straight back gets nothing.
pub const TOP_UP_AFTER: u64 = MINUTES_PER_DAY;

/// The least and most a shop aims to have, for the worst and best of shops.
const STOCK_AT_WORST: u64 = 5;
const STOCK_AT_BEST: u64 = 30;

/// The share of its count, in 100, below which a shop reopening is topped up.
const MINIMUM_SHARE: u64 = 80;

/// How a shop's make time is scaled, as a percentage, at the worst and best.
const SPEED_AT_WORST: u64 = 150;
const SPEED_AT_BEST: u64 = 50;

/// How near its door someone must be to deal with a shop, in cells.
pub const DOOR_REACH: i64 = 1;

#[cfg(test)]
mod tests;
