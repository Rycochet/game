//! How planets are laid out along each sixteenth of the Hall.
//!
//! Entrance `k` of every planet is in the `k`-th sixteenth, at place
//! `(MULTIPLIERS[k] × seed + OFFSETS[k]) mod PLANETS` within it. The
//! multipliers are odd, so this is a shuffle: every seed has one place and
//! every place one seed. It is undone by multiplying by `STEPS[k]`, the
//! multiplier's inverse, which is also how much the seed changes from one
//! place to the next. The steps differ enough between sixteenths that two
//! planets are neighbours in at most one of them (checked by the tests).

use super::{PLANET_BITS, PLANETS};
use crate::world_gate::WORLD_GATES;

const MASK: u64 = PLANETS as u64 - 1;

/// SplitMix64's finaliser, here as a `const fn` for the tables.
const fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// The inverse of an odd number, modulo `PLANETS`: Newton's step doubles the
/// bits that are right each time, from the 3 an odd number starts with.
const fn inverse(odd: u64) -> u64 {
    let mut x = odd;
    let mut i = 0;
    while i < 4 {
        x = x.wrapping_mul(2u64.wrapping_sub(odd.wrapping_mul(x)));
        i += 1;
    }
    x & MASK
}

const fn table(salt: u64, odd: bool) -> [u64; WORLD_GATES] {
    let mut out = [0; WORLD_GATES];
    let mut k = 0;
    while k < WORLD_GATES {
        let value = mix(salt ^ (k as u64 + 1)) & MASK;
        out[k] = if odd { value | 1 } else { value };
        k += 1;
    }
    out
}

/// How much the seed changes from one place to the next, per sixteenth.
pub(super) const STEPS: [u64; WORLD_GATES] = table(0x4841_4c4c_5354_4550, true);

const fn inverses(steps: [u64; WORLD_GATES]) -> [u64; WORLD_GATES] {
    let mut out = [0; WORLD_GATES];
    let mut k = 0;
    while k < WORLD_GATES {
        out[k] = inverse(steps[k]);
        k += 1;
    }
    out
}

pub(super) const MULTIPLIERS: [u64; WORLD_GATES] = inverses(STEPS);
const OFFSETS: [u64; WORLD_GATES] = table(0x4841_4c4c_4f46_4653, false);

/// The place of a seed's gate within its entrance's sixteenth.
pub(super) fn place_of(entrance: usize, seed: u32) -> u32 {
    ((MULTIPLIERS[entrance] * u64::from(seed) + OFFSETS[entrance]) & MASK) as u32
}

/// The seed whose gate is at this place within the sixteenth.
pub(super) fn seed_at(entrance: usize, place: u32) -> u32 {
    let from_start = (u64::from(place) + PLANETS as u64 - OFFSETS[entrance]) & MASK;
    ((STEPS[entrance] * from_start) & MASK) as u32
}

const _: () = assert!(PLANET_BITS + 4 == 32 && WORLD_GATES == 16);
