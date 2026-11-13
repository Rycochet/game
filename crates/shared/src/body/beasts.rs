//! Animals from the creature research (`docs/context/research-creatures.md`): beasts
//! on four legs and birds on two, each as a few measurements their layout is worked
//! out from (`beasts/build.rs`). Unlike a biped's frame, a beast's layout is at its own
//! usual size, in millimetres: it fills its picture whatever its size.

use super::Layout;
use super::measured::{COW, DOG, HORSE, Measured, MeasuredRing};
use super::measured_heads::{COW_HEAD, HORSE_HEAD, MeasuredHead, WOLF_HEAD};
use super::parts::{Ears, Feet, Horns, NONE, Parts, Snout, Tail, Wings};
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
    /// The forelegs' and hind legs' thickness as radii, where each hangs from and at
    /// each joint down to the last above the ground, measured with the trunk.
    pub fore: [i32; 4],
    pub hind: [i32; 4],
    /// How long a foot is along the ground.
    pub foot: i32,
    /// The neck's length, its lean forwards from upright in degrees, and its radius
    /// where it meets the head (it widens to the trunk's last ring).
    pub neck: (i32, i32, i32),
    /// The head: its outline (`HeadShape`) scaled to this long from the back of the
    /// skull to the nose, leaning this many degrees nose down.
    pub head: (i32, i32, &'static MeasuredHead),
    /// How much thicker the coat makes the trunk all round than the measured shape.
    pub coat: i32,
    /// The trunk's shape, as measured, scaled to this size.
    pub measured: &'static Measured,
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
    /// As a beast's: its outline's length, its lean and its outline.
    pub head: (i32, i32, &'static MeasuredHead),
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
            fore: [100, 79, 50, 45],
            hind: [145, 125, 70, 50],
            foot: 100,
            neck: (650, 45, 130),
            head: (860, 45, &HORSE_HEAD),
            coat: 0,
            measured: &HORSE,
        }),
        parts: Parts {
            snout: Snout::Muzzle,
            snout_length: 420,
            ears: 120,
            ear_shape: Ears::Upright,
            tail: 900,
            feet: Feet::Hooves,
            mane: 140,
            tail_hair: 700,
            ..NONE
        },
    },
    Beast {
        name: "Wolf",
        plan: Plan::Fourfoot(Fourfoot {
            withers: 740,
            body: 720,
            barrel: (110, 160),
            fore: [55, 42, 28, 30],
            hind: [70, 51, 38, 36],
            foot: 110,
            neck: (230, 65, 90),
            head: (290, 8, &WOLF_HEAD),
            coat: 25,
            measured: &DOG,
        }),
        parts: Parts {
            snout: Snout::Long,
            snout_length: 170,
            ears: 90,
            ear_shape: Ears::Upright,
            tail: 450,
            tail_shape: Tail::Bushy,
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
            fore: [105, 78, 46, 42],
            hind: [130, 103, 58, 51],
            foot: 104,
            neck: (420, 75, 170),
            head: (520, 40, &COW_HEAD),
            coat: 0,
            measured: &COW,
        }),
        parts: Parts {
            snout: Snout::Muzzle,
            snout_length: 330,
            ears: 130,
            ear_shape: Ears::Sideways,
            horns: Horns::Bull,
            horn_length: 350,
            tail: 800,
            tail_shape: Tail::Thin,
            feet: Feet::Hooves,
            tail_hair: 220,
            dewlap: 130,
            udder: 160,
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
            head: (80, 0, &CHICKEN_HEAD),
        }),
        parts: Parts {
            snout: Snout::Beak,
            snout_length: 30,
            comb: 28,
            feet: Feet::Talons,
            wings: Wings::Folded,
            ..NONE
        },
    },
];

const fn ring(along: i32, half: i32, belly: i32, back: i32) -> MeasuredRing {
    MeasuredRing {
        along,
        half,
        belly,
        back,
    }
}

/// A hen's head, which no mesh has: drawn by hand in the measured heads' terms (in
/// thousandths of its length, belly and back up from its line): a round skull high on
/// the neck, and a short beak pointing a little down.
pub const CHICKEN_HEAD: MeasuredHead = MeasuredHead {
    source: "drawn by hand",
    pitch: 0,
    rings: [
        ring(-250, 80, 200, 450),
        ring(-100, 330, 60, 650),
        ring(50, 380, 20, 700),
        ring(200, 360, 40, 650),
        ring(350, 260, 120, 520),
        ring(500, 120, 200, 380),
        ring(650, 80, 220, 320),
        ring(780, 20, 230, 265),
    ],
};

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
