//! Scent laid ahead: when a block's mobs are first made, each has lately
//! walked a trail out from its home and back (`shared::mob::wild_trail`), so
//! their marks lie round the den, into the blocks round it, already aged.

use shared::mob::wild_trail;
use shared::rng::Seed;

use super::super::scent::Went;
use super::super::{Mob, Population};

/// How long ago a mob got home from its trail, in action ticks: 30 seconds.
const HOME_AGO: u64 = 30 * 20;

/// How long each step home took, in action ticks: a second, an easy walk.
const STEP_TICKS: u64 = 20;

/// Lays the trails of mobs just made on their area's level, as walked home
/// at `now`: the freshest marks by the den, the oldest at the far end, each
/// pointing the way home, the way a nose would follow them.
pub(super) fn lay_trails(population: &mut Population, seed: Seed, mobs: &[Mob], now: u64) {
    let ended = now.saturating_sub(HOME_AGO);
    for mob in mobs {
        let trail = wild_trail(seed, mob.number, mob.home, &population.level);
        // Far end first, so where it crossed its own path the fresher mark stays.
        for (steps, pair) in trail.windows(2).enumerate().rev() {
            let (nearer, further) = (pair[0], pair[1]);
            let went = Went::Towards((nearer.0 - further.0) as i8, (nearer.1 - further.1) as i8);
            let laid = ended.saturating_sub((steps as u64 + 1) * STEP_TICKS);
            population
                .scents
                .lay(mob.id, further, went, mob.scent, laid);
        }
    }
}
