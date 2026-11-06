//! Crafting: making things, building included. A first draft, for the owner to change.

use crate::sheet::Attribute::{Agility, Intellect, Perception, Strength};

use super::super::draft::{Draft, branch, leaf};

pub(in super::super) const CRAFTING: Draft = branch(
    "crafting",
    "Crafting",
    &[Agility, Intellect],
    &[
        leaf("building", "Building", &[Strength, Intellect]),
        branch(
            "smithing",
            "Smithing",
            &[Strength, Agility],
            &[
                leaf("weapons", "Weaponsmithing", &[]),
                leaf("armour", "Armoursmithing", &[]),
                leaf("tools", "Blacksmithing", &[]),
            ],
        ),
        leaf("leatherwork", "Leatherwork", &[]),
        leaf("tailoring", "Tailoring", &[Agility]),
        leaf("woodwork", "Woodwork", &[]),
        leaf("cooking", "Cooking", &[Perception]),
        leaf("alchemy", "Alchemy", &[Intellect]),
    ],
);
