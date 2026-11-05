//! The body parts of each kind of creature.

use super::peoples::{BIRDFOLK, DWARF, ELF, GOBLIN, HALFLING, LIZARDFOLK, MINOTAUR, ON_TOES, ORC};
use super::{BodyPart, CreatureDef, CreatureKind, Layer, Location, Size};
use crate::sheet::{Ability, Skill, Stats, Trait};

pub(super) const SKIN: &[Layer] = &[Layer::Skin];
pub(super) const SKIN_OUTER: &[Layer] = &[Layer::Skin, Layer::Outer];
pub(super) const SKIN_OUTER_OVER: &[Layer] = &[Layer::Skin, Layer::Outer, Layer::Over];
pub(super) const MIDDLE_OUTER: &[Layer] = &[Layer::Middle, Layer::Outer];
pub(super) const MIDDLE_OUTER_OVER: &[Layer] = &[Layer::Middle, Layer::Outer, Layer::Over];
pub(super) const SKIN_MIDDLE: &[Layer] = &[Layer::Skin, Layer::Middle];
pub(super) const SKIN_TO_OUTER: &[Layer] = &[Layer::Skin, Layer::Middle, Layer::Outer];
pub(super) const ALL_LAYERS: &[Layer] = &Layer::ALL;

pub(super) const fn part(location: Location, layers: &'static [Layer]) -> BodyPart {
    BodyPart {
        location,
        layers,
        places: 1,
    }
}

/// Room for a pack and a cloak.
pub(super) const BACK: BodyPart = BodyPart {
    location: Location::Back,
    layers: &[Layer::Outer],
    places: 2,
};

pub(super) const fn fingers(places: u8) -> BodyPart {
    BodyPart {
        location: Location::Fingers,
        layers: SKIN_TO_OUTER,
        places,
    }
}

/// A two-legged body with hands of this many fingers in all. Surcoats and
/// tabards go Over from the shoulders to the legs.
pub(super) const fn biped(finger_count: u8) -> [BodyPart; 11] {
    [
        part(Location::Head, SKIN_TO_OUTER),
        part(Location::Neck, SKIN_TO_OUTER),
        part(Location::Shoulders, MIDDLE_OUTER_OVER),
        BACK,
        part(Location::Arms, ALL_LAYERS),
        part(Location::Torso, ALL_LAYERS),
        part(Location::Waist, MIDDLE_OUTER_OVER),
        part(Location::Hands, MIDDLE_OUTER),
        fingers(finger_count),
        part(Location::Legs, ALL_LAYERS),
        part(Location::Feet, SKIN_TO_OUTER),
    ]
}

pub(super) const TEN_FINGERS: [BodyPart; 11] = biped(10);

pub(super) const CHICKEN: [BodyPart; 6] = [
    part(Location::Head, SKIN),
    part(Location::Neck, SKIN),
    part(Location::Wings, SKIN_OUTER),
    part(Location::Torso, SKIN),
    part(Location::Legs, SKIN_MIDDLE),
    part(Location::Feet, SKIN),
];

/// A caparison goes Over barding.
pub(super) const HORSE: [BodyPart; 6] = [
    part(Location::Head, SKIN_OUTER_OVER),
    part(Location::Neck, SKIN_OUTER_OVER),
    part(Location::Torso, SKIN_OUTER_OVER),
    part(Location::Forelegs, SKIN_OUTER),
    part(Location::Hindlegs, SKIN_OUTER),
    part(Location::Tail, SKIN),
];

pub(super) const CENTAUR: [BodyPart; 10] = [
    part(Location::Head, SKIN_TO_OUTER),
    part(Location::Neck, SKIN_TO_OUTER),
    part(Location::Shoulders, MIDDLE_OUTER_OVER),
    BACK,
    part(Location::Arms, ALL_LAYERS),
    part(Location::Torso, ALL_LAYERS),
    part(Location::Hands, MIDDLE_OUTER),
    fingers(10),
    part(Location::Forelegs, SKIN_OUTER),
    part(Location::Hindlegs, SKIN_OUTER),
];

