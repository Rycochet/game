//! Social: dealing with people and creatures. A first draft, for the owner to change.

use crate::sheet::Attribute::{Empathy, Presence};

use super::super::draft::{Draft, branch, leaf};

pub(in super::super) const SOCIAL: Draft = branch(
    "social",
    "Social",
    &[Presence, Empathy],
    &[
        leaf("persuading", "Persuading", &[Presence, Empathy]),
        leaf("intimidating", "Intimidating", &[Presence]),
        leaf("trading", "Trading", &[]),
        leaf("animals", "Animals", &[Empathy]),
        leaf("leading", "Leading", &[Presence]),
    ],
);
