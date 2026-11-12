//! Pets: an animal of a household, its kind a value and its name a sound.

use serde::{Deserialize, Serialize};

use super::family_names::pick;
use crate::creature::{CreatureKind, three_alike};
use crate::gender::Gender;
use crate::lang::Selector;
use crate::rng::{Rng, Seed};

/// What kind of animal a pet is. Only kinds the game has creatures for (owner:
/// a wolf stands in for a dog); more as creature kinds are added.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PetKind {
    Dog,
    Horse,
    Chicken,
    Cattle,
    // New kinds go at the end: messages carry one by its place.
}

/// A household's animal, as a story names it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pet {
    pub kind: PetKind,
    pub gender: Gender,
    pub name: String,
}

const STARTS: &[&str] = &[
    "Pip", "Bo", "Dot", "Bram", "Tuf", "Mo", "Bis", "Pat", "Nell", "Rus", "Bel", "Sco", "Jas",
    "Bon", "Fen", "Mag",
];
const ENDS: &[&str] = &["", "s", "sy", "ty", "kin", "ble", "per", "pie", "ie"];

impl PetKind {
    pub const ALL: [Self; 4] = [Self::Dog, Self::Horse, Self::Chicken, Self::Cattle];

    /// The creature it is drawn and played as.
    pub const fn creature(self) -> CreatureKind {
        match self {
            Self::Dog => CreatureKind::Wolf,
            Self::Horse => CreatureKind::Horse,
            Self::Chicken => CreatureKind::Chicken,
            Self::Cattle => CreatureKind::Cattle,
        }
    }
}

impl Selector for PetKind {
    fn fact(&self) -> &'static str {
        "pet"
    }

    fn key(&self) -> &'static str {
        match self {
            Self::Dog => "dog",
            Self::Horse => "horse",
            Self::Chicken => "chicken",
            Self::Cattle => "cattle",
        }
    }
}

impl Pet {
    /// A pet of any kind, from a seed of its own.
    pub fn new(seed: Seed) -> Self {
        let kinds = PetKind::ALL;
        let kind = kinds[seed
            .derive("pet kind")
            .rng()
            .range_u64(0, kinds.len() as u64) as usize];
        Self::of_kind(kind, seed)
    }

    /// A pet of this kind, from a seed of its own.
    pub fn of_kind(kind: PetKind, seed: Seed) -> Self {
        Self {
            kind,
            gender: Gender::rolled(seed.derive("gender").value()),
            name: pet_name(&mut seed.derive("pet name").rng()),
        }
    }
}

/// A name for an animal: a short, homely sound.
pub(super) fn pet_name(rng: &mut Rng) -> String {
    let mut name = String::new();
    for _ in 0..8 {
        name = [pick(rng, STARTS), pick(rng, ENDS)].concat();
        if !three_alike(&name) {
            break;
        }
    }
    name
}
