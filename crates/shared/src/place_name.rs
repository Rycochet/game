//! Places' names. Towns: a beginning and an ending suiting how high the town
//! stands, sometimes with one extra word before or after suiting its size
//! ("Little", "Village of"; "Hamlet", "Cross"). Castles and forts: always
//! their kind in front and nothing after ("Castle Ravenhold"), from a wider
//! set of beginnings and endings. The same seed and height always give the
//! same name, ruined or not.
//!
//! The name has its own seed, derived from the poi's map seed, so it draws
//! nothing from the site generator's sequence and no site changes. Only the
//! server works names out (the map seed is secret) and sends the name itself.

mod parts;
mod strongholds;

use crate::poi::PoiKind;
use crate::rng::{Rng, Seed};
use parts::{Affix, ENDINGS, STARTS, affixes};

/// How high a place stands, which makes some endings likelier: "-mouth" and
/// "-ford" low down, "-fell" and "-crag" high up. Any ending can still turn up
/// anywhere, as coasts move over time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Altitude {
    Low,
    Middle,
    High,
}

/// Below this many metres above the sea a place is low.
pub const LOW_BELOW: i32 = 150;
/// From this many metres above the sea a place is high.
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
/// The share of towns' names with an extra word, before or after.
const AFFIX_PERCENT: u64 = 50;

/// What a place is called, or `None` if a `kind` of poi has no name of its
/// own (only towns, castles and forts do). `seed` is the poi's map seed;
/// `metres` how high it stands above the sea.
pub fn place_name(kind: PoiKind, seed: Seed, metres: i32) -> Option<String> {
    let altitude = Altitude::of_metres(metres);
    if let Some(word) = strongholds::kind_word(kind) {
        let mut rng = seed.derive("stronghold name").rng();
        let (starts, endings) = strongholds::words();
        return Some(format!(
            "{word} {}",
            base_name(&mut rng, altitude, &starts, &endings)
        ));
    }
    let affixes = affixes(kind)?;
    let mut rng = seed.derive("town name").rng();
    let base = base_name(&mut rng, altitude, STARTS, ENDINGS);
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
/// times running, no word written twice ("Stonestone").
fn base_name(
    rng: &mut Rng,
    altitude: Altitude,
    starts: &[&str],
    endings: &[(&str, Altitude)],
) -> String {
    let weights: Vec<(usize, u64)> = endings
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
        let start = starts[rng.range_u64(0, starts.len() as u64) as usize];
        let ending = endings[rng.pick_weighted(&weights)].0;
        name = format!("{start}{ending}");
        if reads_well(start, ending) {
            break;
        }
    }
    name
}

/// Whether a beginning and an ending join without a doubled letter at the
/// join, a letter three times running or the same word twice.
fn reads_well(start: &str, ending: &str) -> bool {
    let last = start.chars().last().map(|c| c.to_ascii_lowercase());
    if last == ending.chars().next() || start.eq_ignore_ascii_case(ending) {
        return false;
    }
    let letters: Vec<char> = format!("{start}{ending}").to_lowercase().chars().collect();
    !letters
        .windows(3)
        .any(|three| three[0] == three[1] && three[1] == three[2])
}

#[cfg(test)]
mod stronghold_tests;
#[cfg(test)]
mod tests;
