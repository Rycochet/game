//! The choices for each trait a creature can differ by, for its kind: names, and the
//! colour each is drawn in where it is a colour. From the creature research
//! (`docs/context/research-creatures.md`). Choices are saved by place in the list, so
//! new ones go at the end.

use crate::creature::CreatureKind;

use super::Trait;

/// One choice for a trait: what it is called and, for a colour, its colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    pub name: &'static str,
    /// Red, green and blue, 0 to 255.
    pub colour: Option<[u8; 3]>,
}

const fn colour(name: &'static str, rgb: [u8; 3]) -> Choice {
    Choice {
        name,
        colour: Some(rgb),
    }
}

const fn word(name: &'static str) -> Choice {
    Choice { name, colour: None }
}

const HEIGHT: &[Choice] = &[
    word("Average"),
    word("Very short"),
    word("Short"),
    word("Tall"),
    word("Very tall"),
];

const HAIR: &[Choice] = &[
    colour("Black", [30, 26, 24]),
    colour("Brown", [92, 62, 40]),
    colour("Blonde", [214, 184, 120]),
    colour("Red", [150, 62, 34]),
    colour("Grey", [150, 148, 144]),
    colour("White", [232, 230, 224]),
];

/// People's skin: the first is a plain middle shade, as the bare body was drawn.
const SKIN: &[Choice] = &[
    colour("Medium", [199, 184, 163]),
    colour("Pale", [236, 214, 196]),
    colour("Light", [224, 190, 160]),
    colour("Tan", [190, 142, 104]),
    colour("Brown", [140, 96, 64]),
    colour("Dark", [86, 58, 40]),
];

const DWARF_SKIN: &[Choice] = &[
    colour("Ruddy", [206, 150, 124]),
    colour("Tan", [190, 142, 104]),
    colour("Brown", [140, 96, 64]),
    colour("Grey-brown", [140, 120, 104]),
];

const ELF_SKIN: &[Choice] = &[
    colour("Fair", [232, 210, 190]),
    colour("Pale", [240, 228, 218]),
    colour("Golden", [214, 180, 128]),
    colour("Tan", [190, 142, 104]),
    colour("Bark brown", [120, 88, 62]),
];

const ORC_SKIN: &[Choice] = &[
    colour("Grey-green", [120, 140, 104]),
    colour("Olive", [128, 128, 72]),
    colour("Grey", [128, 128, 124]),
    colour("Dark brown", [92, 66, 48]),
];

const GOBLIN_SKIN: &[Choice] = &[
    colour("Green", [112, 150, 72]),
    colour("Yellow-green", [160, 170, 80]),
    colour("Grey", [136, 136, 128]),
    colour("Ochre", [180, 140, 64]),
];

const KOBOLD_SKIN: &[Choice] = &[
    colour("Red-brown", [150, 82, 56]),
    colour("Rust", [176, 96, 48]),
    colour("Black", [52, 46, 44]),
    colour("Ochre", [180, 140, 64]),
    colour("Grey", [128, 124, 120]),
];

const LIZARD_SKIN: &[Choice] = &[
    colour("Green", [88, 132, 72]),
    colour("Grey-green", [112, 132, 108]),
    colour("Brown", [120, 92, 60]),
    colour("Olive", [120, 120, 64]),
];

const HORSE_COAT: &[Choice] = &[
    colour("Bay", [120, 66, 36]),
    colour("Black", [36, 32, 30]),
    colour("Chestnut", [150, 80, 40]),
    colour("Grey", [160, 158, 152]),
    colour("White", [234, 232, 226]),
];

const BULL_COAT: &[Choice] = &[
    colour("Brown", [104, 70, 46]),
    colour("Black", [40, 34, 32]),
    colour("Red-brown", [130, 64, 40]),
    colour("Grey", [128, 124, 118]),
];

const HEN_FEATHERS: &[Choice] = &[
    colour("Brown", [128, 84, 52]),
    colour("Black", [36, 34, 34]),
    word("Speckled"),
    colour("White", [240, 238, 232]),
];

const BIRD_FEATHERS: &[Choice] = &[
    colour("White", [242, 242, 238]),
    colour("Brown", [128, 84, 52]),
    colour("Grey", [150, 150, 152]),
    colour("Black", [40, 40, 44]),
    colour("Red", [176, 52, 40]),
    colour("Blue", [64, 96, 168]),
];

impl Trait {
    /// The choices a creature of this kind has for this trait, in order.
    pub fn choices_for(self, kind: CreatureKind) -> &'static [Choice] {
        use CreatureKind::{
            Birdfolk, Dwarf, Elf, Goblin, Horse, Kobold, Lizardfolk, Minotaur, Orc,
        };
        match self {
            Self::Height => HEIGHT,
            Self::HairColour => HAIR,
            Self::SkinShade => match kind {
                Dwarf => DWARF_SKIN,
                Elf => ELF_SKIN,
                Orc => ORC_SKIN,
                Goblin => GOBLIN_SKIN,
                Kobold => KOBOLD_SKIN,
                Lizardfolk => LIZARD_SKIN,
                _ => SKIN,
            },
            Self::CoatColour => match kind {
                Horse => HORSE_COAT,
                Minotaur => BULL_COAT,
                _ => HORSE_COAT,
            },
            Self::FeatherColour => match kind {
                Birdfolk => BIRD_FEATHERS,
                _ => HEN_FEATHERS,
            },
        }
    }
}
