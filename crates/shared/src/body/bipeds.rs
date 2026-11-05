//! The first set of two-legged bodies from the creature research
//! (`docs/context/research-creatures.md`), as proportions beside a person's. They aren't
//! kinds of creature yet: for now they are only drawn, for the owner to look over.

use super::Proportions;

/// A body that stands on two legs, by name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Biped {
    pub name: &'static str,
    pub proportions: Proportions,
}

/// One row: heads tall in tenths (a person is 80), then breadth, limbs, arms and legs in
/// thousandths of a person's.
const fn biped(
    name: &'static str,
    heads: i32,
    breadth: i32,
    limbs: i32,
    arms: i32,
    legs: i32,
) -> Biped {
    Biped {
        name,
        proportions: Proportions {
            heads,
            breadth,
            limbs,
            arms,
            legs,
        },
    }
}

/// From the research's biped table: its heads are of a 7.5-head person, so here they
/// are scaled to the frame's 8, and its legs (hip height over height, a person's 0.51)
/// are made thousandths of a person's.
pub const FIRST_BIPEDS: [Biped; 9] = [
    biped("Halfling", 53, 900, 900, 950, 922),
    biped("Dwarf", 53, 1250, 1300, 950, 863),
    biped("Elf", 85, 900, 800, 1030, 1059),
    biped("Orc", 75, 1150, 1200, 1080, 961),
    biped("Goblin", 53, 850, 800, 1100, 902),
    biped("Kobold", 43, 850, 800, 950, 941),
    biped("Lizardfolk", 75, 1000, 950, 1000, 980),
    biped("Minotaur", 75, 1350, 1350, 1050, 980),
    biped("Birdfolk", 75, 950, 800, 1000, 980),
];
