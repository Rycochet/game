//! Straightening a route of cells' middles to any angle.

use super::sized_path::move_cost;
use crate::clearance::RoomMap;
use crate::movement::LevelWay;
use crate::spot::Spot;

/// The same route with corners cut: from each spot, straight on to the
/// furthest later one that a single legal walk (no jump) reaches for no more
/// than going through the spots between, priced along `way` for `rooms`'s
/// size, so squeezes and slopes are never made worse. Jumps stay as they are.
/// The first and last spots are kept.
pub fn straighten_spots(way: &LevelWay, rooms: &RoomMap, spots: &[Spot]) -> Vec<Spot> {
    let Some(&first) = spots.first() else {
        return Vec::new();
    };
    let cost = |a: Spot, b: Spot| move_cost(way, rooms, (a, b), true).unwrap_or(u64::MAX);
    let mut out = vec![first];
    let mut i = 0;
    while i + 1 < spots.len() {
        let mut j = i + 1;
        // What going through the spots costs, from `i` to `j + 1`.
        let mut through = cost(spots[i], spots[j]);
        while j + 1 < spots.len() {
            through = through.saturating_add(cost(spots[j], spots[j + 1]));
            match move_cost(way, rooms, (spots[i], spots[j + 1]), false) {
                Some(direct) if direct <= through => j += 1,
                _ => break,
            }
        }
        out.push(spots[j]);
        i = j;
    }
    out
}
