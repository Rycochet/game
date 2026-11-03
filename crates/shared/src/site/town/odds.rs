//! Weighted picks and odds blended by town size.

use super::{LARGEST, SMALLEST};
use crate::dungeon::Cell;
use crate::rng::Rng;

/// The road surfaces, poorest first, with their weights in the smallest and
/// largest towns.
pub(super) const ROADS: [(Cell, u64, u64); 6] = [
    (Cell::DirtTrack, 60, 2),
    (Cell::GravelTrack, 25, 8),
    (Cell::WoodenBoards, 12, 8),
    (Cell::Cobblestone, 3, 35),
    (Cell::Flagstones, 0, 25),
    (Cell::DressedStone, 0, 22),
];

/// A weight between `small` and `large` for a town of `size` buildings.
pub(super) fn blend(small: u64, large: u64, size: i64) -> u64 {
    let along = (size.max(0) as u64).clamp(SMALLEST, LARGEST) - SMALLEST;
    let span = LARGEST - SMALLEST;
    (small * (span - along) + large * along) / span
}

/// One of `choices` at random, more likely the more it weighs.
pub(super) fn pick<T: Copy>(rng: &mut Rng, choices: &[(T, u64)]) -> T {
    let total: u64 = choices.iter().map(|&(_, weight)| weight).sum();
    let mut left = rng.range_u64(0, total.max(1));
    for &(choice, weight) in choices {
        if left < weight {
            return choice;
        }
        left -= weight;
    }
    choices[0].0
}
