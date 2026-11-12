//! Straightening a route of cells' middles to any angle.

use crate::clearance::{check_segment, segment_cost};
use crate::creature::Size;
use crate::dungeon::Level;
use crate::spot::Spot;

/// The same route with corners cut: from each spot, straight on to the
/// furthest later one that a single legal walk (no jump) reaches for no more
/// than going through the spots between, so squeezes are never made worse.
/// Jumps stay as they are. The first and last spots are kept.
pub fn straighten_spots(level: &Level, size: Size, spots: &[Spot]) -> Vec<Spot> {
    let Some(&first) = spots.first() else {
        return Vec::new();
    };
    let walk = |a: Spot, b: Spot| {
        check_segment(level, size, a, b)
            .filter(|passage| passage.jumped.is_none())
            .and_then(|_| segment_cost(level, size, a, b))
    };
    let mut out = vec![first];
    let mut i = 0;
    while i + 1 < spots.len() {
        let mut j = i + 1;
        // What going through the spots costs, from `i` to `j + 1`.
        let mut through = segment_cost(level, size, spots[i], spots[j]).unwrap_or(u64::MAX);
        while j + 1 < spots.len() {
            through = through.saturating_add(
                segment_cost(level, size, spots[j], spots[j + 1]).unwrap_or(u64::MAX),
            );
            match walk(spots[i], spots[j + 1]) {
                Some(direct) if direct <= through => j += 1,
                _ => break,
            }
        }
        out.push(spots[j]);
        i = j;
    }
    out
}