impl CreatureKind {
    pub const fn def(self) -> CreatureDef {
        use Ability::{Flutter, Kick, NightVision, Trample};
        use Skill::{Climbing, Foraging, Running, Stealth, Swimming};
        use Trait::{CoatColour, FeatherColour, HairColour, Height, SkinShade};
        const fn stats(strength: u8, dexterity: u8, intelligence: u8, constitution: u8) -> Stats {
            Stats {
                strength,
                dexterity,
                intelligence,
                constitution,
            }
        }
        match self {
            Self::Birdfolk => BIRDFOLK,
            Self::Dwarf => DWARF,
            Self::Elf => ELF,
            Self::Goblin => GOBLIN,
            Self::Halfling => HALFLING,
            Self::Lizardfolk => LIZARDFOLK,
            Self::Minotaur => MINOTAUR,
            Self::Orc => ORC,
            Self::Beggar => CreatureDef {
                name: "Beggar",
                weight: 55_000,
                size: Size::Medium,
                inventory: (2, 2),
                body: &TEN_FINGERS,
                speed: 80,
                uses_stairs: true,
                playable: false,
                stats: stats(6, 7, 9, 6),
                skills: &[(Foraging, 1)],
                abilities: &[],
                looks: &[HairColour, SkinShade, Height],
            },
            Self::Centaur => CreatureDef {
                name: "Centaur",
                weight: 500_000,
                size: Size::Large,
                inventory: (6, 5),
                body: &CENTAUR,
                speed: 140,
                uses_stairs: false,
                // Too big to stand in one cell, so not for players (owner, 2026-11-04).
                playable: false,
                stats: stats(14, 10, 9, 13),
                skills: &[(Running, 4), (Foraging, 1)],
                abilities: &[Kick, Trample],
                looks: &[HairColour, SkinShade, CoatColour, Height],
            },
            Self::Chicken => CreatureDef {
                name: "Chicken",
                weight: 2_500,
                size: Size::Tiny,
                inventory: (2, 2),
                body: &CHICKEN,
                speed: 70,
                uses_stairs: false,
                playable: false,
                stats: stats(1, 8, 1, 2),
                skills: &[(Foraging, 3)],
                abilities: &[Flutter],
                looks: &[FeatherColour],
            },
            Self::Giant => CreatureDef {
                name: "Giant",
                weight: 1_500_000,
                speed: 90,
                uses_stairs: true,
                size: Size::Giant,
                inventory: (8, 6),
                body: &TEN_FINGERS,
                // Too big to stand in one cell, so not for players (owner, 2026-11-04).
                playable: false,
                stats: stats(20, 6, 7, 18),
                skills: &[(Climbing, 1)],
                abilities: &[Trample],
                looks: &[HairColour, SkinShade, Height],
            },
            Self::Horse => CreatureDef {
                name: "Horse",
                weight: 500_000,
                size: Size::Large,
                inventory: (4, 3),
                body: &HORSE,
                speed: 150,
                uses_stairs: false,
                playable: false,
                stats: stats(16, 10, 3, 14),
                skills: &[(Running, 5), (Swimming, 1)],
                abilities: &[Kick, Trample],
                looks: &[CoatColour, Height],
            },
            Self::Human => CreatureDef {
                name: "Human",
                weight: 70_000,
                speed: 100,
                uses_stairs: true,
                size: Size::Medium,
                inventory: (6, 4),
                body: &TEN_FINGERS,
                playable: true,
                stats: stats(10, 10, 10, 10),
                skills: &[(Swimming, 1), (Climbing, 1)],
                abilities: &[],
                looks: &[HairColour, SkinShade, Height],
            },
            Self::Kobold => CreatureDef {
                name: "Kobold",
                weight: 25_000,
                size: Size::Small,
                inventory: (4, 3),
                body: &ON_TOES,
                speed: 100,
                uses_stairs: true,
                playable: true,
                stats: stats(7, 13, 9, 8),
                skills: &[(Stealth, 3), (Climbing, 2)],
                abilities: &[NightVision],
                looks: &[SkinShade, Height],
            },
        }
    }
}
