//! The first peoples from the creature research (`docs/context/research-creatures.md`):
//! each kind's body as proportions beside a person's, what it has beyond a person's,
//! and how tall it usually stands.

use super::Proportions;
use super::parts::{Feet, Horns, NONE, Parts, Snout, Wings};
use crate::creature::CreatureKind;

/// A body that stands on two legs, by name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Biped {
    pub name: &'static str,
    /// How tall one usually stands, in millimetres: its frame is a person's height, so
    /// this is only for showing how big it is.
    pub height: i32,
    pub proportions: Proportions,
    pub parts: Parts,
}

/// One row: its usual height (the middle of the research's range), heads tall in tenths
/// (a person is 80), then breadth, limbs, arms and legs in thousandths of a person's,
/// then what it has beyond a person.
const fn biped(
    name: &'static str,
    height: i32,
    [heads, breadth, limbs, arms, legs]: [i32; 5],
    parts: Parts,
) -> Biped {
    Biped {
        name,
        height,
        proportions: Proportions {
            heads,
            breadth,
            limbs,
            arms,
            legs,
        },
        parts,
    }
}

/// From the research's biped table: its heads are of a 7.5-head person, so here they
/// are scaled to the frame's 8, and its legs (hip height over height, a person's 0.51)
/// are made thousandths of a person's.
pub const FIRST_BIPEDS: [Biped; 9] = [
    biped(
        "Halfling",
        1050,
        [53, 900, 900, 950, 922],
        Parts { ears: 15, ..NONE },
    ),
    biped("Dwarf", 1330, [53, 1250, 1300, 950, 863], NONE),
    biped(
        "Elf",
        1700,
        [85, 900, 800, 1030, 1059],
        Parts { ears: 60, ..NONE },
    ),
    biped(
        "Orc",
        1820,
        [75, 1150, 1200, 1080, 961],
        Parts {
            ears: 20,
            tusks: 30,
            ..NONE
        },
    ),
    biped(
        "Goblin",
        1050,
        [53, 850, 800, 1100, 902],
        Parts { ears: 90, ..NONE },
    ),
    biped(
        "Kobold",
        950,
        [43, 850, 800, 950, 941],
        Parts {
            snout: Snout::Long,
            snout_length: 140,
            horns: Horns::Back,
            horn_length: 70,
            tail: 800,
            feet: Feet::Digitigrade,
            ..NONE
        },
    ),
    biped(
        "Lizardfolk",
        1900,
        [75, 1000, 950, 1000, 980],
        Parts {
            snout: Snout::Long,
            snout_length: 160,
            tail: 1000,
            feet: Feet::Digitigrade,
            ..NONE
        },
    ),
    biped(
        "Minotaur",
        2450,
        [75, 1350, 1350, 1050, 980],
        Parts {
            snout: Snout::Muzzle,
            snout_length: 110,
            ears: 40,
            horns: Horns::Bull,
            horn_length: 300,
            tail: 600,
            feet: Feet::Hooves,
            ..NONE
        },
    ),
    biped(
        "Birdfolk",
        1550,
        [75, 950, 800, 1000, 980],
        Parts {
            snout: Snout::Beak,
            snout_length: 90,
            tail: 250,
            feet: Feet::Talons,
            wings: Wings::Arms,
            ..NONE
        },
    ),
];

impl Biped {
    /// The body of one of these kinds, by its name.
    pub fn of(kind: CreatureKind) -> Option<Biped> {
        FIRST_BIPEDS
            .into_iter()
            .find(|biped| biped.name == kind.def().name)
    }
}

impl Parts {
    /// What a kind's body has beyond a person's: nothing for a kind without a biped.
    pub fn of(kind: CreatureKind) -> Parts {
        Biped::of(kind).map_or(NONE, |biped| biped.parts)
    }
}
