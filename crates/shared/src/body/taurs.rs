//! Taurs: a person's upper body standing on another body, where its neck would be: a
//! centaur's on a horse, a drider's on a spider, a scorpion man's on a scorpion. A beast
//! below is one of `BEASTS`, at its own size, or eight legs (`taurs/arthropod.rs`); the
//! upper body is a frame from proportions at a person's, as a biped's is, so everything
//! worn on a person's upper body is built on it and moved into place.

use super::layout::{Layout, Role, Upper};
use super::{BEASTS, Frame, NONE, PERSON, Parts, Plan, Proportions};
use crate::creature::CreatureKind;

mod arthropod;

pub use arthropod::Arthropod;

/// What a taur stands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lower {
    /// One of `BEASTS`, by name.
    Beast(&'static str),
    /// Eight legs.
    Arthropod(Arthropod),
}

/// A taur, by name: the body below, the person above, and its parts: the head's, and
/// for eight legs the abdomen and tail (a beast's tail and feet are its own).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Taur {
    pub name: &'static str,
    pub lower: Lower,
    pub upper: Proportions,
    pub parts: Parts,
}

/// A slender person, as an elf (the research's dark elf on its spider).
const SLENDER: Proportions = Proportions {
    heads: 85,
    breadth: 900,
    limbs: 800,
    arms: 1030,
    legs: 1059,
};

pub const TAURS: [Taur; 3] = [
    Taur {
        name: "Centaur",
        lower: Lower::Beast("Horse"),
        upper: PERSON,
        parts: NONE,
    },
    Taur {
        name: "Drider",
        lower: Lower::Arthropod(Arthropod {
            height: 1150,
            front: 800,
            barrel: (320, 260),
            reach: 1300,
            leg: 55,
        }),
        upper: SLENDER,
        parts: Parts {
            ears: 60,
            abdomen: 1200,
            ..NONE
        },
    },
    Taur {
        name: "Scorpion Man",
        lower: Lower::Arthropod(Arthropod {
            height: 1100,
            front: 700,
            barrel: (300, 220),
            reach: 1100,
            leg: 50,
        }),
        upper: PERSON,
        parts: Parts {
            abdomen: 1000,
            tail: 1300,
            sting: true,
            ..NONE
        },
    },
];

impl Taur {
    /// The taur of this kind, by its name.
    pub fn of(kind: CreatureKind) -> Option<Taur> {
        TAURS.into_iter().find(|taur| taur.name == kind.def().name)
    }

    /// The lower body's layout with the upper body standing where its neck would be:
    /// the person's hips sunk into the top of its front, over its forelegs.
    pub fn layout(&self) -> Layout {
        let (mut layout, deep) = match self.lower {
            Lower::Beast(name) => {
                let beast = BEASTS
                    .into_iter()
                    .find(|beast| beast.name == name)
                    .expect("a taur stands on one of the beasts");
                let mut layout = beast.layout();
                let Plan::Fourfoot(four) = beast.plan else {
                    return layout;
                };
                layout.parts = Parts {
                    tail: beast.parts.tail,
                    tail_hair: beast.parts.tail_hair,
                    feet: beast.parts.feet,
                    ..self.parts
                };
                (layout, four.barrel.1)
            }
            Lower::Arthropod(arthropod) => (arthropod.layout(self.parts), arthropod.barrel.1),
        };
        // The person's hips sink into the top of the lower body, a third of its depth
        // down from its top and a third of it in front of its shoulders.
        let [x, y, z] = layout.spine.at(layout.spine.shoulders);
        layout.upper = Some(Upper {
            frame: self.upper.frame(),
            at: [x, y + deep * 2 / 3, z + deep / 3],
        });
        layout
    }
}

impl Layout {
    /// The upper body as a layout of its own, in place: its trunk from the hips up,
    /// arms, neck and head, with no legs, tail or abdomen.
    pub fn upper_body(&self) -> Option<Layout> {
        let upper = self.upper?;
        let parts = Parts {
            tail: 0,
            tail_hair: 0,
            sting: false,
            abdomen: 0,
            ..self.parts
        };
        let mut body = Layout::biped(&upper.frame, parts);
        body.spine.base = upper.at;
        for slot in &mut body.limbs {
            if slot.is_some_and(|limb| limb.role == Role::Leg) {
                *slot = None;
            }
        }
        Some(body)
    }

    /// The frame worn things are built on, and how far to move them from it: a biped's
    /// own frame where it stands, or a taur's upper body's moved up onto its lower.
    pub fn worn_frame(&self) -> Option<(Frame, [i32; 3])> {
        if let Some(frame) = self.frame() {
            return Some((frame, [0, 0, 0]));
        }
        let upper = self.upper?;
        let [x, y, z] = upper.at;
        Some((upper.frame, [x, y - upper.frame.hip, z]))
    }
}
