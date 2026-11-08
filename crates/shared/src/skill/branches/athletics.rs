//! Athletics: moving the body well. A first draft, for the owner to change.

use crate::sheet::Attribute::{Agility, Perception, Strength, Toughness};

use super::super::draft::{Draft, branch, leaf};

pub(in super::super) const ATHLETICS: Draft = branch(
    "athletics",
    "Athletics",
    &[Strength, Agility, Toughness],
    &[
        leaf("climbing", "Climbing", &[Strength, Agility]),
        leaf("swimming", "Swimming", &[Strength, Toughness]),
        leaf("running", "Running", &[Toughness, Agility]),
        leaf("jumping", "Jumping", &[Strength, Agility]),
        leaf("dodging", "Dodging", &[Agility, Perception]),
        leaf("weighttraining", "Weight Training", &[Strength, Toughness]),
    ],
);
