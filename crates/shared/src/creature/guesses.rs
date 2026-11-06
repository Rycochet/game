//! First guesses at kinds' attributes, to try the rules out until they are
//! balanced: the four the old stats had, the other five left at a person's.

use crate::sheet::Attributes;

/// A person's median for every attribute: what the peoples all start from, as
/// their differences are their bonuses.
pub(super) const PERSON: Attributes = Attributes::all(100);

/// Medians for a kind that isn't a people, by the four the old stats had.
pub(super) const fn medians(
    strength: u16,
    agility: u16,
    toughness: u16,
    intellect: u16,
) -> Attributes {
    Attributes {
        strength,
        agility,
        toughness,
        intellect,
        ..PERSON
    }
}

/// A kind's bonus: placeholders, on these four only, for now (owner, 2026-11-06:
/// real ones later).
pub(super) const fn bonus(
    strength: u16,
    agility: u16,
    toughness: u16,
    intellect: u16,
) -> Attributes {
    Attributes {
        strength,
        agility,
        toughness,
        intellect,
        ..Attributes::all(0)
    }
}

/// No bonus at all.
pub(super) const NONE: Attributes = Attributes::all(0);
