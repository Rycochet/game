//! Stealth: going unseen and getting in. A first draft, for the owner to change.

use crate::sheet::Attribute::{Agility, Intellect, Perception};

use super::super::draft::{Draft, branch, leaf};

pub(in super::super) const STEALTH: Draft = branch(
    "stealth",
    "Stealth",
    &[Agility],
    &[
        leaf("sneaking", "Sneaking", &[]),
        leaf("hiding", "Hiding", &[Agility, Perception]),
        leaf("locks", "Locks", &[Agility, Intellect]),
        branch(
            "traps",
            "Traps",
            &[Agility, Intellect],
            &[
                leaf("setting", "Setting", &[]),
                leaf("disarming", "Disarming", &[]),
            ],
        ),
    ],
);
