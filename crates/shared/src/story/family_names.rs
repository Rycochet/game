//! Family names: a beginning and an ending from lists suiting each people.
//! Peoples with no lists of their own take two of their given names' sounds, as
//! a clan name.

use crate::creature::{CreatureKind, three_alike};
use crate::rng::{Rng, Seed};

struct Parts {
    starts: &'static [&'static str],
    ends: &'static [&'static str],
}

const HUMAN: Parts = Parts {
    starts: &[
        "Ash", "Brook", "Hol", "Mar", "Thorn", "Wel", "Hart", "Fen", "Cal", "Dun", "Pen", "Stan",
        "Wat", "Rad", "Hay", "Lang",
    ],
    ends: &[
        "well", "by", "ton", "ley", "wood", "ford", "more", "wick", "croft", "den", "ham", "lock",
    ],
};
const DWARF: Parts = Parts {
    starts: &[
        "Iron", "Stone", "Deep", "Gold", "Bronze", "Flint", "Copper", "Coal", "Grey", "Oak",
        "Anvil", "Rock",
    ],
    ends: &[
        "fist", "beard", "foot", "delve", "helm", "hand", "forge", "brow", "mantle", "shield",
    ],
};
const ELF: Parts = Parts {
    starts: &[
        "Ar", "Gala", "Ilm", "Lora", "Sil", "Tha", "Eli", "Cae", "Fae", "Mith",
    ],
    ends: &[
        "landir", "wen", "thil", "noril", "dhel", "rion", "sar", "nethil", "iel",
    ],
};
const HALFLING: Parts = Parts {
    starts: &[
        "Bram", "Good", "Tuck", "Under", "Burr", "Green", "Hill", "Thistle", "Sweet", "Apple",
        "Honey", "Mug",
    ],
    ends: &[
        "foot", "bottle", "hill", "wick", "barrel", "burrow", "field", "bank", "dew", "button",
    ],
};

/// A family name for a household of `kind`, from a seed of its own.
pub(super) fn family_name(kind: CreatureKind, seed: Seed) -> String {
    let mut rng = seed.rng();
    let parts = match kind {
        CreatureKind::Human | CreatureKind::Beggar | CreatureKind::Giant => &HUMAN,
        CreatureKind::Dwarf => &DWARF,
        CreatureKind::Elf => &ELF,
        CreatureKind::Halfling => &HALFLING,
        _ => return kind.random_name(&mut rng),
    };
    // A few tries, for one with no letter three times running.
    let mut name = String::new();
    for _ in 0..8 {
        name = [pick(&mut rng, parts.starts), pick(&mut rng, parts.ends)].concat();
        if !three_alike(&name) {
            break;
        }
    }
    name
}

pub(super) fn pick(rng: &mut Rng, list: &[&'static str]) -> &'static str {
    list[rng.range_u64(0, list.len() as u64) as usize]
}
