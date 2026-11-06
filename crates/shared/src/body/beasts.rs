//! Animals from the creature research (`docs/context/research-creatures.md`): beasts
//! on four legs and birds on two, each as a few measurements their layout is worked
//! out from (`beasts/build.rs`). Unlike a biped's frame, a beast's layout is at its own
//! usual size, in millimetres: it fills its picture whatever its size.

use super::Layout;
use super::parts::{Feet, Horns, NONE, Parts, Snout, Wings};
use crate::creature::CreatureKind;

mod build;

/// A beast standing on four legs, level-backed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fourfoot {
    /// The top of the back over the shoulders, from the ground.
    pub withers: i32,
    /// Along the back from the hip joints to the shoulder joints.
    pub body: i32,
    /// The chest's half width and half depth.
    pub barrel: (i32, i32),
    /// The upper legs' thickness, as a radius.
    pub leg: i32,
    /// The neck's length, its lean forwards from upright in degrees, and its radius.
    pub neck: (i32, i32, i32),
    /// The head's length from chin to crown, and its half width and depth.
    pub head: (i32, (i32, i32)),
}

/// A bird on two legs, its back leaning forwards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Bird {
    /// How high its hips are.
    pub hips: i32,
    /// Along the back from the hips to the neck, and how far the back leans forwards
    /// from upright, in degrees.
    pub body: (i32, i32),
    pub barrel: (i32, i32),
    pub leg: i32,
    pub neck: (i32, i32, i32),
    pub head: (i32, (i32, i32)),
}

/// How a beast's body is put together.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Plan {
    Fourfoot(Fourfoot),
    Bird(Bird),
}

/// An animal, by name, with its body's measurements and what it has beyond a plain
/// trunk and head.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Beast {
    pub name: &'static str,
    pub plan: Plan,
    pub parts: Parts,
}

/// The first animals drawn, measured from the research's heights and lengths (the
/// middle of each range). Wolf and cattle are drawn before they are kinds.
pub const BEASTS: [Beast; 4] = [
    Beast {
        name: "Horse",
        plan: Plan::Fourfoot(Fourfoot {
            withers: 1580,
            body: 1100,
            barrel: (270, 340),
            leg: 125,
            neck: (800, 50, 125),
            head: (380, (140, 180)),
        }),
        parts: Parts {
            snout: Snout::Muzzle,
            snout_length: 420,
            ears: 90,
            tail: 900,
            feet: Feet::Hooves,
            ..NONE
        },
    },
    Beast {
        name: "Wolf",
        plan: Plan::Fourfoot(Fourfoot {
            withers: 740,
            body: 720,
            barrel: (110, 160),
            leg: 55,
            neck: (260, 60, 65),
            head: (170, (75, 85)),
        }),
        parts: Parts {
            snout: Snout::Long,
            snout_length: 170,
            ears: 70,
            tail: 450,
            feet: Feet::Digitigrade,
            ..NONE
        },
    },
    Beast {
        name: "Cattle",
        plan: Plan::Fourfoot(Fourfoot {
            withers: 1350,
            body: 1250,
            barrel: (320, 380),
            leg: 130,
            neck: (480, 70, 160),
            head: (360, (150, 170)),
        }),
        parts: Parts {
            snout: Snout::Muzzle,
            snout_length: 330,
            ears: 60,
            horns: Horns::Bull,
            horn_length: 350,
            tail: 800,
            feet: Feet::Hooves,
            ..NONE
        },
    },
    Beast {
        name: "Chicken",
        plan: Plan::Bird(Bird {
            hips: 170,
            body: (240, 65),
            barrel: (75, 90),
            leg: 14,
            neck: (70, 20, 28),
            head: (55, (27, 35)),
        }),
        parts: Parts {
            snout: Snout::Beak,
            snout_length: 30,
            feet: Feet::Talons,
            wings: Wings::Folded,
            ..NONE
        },
    },
];

impl Beast {
    /// The animal of this kind, by its name.
    pub fn of(kind: CreatureKind) -> Option<Beast> {
        BEASTS
            .into_iter()
            .find(|beast| beast.name == kind.def().name)
    }

    /// Its body's layout, at its own usual size.
    pub fn layout(&self) -> Layout {
        match self.plan {
            Plan::Fourfoot(four) => build::fourfoot(&four, self.parts),
            Plan::Bird(bird) => build::bird(&bird, self.parts),
        }
    }
}
