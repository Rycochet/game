//! A head measured on a person's body (a cow's or a reptile's, as JALdMIC's targets put
//! them on MakeHuman's person) placed on one of our frames: the person's own head
//! (`PERSON_HEAD`) is fitted to the frame's oval, and the other head goes where it sits
//! on the same body. So a minotaur's skull is where a person's would be, under the
//! same helmets, and its muzzle reaches out in front. In `head_on_frame/`: `snout.rs`
//! (the muzzle as long as the people's own, its top sloping from the brow to the
//! nose), `inside.rs` (the skull kept inside the oval, the muzzle under the brow).

mod inside;
mod snout;

use super::Frame;
use super::head_shape::HeadShape;
use super::measured_heads::{MeasuredHead, PERSON_HEAD};
use super::rings::Ring;

/// Where the muzzle begins, in thousandths of the way from the oval head's middle to
/// its front: what is in front of this and below the brow may reach out of the oval.
const MUZZLE: i32 = 300;
/// How high the brow is, in thousandths of the head's height from the chin, as
/// helmets' faces have it (`item_art::armour::head::face`): what is above it is
/// inside the oval, as a helmet's bowl or brim comes down to it.
const BROW: i32 = 600;
/// How far under the brow a muzzle's top stays, in millimetres: a brim's thickness.
const UNDER_BROW: i32 = 12;

impl MeasuredHead {
    /// Its shape on this frame's head, from the frame's chin (`Layout::chin`), in whole
    /// millimetres, with a muzzle reaching `snout` in front of the face (none if 0):
    /// the skull inside the frame's oval head, which every helmet and coif is built
    /// round, and the muzzle under the brow.
    pub fn on_frame(&self, frame: &Frame, snout: i32) -> HeadShape {
        self.placed(frame).with_snout(frame, snout).inside(frame)
    }

    /// Where it goes on the frame's oval, as measured.
    fn placed(&self, frame: &Frame) -> HeadShape {
        // Everything in the mesh's millimetres times a thousand, as a share times a length.
        let person = |share: i32| i64::from(share) * i64::from(PERSON_HEAD.length);
        let rings = PERSON_HEAD.rings;
        let most = |of: fn(&super::MeasuredRing) -> i32| rings.iter().map(of).max().unwrap_or(0);
        let least_belly = rings.iter().map(|ring| ring.belly).min().unwrap_or(0);
        let (back_of_skull, low, high) = (
            person(rings[0].along),
            person(least_belly),
            person(most(|ring| ring.back)),
        );
        let (widest, long) = (person(most(|ring| ring.half)), person(1000));
        let (across, deep) = (i64::from(frame.head.0), i64::from(frame.head.1));
        let tall = i64::from(frame.height - frame.chin);
        let mesh = |share: i32| i64::from(share) * i64::from(self.length);
        // From the back of the oval forwards, and from the chin up.
        let along = |share: i32| -deep + (mesh(share) - back_of_skull) * 2 * deep / long;
        let up = |share: i32| (mesh(share) - low) * tall / (high - low);
        let rings = self.rings.map(|ring| Ring {
            along: along(ring.along) as i32,
            half: (mesh(ring.half) * across / widest) as i32,
            belly: -up(ring.belly) as i32,
            back: up(ring.back) as i32,
        });
        HeadShape {
            pitch: self.pitch - PERSON_HEAD.pitch,
            rings,
        }
    }
}

#[cfg(test)]
mod tests;
