//! Survival: living off the land, and you against a creature. Observation is what
//! notices traps and the like, rather than a bare Perception check. A first draft,
//! for the owner to change.

use crate::sheet::Attribute::{Agility, Perception, Toughness};

use super::super::draft::{Draft, branch, leaf};

pub(in super::super) const SURVIVAL: Draft = branch(
    "survival",
    "Survival",
    &[Perception, Toughness],
    &[
        leaf("observation", "Observation", &[Perception]),
        leaf("tracking", "Tracking", &[Perception]),
        branch(
            "hunting",
            "Hunting",
            &[Perception, Agility],
            &[
                leaf("trapping", "Trapping", &[]),
                leaf("butchery", "Butchery", &[]),
            ],
        ),
        leaf("fishing", "Fishing", &[Perception]),
    ],
);
