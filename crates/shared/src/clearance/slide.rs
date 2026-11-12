//! Sliding along what is in the way, for the client's movement: as far as a
//! move goes, then on along whichever of east-west or north-south goes further.

use super::check_segment;
use crate::creature::Size;
use crate::dungeon::Level;
use crate::spot::Spot;

/// How finely a move is cut when looking for how far it goes.
const PARTS: i64 = 1024;

/// Where a creature of `size` ends up moving from `from` towards `to`, walking
/// only (sliding never jumps): `to` if the move is legal, squeezing or not;
/// otherwise as far along it as it can go, then sliding on east-west or
/// north-south, whichever gets further (east-west on a tie), never more
/// squeezed than at `from`, so nobody is pressed into a wall by pushing
/// against it. `from` if it can't move at all.
pub fn slide(level: &Level, size: Size, from: Spot, to: Spot) -> Spot {
    let Some(start) = check_segment(level, size, from, from) else {
        return from;
    };
    let walks = |a: Spot, b: Spot| {
        check_segment(level, size, a, b).is_some_and(|passage| {
            passage.jumped.is_none() && passage.squeeze_per_mille <= start.squeeze_per_mille
        })
    };
    if check_segment(level, size, from, to).is_some_and(|passage| passage.jumped.is_none()) {
        return to;
    }
    let furthest = |a: Spot, b: Spot| {
        if walks(a, b) {
            return b;
        }
        // The last legal part found, by halving.
        let (mut good, mut bad) = (0, PARTS);
        while bad - good > 1 {
            let mid = (good + bad) / 2;
            if walks(a, a.towards(b, mid, PARTS)) {
                good = mid;
            } else {
                bad = mid;
            }
        }
        a.towards(b, good, PARTS)
    };
    let stop = furthest(from, to);
    let along_x = furthest(stop, Spot::new(to.x, stop.y));
    let along_y = furthest(stop, Spot::new(stop.x, to.y));
    if stop.distance_squared(along_y) > stop.distance_squared(along_x) {
        along_y
    } else {
        along_x
    }
}
