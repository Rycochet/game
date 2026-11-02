//! Kinds of creature: the same for players and, later, mobs. What each kind's
//! body is like, and so what it can wear. See `docs/context/shared/creature.md`.
//!
//! A table here, as with kinds of item. Different bodies use different locations
//! (`Forelegs` isn't `Legs`), so trousers don't fit a horse, and a centaur is a
//! human top on those.

use serde::{Deserialize, Serialize};

use crate::sheet::{Ability, Feeling, Reaction, Skill, Stats, Trait};

/// Clothing layers, from the skin out. A layer covers those under it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Layer {
    Skin,
    Middle,
    Outer,
}

impl Layer {
    pub const ALL: [Self; 3] = [Self::Skin, Self::Middle, Self::Outer];
}

/// Somewhere on a body that clothes can go.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Location {
    Head,
    Neck,
    Shoulders,
    Arms,
    Wings,
    Torso,
    Waist,
    Hands,
    Fingers,
    Legs,
    Feet,
    Forelegs,
    Hindlegs,
    Tail,
}

/// How big a creature is, or the clothes made for one, smallest first.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Size {
    Tiny,
    Small,
    #[default]
    Medium,
    Large,
    Huge,
    Giant,
}

impl Size {
    pub const ALL: [Self; 6] = [
        Self::Tiny,
        Self::Small,
        Self::Medium,
        Self::Large,
        Self::Huge,
        Self::Giant,
    ];

    /// What it is called.
    pub fn name(self) -> &'static str {
        match self {
            Self::Tiny => "Tiny",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::Huge => "Huge",
            Self::Giant => "Giant",
        }
    }

    /// How big it is beside `Medium`, in quarters, to keep to whole numbers.
    pub fn quarters(self) -> u32 {
        match self {
            Self::Tiny => 2,
            Self::Small => 3,
            Self::Medium => 4,
            Self::Large => 5,
            Self::Huge => 6,
            Self::Giant => 8,
        }
    }

    /// `by` sizes up or, negative, down, stopping at the ends.
    pub fn stepped(self, by: i8) -> Self {
        let last = Self::ALL.len() as i8 - 1;
        Self::ALL[(self as i8 + by).clamp(0, last) as usize]
    }

    /// How many sizes apart.
    pub fn apart(self, other: Self) -> u8 {
        (self as i8 - other as i8).unsigned_abs()
    }
}

/// One location on a body: which layers go there, and how many places it has for
/// things that take only one (fingers for rings).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyPart {
    pub location: Location,
    pub layers: &'static [Layer],
    pub places: u8,
}

/// What a kind of creature is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreatureDef {
    pub name: &'static str,
    /// What its body weighs, in grams, with nothing on it.
    pub weight: u32,
    pub size: Size,
    /// Its base grid: what it can carry with no bag.
    pub pockets: (u8, u8),
    pub body: &'static [BodyPart],
    /// How fast it goes for its kind, as a percentage of ordinary.
    pub speed: u32,
    /// Whether it can go up and down stairs, so as to follow someone.
    pub uses_stairs: bool,
    /// Whether someone can choose to play one.
    pub playable: bool,
    pub stats: Stats,
    /// Skills it starts with, and how good at them.
    pub skills: &'static [(Skill, u8)],
    /// What it can do because of what it is.
    pub abilities: &'static [Ability],
    /// What can differ between creatures of this kind, to look at.
    pub looks: &'static [Trait],
}

impl CreatureDef {
    pub fn part(&self, location: Location) -> Option<&'static BodyPart> {
        self.body.iter().find(|part| part.location == location)
    }

    /// How many fingers it has: none if it has no `Fingers`.
    pub fn fingers(&self) -> Option<u8> {
        self.part(Location::Fingers).map(|part| part.places)
    }
}

