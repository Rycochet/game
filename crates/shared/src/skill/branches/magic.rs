//! Magic: a level shallower than weapons, as casting happens less often than attacks.
//! Arcane leans on Intellect, divine on Empathy. A first draft, for the owner to change.

use crate::sheet::Attribute::{Aura, Empathy, Intellect, Will};

use super::super::draft::{Draft, branch, leaf};

pub(in super::super) const MAGIC: Draft = branch(
    "magic",
    "Magic",
    &[Intellect, Will],
    &[
        branch(
            "arcane",
            "Arcane",
            &[Intellect],
            &[
                leaf("fire", "Fire", &[]),
                leaf("water", "Water", &[]),
                leaf("air", "Air", &[]),
                leaf("earth", "Earth", &[]),
                leaf("illusion", "Illusion", &[Intellect, Will]),
            ],
        ),
        branch(
            "divine",
            "Divine",
            &[Empathy],
            &[
                leaf("healing", "Healing", &[]),
                leaf("blessing", "Blessing", &[]),
                leaf("warding", "Warding", &[Empathy, Aura]),
            ],
        ),
        branch(
            "wild",
            "Wild",
            &[Aura, Empathy],
            &[
                leaf("beasts", "Beasts", &[]),
                leaf("plants", "Plants", &[]),
                leaf("weather", "Weather", &[]),
            ],
        ),
    ],
);
