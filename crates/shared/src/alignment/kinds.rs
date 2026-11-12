//! Which kinds have an alignment, which way each leans, and a mob's rolled
//! from its own hash about its kind's leaning. Numbers are *to balance*.

use super::Alignment;
use crate::creature::CreatureKind;

/// How far either side of its kind's leaning a mob's alignment may be rolled.
pub const ROLLED_SPREAD: i32 = 250;

impl CreatureKind {
    /// Where this kind's alignment leans, or `None` for an animal, which acts
    /// on instinct rather than by choice (owner, 2026-11-12): hostile kinds
    /// lean evil, halflings and beggars good, other peoples neither.
    pub const fn alignment_leaning(self) -> Option<i16> {
        use CreatureKind::*;
        match self {
            Cattle | Chicken | Horse | Wolf => None,
            Drider | Goblin | Kobold | Minotaur | Orc => Some(-400),
            Beggar | Halfling => Some(150),
            Birdfolk | Centaur | Dwarf | Elf | Giant | Human | Lizardfolk | ScorpionMan => Some(0),
        }
    }
}

impl Alignment {
    /// A mob's, rolled from a hash of its own within `ROLLED_SPREAD` of its
    /// kind's leaning; `None` for an animal.
    pub fn rolled(kind: CreatureKind, hash: u64) -> Option<Self> {
        let leaning = i32::from(kind.alignment_leaning()?);
        let span = (2 * ROLLED_SPREAD + 1) as u64;
        let nudge = (hash % span) as i32 - ROLLED_SPREAD;
        Some(Self::new(leaning + nudge))
    }
}
