//! Which sides of a town its roads leave from, for laying the town out.

use super::RANGE;
use super::network::joined;
use crate::poi::{Poi, PoiMap};

/// Sides of a town (north, east, south, west) that roads leave from: the side
/// facing each town it is joined to, by the straight bearing between them.
///
/// From the road network's links, not the routed roads: the server finds those
/// later, on its road thread, and a town must never change once it has been
/// made. A link whose road turns out to be cut by water still counts. Nothing
/// for anything but a town.
pub fn town_sides(pois: &PoiMap, town: &Poi) -> [bool; 4] {
    let mut sides = [false; 4];
    if !town.kind.is_town() {
        return sides;
    }
    // Every town within `RANGE`: any third town that could part two joined
    // ones is nearer than that.
    let towns: Vec<Poi> = pois
        .in_area(
            (town.x - RANGE, town.y - RANGE),
            (town.x + RANGE, town.y + RANGE),
        )
        .into_iter()
        .filter(|poi| poi.kind.is_town())
        .collect();
    for other in &towns {
        if joined(town, other, &towns).is_some() {
            sides[side_towards(other.x - town.x, other.y - town.y)] = true;
        }
    }
    sides
}

/// The side (north, east, south, west) facing `(dx, dy)`: east or west if it
/// is further across than up or down.
fn side_towards(dx: i64, dy: i64) -> usize {
    if dx.abs() > dy.abs() {
        if dx > 0 { 1 } else { 3 }
    } else if dy > 0 {
        2
    } else {
        0
    }
}
