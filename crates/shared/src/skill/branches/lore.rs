//! Lore: what is known, and finding out more. A first draft, for the owner to change.

use crate::sheet::Attribute::{Intellect, Perception};

use super::super::draft::{Draft, branch, leaf};

pub(in super::super) const LORE: Draft = branch(
    "lore",
    "Lore",
    &[Intellect],
    &[
        leaf("researching", "Researching", &[]),
        leaf("creatures", "Creatures", &[]),
        leaf("history", "History", &[]),
        leaf("languages", "Languages", &[]),
        leaf("appraising", "Appraising", &[Intellect, Perception]),
    ],
);