/// A kind of creature. Saved by name, so new ones can go anywhere in the list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CreatureKind {
    Beggar,
    Centaur,
    Chicken,
    Giant,
    Horse,
    #[default]
    Human,
    Kobold,
}

const SKIN: &[Layer] = &[Layer::Skin];
const SKIN_OUTER: &[Layer] = &[Layer::Skin, Layer::Outer];
const MIDDLE: &[Layer] = &[Layer::Middle];
const MIDDLE_OUTER: &[Layer] = &[Layer::Middle, Layer::Outer];
const SKIN_MIDDLE: &[Layer] = &[Layer::Skin, Layer::Middle];
const ALL_LAYERS: &[Layer] = &Layer::ALL;

const fn part(location: Location, layers: &'static [Layer]) -> BodyPart {
    BodyPart {
        location,
        layers,
        places: 1,
    }
}

const fn fingers(places: u8) -> BodyPart {
    BodyPart {
        location: Location::Fingers,
        layers: ALL_LAYERS,
        places,
    }
}

/// A two-legged body with hands of this many fingers in all.
const fn biped(finger_count: u8) -> [BodyPart; 10] {
    [
        part(Location::Head, SKIN_OUTER),
        part(Location::Neck, SKIN_OUTER),
        part(Location::Shoulders, MIDDLE_OUTER),
        part(Location::Arms, ALL_LAYERS),
        part(Location::Torso, ALL_LAYERS),
        part(Location::Waist, MIDDLE),
        part(Location::Hands, MIDDLE_OUTER),
        fingers(finger_count),
        part(Location::Legs, ALL_LAYERS),
        part(Location::Feet, ALL_LAYERS),
    ]
}

const TEN_FINGERS: [BodyPart; 10] = biped(10);
const EIGHT_FINGERS: [BodyPart; 10] = biped(8);

const CHICKEN: [BodyPart; 6] = [
    part(Location::Head, SKIN),
    part(Location::Neck, SKIN),
    part(Location::Wings, SKIN_OUTER),
    part(Location::Torso, SKIN),
    part(Location::Legs, SKIN_MIDDLE),
    part(Location::Feet, SKIN),
];

const HORSE: [BodyPart; 6] = [
    part(Location::Head, SKIN_OUTER),
    part(Location::Neck, SKIN_OUTER),
    part(Location::Torso, SKIN_OUTER),
    part(Location::Forelegs, SKIN_OUTER),
    part(Location::Hindlegs, SKIN_OUTER),
    part(Location::Tail, SKIN),
];

const CENTAUR: [BodyPart; 9] = [
    part(Location::Head, SKIN_OUTER),
    part(Location::Neck, SKIN_OUTER),
    part(Location::Shoulders, MIDDLE_OUTER),
    part(Location::Arms, ALL_LAYERS),
    part(Location::Torso, ALL_LAYERS),
    part(Location::Hands, MIDDLE_OUTER),
    fingers(10),
    part(Location::Forelegs, SKIN_OUTER),
    part(Location::Hindlegs, SKIN_OUTER),
];

impl CreatureKind {
    /// Every kind, alphabetically.
    pub const ALL: [Self; 7] = [
        Self::Beggar,
        Self::Centaur,
        Self::Chicken,
        Self::Giant,
        Self::Horse,
        Self::Human,
        Self::Kobold,
    ];

    /// How this kind feels about another, to begin with. A kind likes its own.
    pub fn feeling_about(self, other: Self) -> Feeling {
        use CreatureKind::{Beggar, Giant, Kobold};
        match (self, other) {
            // Has nothing against anyone, and hopes.
            (Beggar, _) => Feeling::Neutral,
            (one, two) if one == two => Feeling::Friendly,
            (Giant, Kobold) | (Kobold, _) => Feeling::Unfriendly,
            _ => Feeling::Neutral,
        }
    }

