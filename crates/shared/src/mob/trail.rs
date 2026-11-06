//! Where a wild mob has wandered lately: a trail out from its home and back,
//! from the secret seed and its number alone, so the land round a den is
//! already marked when a block first comes to life (the server lays the
//! scent along it, aged).

use crate::dungeon::Level;
use crate::movement::{Step, step_on_level};
use crate::rng::Seed;

#[cfg(test)]
#[path = "trail_tests.rs"]
mod tests;

/// Marks these hashes as being for trails.
const PURPOSE: i64 = 0x0074_726c;

/// The fewest steps a trail takes out from home.
pub const TRAIL_SHORTEST: i64 = 24;

/// The most: enough to reach well into the blocks round a den's.
pub const TRAIL_LONGEST: i64 = 96;

/// In this many steps out of 4 the heading turns an eighth either way.
const TURNS_IN_4: u64 = 1;

/// The eight headings, round the compass.
const HEADINGS: [(i64, i64); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// The cells a wild mob passed through on its way out from `home`, `home`
/// first, each one step (or jump) from the last, as far as it got before
/// turning back on the same way. It keeps a heading, turning now and then,
/// and goes round what is in its way; it stops early where it can't go on or
/// at the level's edge.
pub fn wild_trail(seed: Seed, number: u32, home: (i64, i64), level: &Level) -> Vec<(i64, i64)> {
    let roll = |what: i64| seed.at(&[PURPOSE, i64::from(number), what]);
    let span = (TRAIL_LONGEST - TRAIL_SHORTEST + 1) as u64;
    let steps = TRAIL_SHORTEST + (roll(0) % span) as i64;
    let mut heading = (roll(1) % 8) as usize;
    let mut cells = vec![home];
    let mut at = home;
    for step in 0..steps {
        let turn = roll(2 + step);
        if turn % 4 < TURNS_IN_4 {
            // Left or right, by the next bit.
            heading = (heading + if turn / 4 % 2 == 0 { 1 } else { 7 }) % 8;
        }
        // Straight on, else a little aside, else further aside.
        let next = [0, 1, 7, 2, 6].iter().find_map(|&aside| {
            let (dx, dy) = HEADINGS[(heading + aside) % 8];
            let to = step_on_level(level, at, Step::new(dx, dy)?)?;
            Some(((heading + aside) % 8, to))
        });
        let Some((went, to)) = next else {
            break;
        };
        heading = went;
        at = to;
        cells.push(to);
    }
    cells
}
