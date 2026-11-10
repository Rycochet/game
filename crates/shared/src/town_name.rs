//! Towns' names: a beginning and an ending suiting how high the town stands,
//! sometimes with one extra word before or after suiting its size ("Little",
//! "Village of"; "Hamlet", "Cross"). The same seed and height always give the
//! same name.
//!
//! The name has its own seed, derived from the poi's map seed, so it draws
//! nothing from the town generator's sequence and no site changes. Only the
//! server works names out (the map seed is secret) and sends the name itself.

mod parts;

use crate::poi::PoiKind;
use crate::rng::{Rng, Seed};
use parts::{Affix, ENDINGS, STARTS, affixes};

/// How high a town stands, which makes some endings likelier: "-mouth" and
/// "-ford" low down, "-fell" and "-crag" high up. Any ending can still turn up
/// anywhere, as coasts move over time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Altitude {
    Low,
    Middle,
    High,
}

/// Below this many metres above the sea a town is low.
pub const LOW_BELOW: i32 = 150;
/// From this many metres above the sea a town is high.
pub const HIGH_FROM: i32 = 800;

impl Altitude {
    /// The band a height in metres above the sea falls in.
    pub fn of_metres(metres: i32) -> Self {
        if metres < LOW_BELOW {
            Self::Low
        } else if metres < HIGH_FROM {
            Self::Middle
        } else {
            Self::High
        }
    }
}

/// How much likelier an ending is in its own band than in the others.
const OWN_BAND_WEIGHT: u64 = 6;
/// The share of names with an extra word, before or after.
const AFFIX_PERCENT: u64 = 50;

/// What a town is called, or `None` if `kind` isn't a town. `seed` is the poi's
/// map seed; `metres` how high it stands above the sea.
pub fn town_name(kind: PoiKind, seed: Seed, metres: i32) -> Option<String> {
    let affixes = affixes(kind)?;
    let mut rng = seed.derive("town name").rng();
    let base = base_name(&mut rng, Altitude::of_metres(metres));
    if rng.range_u64(0, 100) >= AFFIX_PERCENT {
        return Some(base);
    }
    let affix = affixes[rng.range_u64(0, affixes.len() as u64) as usize];
    Some(match affix {
        Affix::Before(word) => format!("{word} {base}"),
        Affix::After(word) => format!("{base} {word}"),
    })
}

/// A beginning and an ending, such as "Thornbury". A few tries for one that
/// reads well: no letter written twice across the join ("Elmmouth"), none three
/// times running.
fn base_name(rng: &mut Rng, altitude: Altitude) -> String {
    let weights: Vec<(usize, u64)> = ENDINGS
        .iter()
        .enumerate()
        .map(|(index, (_, band))| {
            let weight = if *band == altitude {
                OWN_BAND_WEIGHT
            } else {
                1
            };
            (index, weight)
        })
        .collect();
    let mut name = String::new();
    for _ in 0..8 {
        let start = STARTS[rng.range_u64(0, STARTS.len() as u64) as usize];
        let ending = ENDINGS[rng.pick_weighted(&weights)].0;
        name = format!("{start}{ending}");
        if reads_well(start, ending) {
            break;
        }
    }
    name
}

/// Whether a beginning and an ending join without a doubled letter at the
/// join or a letter three times running.
fn reads_well(start: &str, ending: &str) -> bool {
    let last = start.chars().last().map(|c| c.to_ascii_lowercase());
    if last == ending.chars().next() {
        return false;
    }
    let letters: Vec<char> = format!("{start}{ending}").to_lowercase().chars().collect();
    !letters
        .windows(3)
        .any(|three| three[0] == three[1] && three[1] == three[2])
}

#[cfg(test)]
mod tests;
