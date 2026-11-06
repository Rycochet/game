//! Names for new characters: a beginning, perhaps a middle and an ending, from
//! sounds that suit each people. Only a suggestion: the player may type any name.

use super::CreatureKind;
use crate::rng::Rng;

/// The sounds one people's names are made of.
struct Sounds {
    starts: &'static [&'static str],
    middles: &'static [&'static str],
    ends: &'static [&'static str],
}

const HUMAN: Sounds = Sounds {
    starts: &[
        "Al", "Bren", "Cor", "Ed", "Gar", "Hal", "Jor", "Mar", "Os", "Ro", "Tam", "Wil", "El",
        "Mir", "An", "Be",
    ],
    middles: &["a", "e", "i", ""],
    ends: &[
        "an", "ric", "wyn", "ton", "ard", "na", "lla", "en", "dric", "ly",
    ],
};
const DWARF: Sounds = Sounds {
    starts: &[
        "Bor", "Dur", "Thra", "Grim", "Kaz", "Bal", "Dvo", "Fun", "Gim", "Thor", "Brom", "Hil",
    ],
    middles: &["in", "a", "o", "u", ""],
    ends: &[
        "din", "grim", "li", "rek", "dur", "bar", "nar", "ki", "da", "gun",
    ],
};
const ELF: Sounds = Sounds {
    starts: &[
        "Ae", "Ela", "Fae", "Gal", "Ith", "Lor", "Mel", "Syl", "Tha", "Ari", "Cel", "Ny",
    ],
    middles: &["la", "ri", "the", "ne", "li", ""],
    ends: &[
        "dor", "wen", "iel", "ion", "thas", "ra", "nor", "wyn", "las",
    ],
};
const ORC: Sounds = Sounds {
    starts: &[
        "Gro", "Ug", "Mog", "Thrak", "Gor", "Ur", "Naz", "Skar", "Bol", "Dag",
    ],
    middles: &["a", "u", ""],
    ends: &["ash", "nak", "gul", "dush", "rok", "ug", "mash", "grim"],
};
const GOBLIN: Sounds = Sounds {
    starts: &[
        "Snik", "Grib", "Nix", "Zog", "Skab", "Rik", "Wiz", "Gnat", "Fizz", "Krin",
    ],
    middles: &["", "e", "i"],
    ends: &["it", "le", "zit", "nob", "sy", "gle", "ik", "wort"],
};
const HALFLING: Sounds = Sounds {
    starts: &[
        "Bil", "Pip", "Mer", "Ro", "Sam", "Fro", "Lob", "Tob", "Dai", "Mil", "Rosi", "Hob",
    ],
    middles: &["", "a", "o"],
    ends: &["bo", "pin", "wise", "ley", "do", "lo", "sy", "kin", "by"],
};
const KOBOLD: Sounds = Sounds {
    starts: &["Kip", "Yap", "Snek", "Rit", "Tik", "Zik", "Mek", "Vrik"],
    middles: &["", "a", "i"],
    ends: &["ka", "rik", "yip", "tak", "ix", "ek"],
};
const LIZARDFOLK: Sounds = Sounds {
    starts: &["Ss", "Tsa", "Kesh", "Zhar", "Is", "Rhes", "Sil", "Vash"],
    middles: &["", "i", "a"],
    ends: &["ka", "irr", "thes", "ash", "eth", "ix", "ssa"],
};
const MINOTAUR: Sounds = Sounds {
    starts: &["Bru", "Tor", "Gor", "Mor", "Ast", "Bel", "Kar", "Ster"],
    middles: &["", "e", "o"],
    ends: &["gan", "ros", "dax", "ion", "mos", "tar", "eus"],
};
const BIRDFOLK: Sounds = Sounds {
    starts: &["Kee", "Tir", "Wren", "Ky", "Ree", "Pip", "Shri", "Ka"],
    middles: &["", "a", "i"],
    ends: &["ree", "wit", "ka", "eel", "ah", "ki", "rr"],
};

impl CreatureKind {
    /// A name that sounds like one of this kind's own. The same `rng` gives the
    /// same name.
    pub fn random_name(self, rng: &mut Rng) -> String {
        let sounds = match self {
            Self::Dwarf => &DWARF,
            Self::Elf => &ELF,
            Self::Orc => &ORC,
            Self::Goblin => &GOBLIN,
            Self::Halfling => &HALFLING,
            Self::Kobold => &KOBOLD,
            Self::Lizardfolk => &LIZARDFOLK,
            Self::Minotaur => &MINOTAUR,
            Self::Birdfolk => &BIRDFOLK,
            _ => &HUMAN,
        };
        let mut pick = |list: &[&'static str]| list[rng.range_u64(0, list.len() as u64) as usize];
        // A few tries, for one with no letter three times running.
        let mut name = String::new();
        for _ in 0..8 {
            name = [pick(sounds.starts), pick(sounds.middles), pick(sounds.ends)].concat();
            if !three_alike(&name) {
                break;
            }
        }
        name
    }
}

/// Whether a letter comes three times running ("Sss").
fn three_alike(name: &str) -> bool {
    let letters: Vec<char> = name.chars().map(|c| c.to_ascii_lowercase()).collect();
    letters
        .windows(3)
        .any(|three| three[0] == three[1] && three[1] == three[2])
}

#[cfg(test)]
mod tests;
