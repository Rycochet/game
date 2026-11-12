//! What stories are told about: the people in them, their pets, named bosses
//! and the places they went, as names and as facts. Names are sounds and are
//! never translated; everything else is a value (`lang::Selector`) each
//! language words for itself. All of it comes from seeds, so the server can
//! make it again whenever it needs to. Plots and beats come later
//! (Flavour text PR 3). See `docs/context/shared/story.md` and
//! `docs/context/story-text.md`.

mod boss;
mod direction;
mod distance;
mod epithet;
mod family_names;
mod household;
mod pet;
mod place;

pub use boss::Boss;
pub use direction::Direction;
pub use distance::{DAY_WITHIN, DistanceBand, NEAR_WITHIN, SHORT_WITHIN};
pub use epithet::Epithet;
pub use household::{Household, Person};
pub use pet::{Pet, PetKind};
pub use place::{Condition, PlaceDescription, PlaceKind};

use crate::creature::CreatureKind;

/// Whether this kind is a people, with families and family names, rather than
/// an animal. Every kind is listed, so a new one has to be decided on.
fn is_people(kind: CreatureKind) -> bool {
    use CreatureKind::*;
    match kind {
        Cattle | Chicken | Horse | Wolf => false,
        Beggar | Birdfolk | Centaur | Drider | Dwarf | Elf | Giant | Goblin | Halfling | Human
        | Kobold | Lizardfolk | Minotaur | Orc | ScorpionMan => true,
    }
}

/// The east–west and north–south way from one world map position to another,
/// the near way round a planet `width` points wide. North is −y.
fn offset(from: (i64, i64), to: (i64, i64), width: i64) -> (i64, i64) {
    let mut dx = (to.0 - from.0).rem_euclid(width.max(1));
    if dx > width / 2 {
        dx -= width;
    }
    (dx, to.1 - from.1)
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod where_tests;
#[cfg(test)]
mod words_tests;
