//! The body parts of each kind of creature.

use super::animals::{CATTLE, WOLF};
use super::guesses::{PERSON, medians};
use super::peoples::{BIRDFOLK, DWARF, ELF, GOBLIN, HALFLING, LIZARDFOLK, MINOTAUR, ON_TOES, ORC};
use super::taurs::{DRIDER, SCORPION_MAN};
use super::{BodyPart, CreatureDef, CreatureKind, Layer, Location, Size};
use crate::sheet::{Ability, Trait};

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

/// Two hands to hold weapons in (`Location::Held`).
pub(super) const HELD: BodyPart = BodyPart {
    location: Location::Held,
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
pub(super) const fn biped(finger_count: u8) -> [BodyPart; 12] {
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
        HELD,
    ]
}

pub(super) const TEN_FINGERS: [BodyPart; 12] = biped(10);

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

pub(super) const CENTAUR: [BodyPart; 11] = [
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
    HELD,
];

impl CreatureKind {
    pub const fn def(self) -> CreatureDef {
        use crate::sheet::Attribute::Agility;
        use Ability::{Flutter, Kick, NightVision, Trample};
        use Trait::{Beard, CoatColour, FeatherColour, HairColour, HairStyle, Height, SkinShade};
        match self {
            Self::Birdfolk => BIRDFOLK,
            Self::Cattle => CATTLE,
            Self::Drider => DRIDER,
            Self::ScorpionMan => SCORPION_MAN,
            Self::Wolf => WOLF,
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
                attributes: medians(60, 70, 60, 90),
                bonus_set: &[],
                skills: &[("gathering/foraging", 1_000)],
                abilities: &[],
                looks: &[HairColour, HairStyle, Beard, SkinShade, Height],
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
                attributes: medians(140, 100, 130, 90),
                bonus_set: &[],
                skills: &[("athletics/running", 64_000), ("gathering/foraging", 1_000)],
                abilities: &[Kick, Trample],
                looks: &[HairColour, HairStyle, Beard, SkinShade, CoatColour, Height],
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
                attributes: medians(10, 80, 20, 10),
                bonus_set: &[],
                skills: &[("gathering/foraging", 27_000)],
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
                attributes: medians(200, 60, 180, 70),
                bonus_set: &[],
                skills: &[("athletics/climbing", 1_000)],
                abilities: &[Trample],
                looks: &[HairColour, HairStyle, Beard, SkinShade, Height],
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
                attributes: medians(160, 100, 140, 30),
                bonus_set: &[],
                skills: &[
                    ("athletics/running", 125_000),
                    ("athletics/swimming", 1_000),
                ],
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
                attributes: PERSON,
                bonus_set: &[],
                skills: &[("athletics/swimming", 1_000), ("athletics/climbing", 1_000)],
                abilities: &[],
                looks: &[HairColour, HairStyle, Beard, SkinShade, Height],
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
                // Playable, so a person's medians, with its bonus.
                attributes: PERSON,
                bonus_set: &[(Agility, 15)],
                skills: &[("stealth/sneaking", 27_000), ("athletics/climbing", 8_000)],
                abilities: &[NightVision],
                looks: &[SkinShade, Height],
            },
        }
    }
}
