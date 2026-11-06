//! Taurs on eight legs from the creature research (`docs/context/research-creatures.md`):
//! the drider (from the SRD, credited in `docs/credits.md`) and the scorpion man. A
//! person's upper body on a spider's or a scorpion's; their bodies' looks are
//! `shared::body::TAURS`. Not playable, as they don't stand in one cell. Attributes,
//! skills and weights are first guesses.

use super::bodies::{
    ALL_LAYERS, BACK, MIDDLE_OUTER, MIDDLE_OUTER_OVER, SKIN_TO_OUTER, fingers, part,
};
use super::guesses::medians;
use super::{BodyPart, CreatureDef, Location, Size};
use crate::sheet::{Ability, Trait};

/// A person's upper body: nothing worn below the waist, as there is no person there.
const UPPER_BODY: [BodyPart; 8] = [
    part(Location::Head, SKIN_TO_OUTER),
    part(Location::Neck, SKIN_TO_OUTER),
    part(Location::Shoulders, MIDDLE_OUTER_OVER),
    BACK,
    part(Location::Arms, ALL_LAYERS),
    part(Location::Torso, ALL_LAYERS),
    part(Location::Hands, MIDDLE_OUTER),
    fingers(10),
];

const LOOKS: &[Trait] = &[
    Trait::HairColour,
    Trait::HairStyle,
    Trait::SkinShade,
    Trait::CoatColour,
];
/// A scorpion man may have a beard; a drider, a dark elf, has none.
const BEARDED: &[Trait] = &[
    Trait::HairColour,
    Trait::HairStyle,
    Trait::Beard,
    Trait::SkinShade,
    Trait::CoatColour,
];

/// A dark elf on a giant spider's body, cursed, hunting in the deep places.
pub(super) const DRIDER: CreatureDef = CreatureDef {
    name: "Drider",
    weight: 400_000,
    size: Size::Large,
    inventory: (6, 4),
    body: &UPPER_BODY,
    speed: 110,
    uses_stairs: true,
    playable: false,
    attributes: medians(130, 120, 120, 100),
    bonus_set: &[],
    skills: &[("athletics/climbing", 2500), ("stealth/sneaking", 900)],
    abilities: &[Ability::NightVision],
    looks: LOOKS,
};

/// A guardian with a man's upper body on a scorpion's.
pub(super) const SCORPION_MAN: CreatureDef = CreatureDef {
    name: "Scorpion Man",
    weight: 320_000,
    size: Size::Large,
    inventory: (6, 4),
    body: &UPPER_BODY,
    speed: 100,
    uses_stairs: true,
    playable: false,
    attributes: medians(150, 90, 150, 90),
    bonus_set: &[],
    skills: &[("athletics/climbing", 100)],
    abilities: &[Ability::NightVision],
    looks: BEARDED,
};
