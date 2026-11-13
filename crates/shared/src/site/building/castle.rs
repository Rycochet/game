//! Castles (owner's design, 2026-11-13): a keep (the building itself) in a
//! courtyard with a pond, inside an outer wall of stone or dressed blocks with
//! towers on every corner and spaced along it, and one gate of two doors in
//! the middle of the south side, the way in, between two short towers. Often a
//! moat round the outside, with a bridge from the gate.
//!
//! The castle's size is rolled first (`CastleSize`) and the keep is what fits
//! inside. Each tower stands on its own with its own stairs; its floors above
//! the ground are on the keep's levels, apart from the keep's rooms (no wall
//! walk joins them). A ruin's wall has gaps and its towers may have lost their
//! tops; everything is still reached from the gate.

// Split into files (`cargo xtask lint` keeps files small).
mod ground;
mod options;
mod plan;
mod poi_castle;
mod size;
mod tower;
mod towers;
mod upper;

pub use options::CastleOptions;
pub(in crate::site) use plan::Castle;
pub use poi_castle::PoiCastle;
pub(in crate::site) use size::CastleSize;
use tower::Tower;
pub use towers::{GATE_TOWER_SIZE, TOWER_SIZE, TowerSpot, tower_spots};

use super::super::SiteLevel;
use crate::dungeon::{Level, RoomId};
use crate::rng::Seed;

impl Castle {
    /// Level `index` of a castle (0 the ground floor), from the keep's.
    pub(in crate::site) fn level(&self, seed: Seed, index: u32, floor: SiteLevel) -> SiteLevel {
        if index == 0 {
            self.surround(seed, floor)
        } else {
            self.raise(index, floor)
        }
    }
}

/// A room number no cell of `level` has yet.
fn next_room(level: &Level) -> RoomId {
    let (min_x, min_y, width, height) = level.bounds();
    let most = (min_y..min_y + height)
        .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
        .filter_map(|(x, y)| level.room(x, y))
        .map(|room| room.0 + 1)
        .max();
    RoomId(most.unwrap_or(0))
}

#[cfg(test)]
mod tests;
