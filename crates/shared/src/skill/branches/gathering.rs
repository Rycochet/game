//! Gathering: taking what the land gives. A first draft, for the owner to change.

use crate::sheet::Attribute::{Intellect, Perception, Strength, Toughness};

use super::super::draft::{Draft, branch, leaf};

pub(in super::super) const GATHERING: Draft = branch(
    "gathering",
    "Gathering",
    &[Perception, Toughness],
    &[
        leaf("foraging", "Foraging", &[Perception]),
        branch(
            "farming",
            "Farming",
            &[Toughness],
            &[
                leaf("crops", "Crops", &[]),
                leaf("orchards", "Orchards", &[]),
                leaf("livestock", "Livestock", &[]),
            ],
        ),
        branch(
            "mining",
            "Mining",
            &[Strength, Toughness],
            &[
                leaf("stone", "Stone", &[]),
                leaf("ore", "Ore", &[]),
                leaf("gems", "Gems", &[]),
            ],
        ),
        leaf("woodcutting", "Woodcutting", &[Strength, Toughness]),
        leaf("prospecting", "Prospecting", &[Perception, Intellect]),
        leaf("herbalism", "Herbalism", &[Intellect, Perception]),
    ],
);
