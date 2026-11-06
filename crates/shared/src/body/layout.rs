//! Any body's layout, whatever stands on how many legs: a spine that can tilt from
//! upright to level, a neck and head, limbs hung from the hips or shoulders in slots
//! with roles, and what it has beyond a person's (`Parts`). Everything is whole
//! millimetres and degrees at a person's scale, as a `Frame` is, so one form can later
//! be blended into another by changing numbers. A `Frame` is the view of a layout that
//! stands upright on two legs with two arms (`biped_view.rs`).

use super::trig::{ONE, cos, sin};
use super::{Beast, Biped, Frame, GIANT, PERSON, Parts, Side, Taur};
use crate::creature::CreatureKind;

/// The most limbs a body has: a spider's eight legs.
pub const MAX_LIMBS: usize = 8;
/// The most bones in a limb, shoulder or hip down to the ground or the fingertips.
pub const MAX_BONES: usize = 4;

/// One level across the trunk: how far along the spine it is from the hips, and its
/// half widths across and front to back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Station {
    pub along: i32,
    pub half: (i32, i32),
}

/// The trunk, from the middle of the hips to the base of the neck.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Spine {
    /// The middle of the hips: x towards the body's left, y up, z forwards.
    pub base: [i32; 3],
    /// How far it leans forwards, in degrees: 0 upright, as a person; 90 level, as a
    /// horse's back.
    pub pitch: i32,
    pub hips: Station,
    pub waist: Station,
    pub chest: Station,
    /// How far along the shoulder joints are.
    pub shoulders: i32,
    /// How far along the neck starts: the spine's length.
    pub neck_base: i32,
}

/// The neck from the top of the spine to the chin, and the head above it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Neck {
    pub length: i32,
    /// Its thickness, as a radius.
    pub radius: i32,
    /// How far it leans forwards, in degrees: 0 upright, as a person's; a horse's
    /// reaches up and forwards.
    pub pitch: i32,
}

/// The head, from the chin to the crown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Head {
    pub length: i32,
    /// Half widths across and front to back.
    pub half: (i32, i32),
}

/// What a limb is for: what may be worn on it, and what it becomes when one form turns
/// into another (arms into forelegs, legs into hind legs).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Arm,
    /// An arm that is a wing, with feathers along it.
    WingArm,
    Leg,
    Foreleg,
    Hindleg,
    /// One of an insect's or a spider's legs.
    JointedLeg,
}

/// Where on the spine a limb hangs from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Girdle {
    Hips,
    Shoulders,
    /// This far along the spine from the hips, as a spider's legs are spread along it.
    Along(i32),
}

/// One bone of a limb. Its far end is `way` × `span` / 1000 from its near end: `way` is
/// a direction as millimetres per metre (x outwards from the body, y up, z forwards),
/// so a bone hanging straight down is `[0, -1000, 0]` and its span is its length.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Bone {
    pub span: i32,
    pub way: [i32; 3],
    /// Its thickness along it, and at the joint it ends in, as radii.
    pub radius: i32,
    pub end_radius: i32,
}

/// A limb in one of a body's slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Limb {
    pub role: Role,
    pub girdle: Girdle,
    pub side: Side,
    /// How far out from the spine it hangs.
    pub apart: i32,
    /// From the body outwards; the rest of the slots empty.
    pub bones: [Option<Bone>; MAX_BONES],
    /// How long its foot is along the ground, 0 if it doesn't stand on one.
    pub foot: i32,
}

/// A person's upper body standing on a beast's, as a centaur's: its frame (only the
/// part from the hips up is drawn or worn on), and where on the lower body the middle
/// of its hips is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Upper {
    pub frame: Frame,
    pub at: [i32; 3],
}

/// A whole body, standing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Layout {
    pub spine: Spine,
    /// The neck and head; a taur's lower body has none (all 0), the upper body's
    /// being its own.
    pub neck: Neck,
    pub head: Head,
    pub limbs: [Option<Limb>; MAX_LIMBS],
    pub parts: Parts,
    /// An upright upper body standing on this one, if any.
    pub upper: Option<Upper>,
}

impl Layout {
    /// A kind's body, for the kinds drawn so far: those on two legs, and animals.
    pub fn of(kind: CreatureKind) -> Option<Layout> {
        let parts = Parts::of(kind);
        match kind {
            CreatureKind::Human | CreatureKind::Beggar => Some(Self::biped(&PERSON.frame(), parts)),
            CreatureKind::Giant => Some(Self::biped(&GIANT, parts)),
            _ => Biped::of(kind)
                .map(|biped| Self::biped(&biped.proportions.frame(), parts))
                .or_else(|| Beast::of(kind).map(|beast| beast.layout()))
                .or_else(|| Taur::of(kind).map(|taur| taur.layout())),
        }
    }

    /// Where the neck's top is: the chin, under the head.
    pub fn chin(&self) -> [i32; 3] {
        let [x, y, z] = self.spine.at(self.spine.neck_base);
        let length = self.neck.length;
        [
            x,
            y + length * cos(self.neck.pitch) / ONE,
            z + length * sin(self.neck.pitch) / ONE,
        ]
    }

    /// The limbs it has.
    pub fn limbs(&self) -> impl Iterator<Item = &Limb> {
        self.limbs.iter().flatten()
    }

    /// Its limb of this role on this side, the first if it has several.
    pub fn limb(&self, role: Role, side: Side) -> Option<&Limb> {
        self.limbs()
            .find(|limb| limb.role == role && limb.side == side)
    }

    /// Where a limb hangs from.
    pub fn root(&self, limb: &Limb) -> [i32; 3] {
        let along = match limb.girdle {
            Girdle::Hips => 0,
            Girdle::Shoulders => self.spine.shoulders,
            Girdle::Along(along) => along,
        };
        let [x, y, z] = self.spine.at(along);
        [x + limb.side.sign() * limb.apart, y, z]
    }

    /// A limb's joints, from where it hangs to the end of its last bone.
    pub fn joints(&self, limb: &Limb) -> Vec<[i32; 3]> {
        let mut at = self.root(limb);
        let mut joints = vec![at];
        for bone in limb.bones.iter().flatten() {
            let [x, y, z] = bone.way;
            let step = |v: i32| v * bone.span / 1000;
            at = [
                at[0] + limb.side.sign() * step(x),
                at[1] + step(y),
                at[2] + step(z),
            ];
            joints.push(at);
        }
        joints
    }
}

impl Spine {
    /// The point this far along it from the middle of the hips.
    pub fn at(&self, along: i32) -> [i32; 3] {
        let [x, y, z] = self.base;
        [
            x,
            y + along * cos(self.pitch) / ONE,
            z + along * sin(self.pitch) / ONE,
        ]
    }
}

#[cfg(test)]
mod tests;
