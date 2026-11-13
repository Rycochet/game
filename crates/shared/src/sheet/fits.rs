//! Which looks suit a creature: its kind's traits and choices, and its gender's
//! rule, that a female creature has no beard (owner, 2026-11-13). Checked by
//! the server for a new character, kept to by mobs' rolls and by old saves as
//! they load, and followed by the creation screen.

use crate::creature::CreatureKind;
use crate::gender::Gender;

use super::{Appearance, Trait};

impl Trait {
    /// Whether a creature of this kind and gender has a choice for this trait:
    /// one of its kind's looks, and not a beard on a female.
    pub fn applies(self, kind: CreatureKind, gender: Gender) -> bool {
        kind.def().looks.contains(&self) && !(self == Trait::Beard && gender == Gender::Female)
    }
}

impl Appearance {
    /// Whether every choice is for a trait this kind and gender have, and is one
    /// of its choices.
    pub fn suits(&self, kind: CreatureKind, gender: Gender) -> bool {
        self.0.iter().all(|(&which, &choice)| {
            which.applies(kind, gender) && (choice as usize) < which.choices_for(kind).len()
        })
    }

    /// Drops what this gender can't have (a female's beard): for a gender
    /// newly chosen or rolled, and for saves from before the rule.
    pub fn fit(&mut self, gender: Gender) {
        if gender == Gender::Female {
            self.0.remove(&Trait::Beard);
        }
    }

    /// This look, as `fit` leaves it.
    pub fn fitted(mut self, gender: Gender) -> Self {
        self.fit(gender);
        self
    }
}
