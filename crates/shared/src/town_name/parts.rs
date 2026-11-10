//! The words towns' names are made of. Changing any list changes the names of
//! towns already visited: the golden test fails on purpose.

use super::Altitude::{self, High, Low, Middle};
use crate::poi::PoiKind;

/// Beginnings, written with a capital.
pub(super) const STARTS: &[&str] = &[
    "Ash", "Thorn", "Mill", "Oak", "Wey", "Stan", "Ald", "Bram", "Elm", "Hol", "Ran", "Sel",
    "Wens", "Har", "Lang", "Mar", "Nor", "Sut", "Wes", "Bex", "Cal", "Dun", "Eden", "Fern", "Gold",
    "Hart", "Hazel", "Kirk", "Lin", "Mor", "Ox", "Pen", "Red", "Rook", "Sand", "Stone", "Swan",
    "Thurl", "Wal", "Whit", "Wil", "Wyn", "Ayle", "Brad", "Chip", "Cran", "Ely", "Bur",
];

/// Endings, in lower case, with the height they suit best.
pub(super) const ENDINGS: &[(&str, Altitude)] = &[
    ("ford", Low),
    ("mouth", Low),
    ("wick", Low),
    ("marsh", Low),
    ("well", Low),
    ("mere", Low),
    ("brook", Low),
    ("bridge", Low),
    ("haven", Low),
    ("fleet", Low),
    ("lea", Low),
    ("ton", Middle),
    ("bury", Middle),
    ("ham", Middle),
    ("by", Middle),
    ("stead", Middle),
    ("field", Middle),
    ("worth", Middle),
    ("thorpe", Middle),
    ("cote", Middle),
    ("ley", Middle),
    ("stow", Middle),
    ("fell", High),
    ("crag", High),
    ("moor", High),
    ("top", High),
    ("law", High),
    ("dale", High),
    ("combe", High),
    ("ridge", High),
    ("hope", High),
    ("scar", High),
    ("edge", High),
];

/// An extra word, written before or after the name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Affix {
    Before(&'static str),
    After(&'static str),
}

use Affix::{After, Before};

const HAMLET: &[Affix] = &[
    Before("Little"),
    Before("Nether"),
    Before("Hamlet of"),
    After("Hamlet"),
    After("End"),
    After("Green"),
];
const VILLAGE: &[Affix] = &[
    Before("Little"),
    Before("Old"),
    Before("Village of"),
    After("Village"),
    After("Hamlet"),
    After("Cross"),
    After("Green"),
];
const TOWN: &[Affix] = &[
    Before("Great"),
    Before("Market"),
    Before("Old"),
    After("Town"),
    After("Market"),
    After("Cross"),
];
const CITY: &[Affix] = &[
    Before("Great"),
    Before("King's"),
    Before("Old"),
    After("City"),
    After("Town"),
];

/// The extra words a size of town may have, or `None` if `kind` isn't a town.
/// A word may suit more than one size: a village can be called "Hamlet".
pub(super) fn affixes(kind: PoiKind) -> Option<&'static [Affix]> {
    match kind {
        PoiKind::Hamlet => Some(HAMLET),
        PoiKind::Village => Some(VILLAGE),
        PoiKind::Town => Some(TOWN),
        PoiKind::City => Some(CITY),
        _ => None,
    }
}
