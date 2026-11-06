//! Taurs: a person's upper body standing on a beast's, where the beast's neck would
//! be, as a centaur's on a horse. The lower body is one of `BEASTS`, at its own size;
//! the upper body is a frame from proportions at a person's, as a biped's is, so
//! everything worn on a person's upper body is built on it and moved into place.

use super::layout::{Layout, Role, Upper};
use super::{BEASTS, Frame, NONE, PERSON, Parts, Plan, Proportions};
use crate::creature::CreatureKind;

/// A taur, by name: the beast below, the person above and its head's parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Taur {
    pub name: &'static str,
    /// The name of the beast in `BEASTS` it stands on.
    pub lower: &'static str,
    pub upper: Proportions,
    /// What its head has beyond a person's; the tail and feet are the beast's.
    pub parts: Parts,
}

pub const TAURS: [Taur; 1] = [Taur {
    name: "Centaur",
    lower: "Horse",
    upper: PERSON,
    parts: NONE,
}];

impl Taur {
    /// The taur of this kind, by its name.
    pub fn of(kind: CreatureKind) -> Option<Taur> {
        TAURS.into_iter().find(|taur| taur.name == kind.def().name)
    }

    /// The beast's layout with the upper body standing where its neck would be: the
    /// person's hips sunk into the top of its chest, above its forelegs.
    pub fn layout(&self) -> Layout {
        let beast = BEASTS
            .into_iter()
            .find(|beast| beast.name == self.lower)
            .expect("a taur stands on one of the beasts");
        let mut layout = beast.layout();
        let Plan::Fourfoot(four) = beast.plan else {
            return layout;
        };
        let deep = four.barrel.1;
        let [x, _, z] = layout.spine.at(layout.spine.shoulders);
        layout.upper = Some(Upper {
            frame: self.upper.frame(),
            at: [x, four.withers - deep / 3, z + deep / 3],
        });
        layout.parts = Parts {
            tail: beast.parts.tail,
            feet: beast.parts.feet,
            ..self.parts
        };
        layout
    }
}

impl Layout {
    /// The upper body as a layout of its own, in place: its trunk from the hips up,
    /// arms, neck and head, with no legs or tail.
    pub fn upper_body(&self) -> Option<Layout> {
        let upper = self.upper?;
        let parts = Parts {
            tail: 0,
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
