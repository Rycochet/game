//! How near a poi a player must stand on the world map to go into it (by
//! Down, or a click on it): the same rule on both sides.

use super::Poi;

/// How far apart, in world map points, the middles of the player's token and
/// a poi's may be for the player to go into it: both are 16 points across,
/// and at 11 apart about a fifth of their area overlaps (owner).
pub const ENTER_DISTANCE: i64 = 11;

/// Whether a player at `(x, y)` on the world map may go into `poi`: within
/// `ENTER_DISTANCE` of its middle (a straight line, the near way round a
/// planet `width` points round).
pub fn can_enter_from(poi: &Poi, (x, y): (i64, i64), width: i64) -> bool {
    let dx = (poi.x - x).rem_euclid(width);
    let dx = dx.min(width - dx);
    let dy = poi.y - y;
    dx * dx + dy * dy <= ENTER_DISTANCE * ENTER_DISTANCE
}

#[cfg(test)]
mod tests;
