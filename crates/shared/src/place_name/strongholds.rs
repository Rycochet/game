//! Castles' and forts' names: their kind always in front ("Castle", "Fort"),
//! nothing after, and a base from towns' words plus sterner ones, so there are
//! more of them. Changing a list renames every castle and fort (golden test).

use super::Altitude::{self, High, Low, Middle};
use super::parts::{ENDINGS, STARTS};
use crate::poi::PoiKind;

/// Beginnings only castles and forts use, as well as towns'.
const MORE_STARTS: &[&str] = &[
    "Raven", "Iron", "Black", "Grey", "Wolf", "Storm", "Dragon", "Eagle", "Hawk", "Frost", "High",
    "Shield", "Crow", "Drake", "Lion", "Stag", "Falcon", "Ard", "Bel", "Mal", "Vor", "Gal",
];

/// Endings only castles and forts use, as well as towns', with the height
/// they suit best.
const MORE_ENDINGS: &[(&str, Altitude)] = &[
    ("gate", Low),
    ("march", Low),
    ("water", Low),
    ("hold", Middle),
    ("keep", Middle),
    ("guard", Middle),
    ("burgh", Middle),
    ("wall", Middle),
    ("stone", Middle),
    ("watch", High),
    ("mont", High),
    ("spire", High),
    ("rock", High),
    ("helm", High),
];

/// The word always written before a castle's or fort's name, or `None` for
/// any other kind.
pub(super) fn kind_word(kind: PoiKind) -> Option<&'static str> {
    match kind {
        PoiKind::Castle => Some("Castle"),
        PoiKind::Fort => Some("Fort"),
        _ => None,
    }
}

/// Every beginning and ending a castle or fort may have.
pub(super) fn words() -> (Vec<&'static str>, Vec<(&'static str, Altitude)>) {
    let starts = STARTS.iter().chain(MORE_STARTS).copied().collect();
    let endings = ENDINGS.iter().chain(MORE_ENDINGS).copied().collect();
    (starts, endings)
}
