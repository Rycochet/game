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
