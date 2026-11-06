//! The first peoples from the creature research (`docs/context/research-creatures.md`):
//! two-legged kinds that can be played. Their bodies' looks are in `shared::body`; here
//! is what they can wear and what they are like. All share a person's medians, and
//! differ by their bonuses; bonuses, skills and weights are first guesses from the
//! research, to try the rules out.

use super::bodies::{SKIN, TEN_FINGERS, biped};
use super::guesses::PERSON;
use super::{BodyPart, CreatureDef, Layer, Location, Size};
use crate::sheet::{Ability, Attribute, Skill, Trait};

use Attribute::{Agility, Intellect, Strength, Toughness};

use Ability::{Flutter, NightVision, Trample};
use Skill::{Climbing, Foraging, Running, Stealth, Swimming};
use Trait::{Beard, CoatColour, FeatherColour, HairColour, HairStyle, Height, SkinShade};

/// A body with these layers at one of its locations instead.
const fn with(
    mut body: [BodyPart; 11],
    location: Location,
    layers: &'static [Layer],
) -> [BodyPart; 11] {
    let mut at = 0;
    while at < body.len() {
        if body[at].location as u8 == location as u8 {
            body[at].layers = layers;
        }
        at += 1;
    }
    body
}

/// On its toes, hooves or talons: a person's boots and sabatons don't go on its feet.
const fn not_plantigrade(body: [BodyPart; 11]) -> [BodyPart; 11] {
    with(body, Location::Feet, SKIN)
}

/// On its toes, with eight fingers: kobolds and lizardfolk.
pub(super) const ON_TOES: [BodyPart; 11] = not_plantigrade(biped(8));
const HOOVES: [BodyPart; 11] = not_plantigrade(TEN_FINGERS);
/// Its arms are wings and it stands on talons: nothing but rings and the like on its
/// arms, hands or feet.
const WINGED: [BodyPart; 11] = not_plantigrade(with(
    with(biped(6), Location::Arms, SKIN),
    Location::Hands,
    SKIN,
));

/// A people with plantigrade feet and ten fingers, playable, going at an ordinary pace.
const fn people(
    name: &'static str,
    weight: u32,
    size: Size,
    inventory: (u8, u8),
    bonus_set: &'static [(Attribute, u16)],
) -> CreatureDef {
    CreatureDef {
        name,
        weight,
        size,
        inventory,
        body: &TEN_FINGERS,
        speed: 100,
        uses_stairs: true,
        playable: true,
        attributes: PERSON,
        bonus_set,
        skills: &[],
        abilities: &[],
        looks: &[HairColour, HairStyle, SkinShade, Height],
    }
}

pub(super) const HALFLING: CreatureDef = CreatureDef {
    skills: &[(Stealth, 3), (Foraging, 1)],
    ..people("Halfling", 28_000, Size::Small, (5, 3), &[(Agility, 15)])
};

pub(super) const DWARF: CreatureDef = CreatureDef {
    looks: &[HairColour, HairStyle, Beard, SkinShade, Height],
    skills: &[(Climbing, 1)],
    abilities: &[NightVision],
    ..people(
        "Dwarf",
        75_000,
        Size::Small,
        (6, 4),
        &[(Strength, 10), (Toughness, 20)],
    )
};

pub(super) const ELF: CreatureDef = CreatureDef {
    skills: &[(Stealth, 2), (Foraging, 2)],
    ..people(
        "Elf",
        58_000,
        Size::Medium,
        (6, 4),
        &[(Agility, 15), (Intellect, 10)],
    )
};

pub(super) const ORC: CreatureDef = CreatureDef {
    looks: &[HairColour, HairStyle, Beard, SkinShade, Height],
    skills: &[(Running, 1)],
    abilities: &[NightVision],
    ..people(
        "Orc",
        100_000,
        Size::Medium,
        (6, 4),
        &[(Strength, 15), (Toughness, 10)],
    )
};

pub(super) const GOBLIN: CreatureDef = CreatureDef {
    skills: &[(Stealth, 2), (Climbing, 2)],
    abilities: &[NightVision],
    looks: &[SkinShade, Height],
    ..people("Goblin", 24_000, Size::Small, (4, 3), &[(Agility, 15)])
};

pub(super) const LIZARDFOLK: CreatureDef = CreatureDef {
    body: &ON_TOES,
    skills: &[(Swimming, 3)],
    looks: &[SkinShade, Height],
    ..people(
        "Lizardfolk",
        110_000,
        Size::Medium,
        (6, 4),
        &[(Strength, 10), (Toughness, 10)],
    )
};

pub(super) const MINOTAUR: CreatureDef = CreatureDef {
    body: &HOOVES,
    skills: &[(Running, 2)],
    abilities: &[Trample],
    looks: &[CoatColour, Height],
    ..people(
        "Minotaur",
        380_000,
        Size::Large,
        (7, 5),
        &[(Strength, 30), (Toughness, 20)],
    )
};

pub(super) const BIRDFOLK: CreatureDef = CreatureDef {
    body: &WINGED,
    skills: &[(Climbing, 1)],
    abilities: &[Flutter],
    looks: &[FeatherColour, Height],
    ..people("Birdfolk", 35_000, Size::Medium, (5, 3), &[(Agility, 20)])
};