    /// What this kind does about something it feels this way about.
    pub fn reaction(self, feeling: Feeling) -> Reaction {
        use CreatureKind::{Chicken, Horse};
        match (self, feeling) {
            (_, Feeling::None) => Reaction::Ignore,
            (Chicken | Horse, Feeling::Friendly) => Reaction::Ignore,
            (_, Feeling::Friendly) => Reaction::Approach,
            (Chicken, _) => Reaction::Flee,
            (Horse, Feeling::Unfriendly) => Reaction::Flee,
            (_, Feeling::Unfriendly) => Reaction::Attack,
            (_, Feeling::Neutral) => Reaction::Ignore,
        }
    }

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
            Self::Beggar => CreatureDef {
                name: "Beggar",
                weight: 55_000,
                size: Size::Medium,
                pockets: (2, 2),
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
                pockets: (6, 5),
                body: &CENTAUR,
                speed: 140,
                uses_stairs: false,
                playable: true,
                stats: stats(14, 10, 9, 13),
                skills: &[(Running, 4), (Foraging, 1)],
                abilities: &[Kick, Trample],
                looks: &[HairColour, SkinShade, CoatColour, Height],
            },
            Self::Chicken => CreatureDef {
                name: "Chicken",
                weight: 2_500,
                size: Size::Tiny,
                pockets: (2, 2),
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
                pockets: (8, 6),
                body: &TEN_FINGERS,
                playable: true,
                stats: stats(20, 6, 7, 18),
                skills: &[(Climbing, 1)],
                abilities: &[Trample],
                looks: &[HairColour, SkinShade, Height],
            },
            Self::Horse => CreatureDef {
                name: "Horse",
                weight: 500_000,
                size: Size::Large,
                pockets: (4, 3),
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
                pockets: (6, 4),
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
                pockets: (4, 3),
                body: &EIGHT_FINGERS,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_is_listed_alphabetically_and_no_location_twice() {
        let names: Vec<_> = CreatureKind::ALL.iter().map(|k| k.def().name).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
        for kind in CreatureKind::ALL {
            let body = kind.def().body;
            for (index, part) in body.iter().enumerate() {
                assert!(!part.layers.is_empty(), "{:?}", part.location);
                assert!(part.places > 0);
                assert!(
                    body[index + 1..]
                        .iter()
                        .all(|o| o.location != part.location)
                );
            }
        }
    }

    #[test]
    fn every_kind_weighs_something() {
        for kind in CreatureKind::ALL {
            assert!(kind.def().weight > 0, "{:?}", kind);
        }
    }

    #[test]
    fn sizes_step_and_stop_at_the_ends() {
        assert_eq!(Size::Medium.stepped(-1), Size::Small);
        assert_eq!(Size::Tiny.stepped(-3), Size::Tiny);
        assert_eq!(Size::Huge.stepped(5), Size::Giant);
        assert_eq!(Size::Tiny.apart(Size::Large), 3);
    }

    #[test]
    fn kinds_like_their_own_and_react_as_they_are() {
        use CreatureKind::{Chicken, Human, Kobold};
        assert_eq!(Kobold.feeling_about(Kobold), Feeling::Friendly);
        assert_eq!(Kobold.feeling_about(Human), Feeling::Unfriendly);
        assert_eq!(Chicken.feeling_about(Human), Feeling::Neutral);
        assert_eq!(Kobold.reaction(Feeling::Unfriendly), Reaction::Attack);
        assert_eq!(Kobold.reaction(Feeling::Friendly), Reaction::Approach);
        assert_eq!(Chicken.reaction(Feeling::Neutral), Reaction::Flee);
        assert_eq!(Human.reaction(Feeling::Neutral), Reaction::Ignore);
    }

    #[test]
    fn finger_counts_differ_by_kind() {
        assert_eq!(CreatureKind::Human.def().fingers(), Some(10));
        assert_eq!(CreatureKind::Kobold.def().fingers(), Some(8));
        assert_eq!(CreatureKind::Chicken.def().fingers(), None);
    }
}
