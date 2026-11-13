//! Which arms a town's middle has: towards the roads coming in, the rest by
//! its size.

use super::super::grid::{chance, roll};
use crate::rng::Rng;

/// Which arms (north, east, south, west) the middle of a town of `target`
/// buildings has. Small towns are mostly one straight road, big ones mostly
/// crossroads; every side in `sides` (where roads come in from outside) has
/// one, and the shape rolled only adds more: a straight road with one side
/// carries on out the far side, other arms are any of those left.
pub(super) fn choose_arms(rng: &mut Rng, target: i64, sides: [bool; 4]) -> [bool; 4] {
    let straight = (60 - target).max(10) as u32;
    let t_junction = 40;
    let shape = roll(rng, 0, 99) as u32;
    let shape = if target < 6 { 0 } else { shape };
    let rolled = if shape < straight {
        if chance(rng, 50) {
            [true, false, true, false]
        } else {
            [false, true, false, true]
        }
    } else if shape < straight + t_junction {
        let missing = roll(rng, 0, 3) as usize;
        std::array::from_fn(|i| i != missing)
    } else {
        [true; 4]
    };
    let given = sides.iter().filter(|&&side| side).count();
    if given == 0 {
        return rolled;
    }
    let wanted = rolled.iter().filter(|&&arm| arm).count();
    let mut arms = sides;
    if given == 1 && wanted == 2 {
        let side = sides.iter().position(|&side| side).unwrap_or(0);
        arms[(side + 2) % 4] = true;
        return arms;
    }
    for _ in given..wanted {
        let left: Vec<usize> = (0..4).filter(|&i| !arms[i]).collect();
        arms[left[roll(rng, 0, left.len() as i64 - 1) as usize]] = true;
    }
    arms
}
