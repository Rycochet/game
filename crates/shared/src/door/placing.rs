//! Choosing which of a level's doors are locked, and who carries each key, so
//! that every key can be reached without going through any locked door: never
//! behind its own door, and never two keys each behind the other's door.

use std::collections::{BTreeSet, VecDeque};

use crate::dungeon::{Cell, Level};
use crate::movement::{Step, step_on_level};
use crate::rng::Seed;

/// Out of 100: how many closed doors a generator locks, if their keys can be
/// placed. Secret doors far less often.
const LOCKED_PERCENT: u64 = 10;
const SECRET_LOCKED_PERCENT: u64 = 2;

/// The closed doors of `level` to lock (its swings set), each with the holder
/// (from `holders`: a number and where it stands) to carry its key, in the
/// order chosen. Reach is from `starts` (where anyone comes onto the level),
/// with every locked door and every secret door shut: a holder is only given a
/// key while it can still be reached that way, and a door only locked while
/// every key already given can, so no key is ever shut behind a locked door.
pub fn place_locks(
    seed: Seed,
    level: &Level,
    starts: &[(i64, i64)],
    holders: &[(u32, (i64, i64))],
) -> Vec<((i64, i64), u32)> {
    let seed = seed.derive("locks");
    let mut chosen: Vec<((i64, i64), u32)> = Vec::new();
    let doors: Vec<((i64, i64), bool)> = level
        .swings()
        .filter(|&(_, swing)| swing.is_shut())
        .map(|((x, y), _)| ((x, y), level.cell(x, y) == Cell::SecretDoor))
        .collect();
    for ((x, y), secret) in doors {
        let roll = seed.at(&[x, y]);
        let percent = if secret {
            SECRET_LOCKED_PERCENT
        } else {
            LOCKED_PERCENT
        };
        if roll % 100 >= percent {
            continue;
        }
        let mut shut: BTreeSet<(i64, i64)> = chosen.iter().map(|&(at, _)| at).collect();
        shut.insert((x, y));
        let reach = reach_without(level, starts, &shut);
        let keys_safe = chosen.iter().all(|&(_, holder)| {
            holders
                .iter()
                .any(|&(n, at)| n == holder && reach.contains(&at))
        });
        let free: Vec<u32> = holders
            .iter()
            .filter(|(_, at)| reach.contains(at))
            .map(|&(number, _)| number)
            .collect();
        if !keys_safe || free.is_empty() {
            continue;
        }
        let holder = free[((roll / 100) % free.len() as u64) as usize];
        chosen.push(((x, y), holder));
    }
    chosen
}

/// The cells reachable from `starts` by legal steps, with `shut` doors and
/// every secret door counting as closed (other doors as open, as anyone may
/// open them).
pub fn reach_without(
    level: &Level,
    starts: &[(i64, i64)],
    shut: &BTreeSet<(i64, i64)>,
) -> BTreeSet<(i64, i64)> {
    let mut open = level.with_all_doors_open();
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if level.cell(x, y) == Cell::SecretDoor || shut.contains(&(x, y)) {
                open.set_swing(x, y, crate::door::Swing::Closed);
            }
        }
    }
    let mut found: BTreeSet<(i64, i64)> = starts.iter().copied().collect();
    let mut queue: VecDeque<(i64, i64)> = starts.iter().copied().collect();
    while let Some(at) = queue.pop_front() {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let Some(step) = Step::new(dx, dy) else {
                    continue;
                };
                if let Some(next) = step_on_level(&open, at, step)
                    && found.insert(next)
                {
                    queue.push_back(next);
                }
            }
        }
    }
    found
}
