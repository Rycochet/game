//! A beast people's muzzle as long as its own (`Parts::snout_length`), whatever the
//! mesh's was, its top sloping straight from the brow down to the nose.

use super::super::Frame;
use super::super::head_shape::HeadShape;
use super::super::rings::HEAD_RINGS;
use super::{BROW, MUZZLE, UNDER_BROW};

impl HeadShape {
    /// The same head with what is in front of the face (the oval's front, along its
    /// line) stretched or squeezed so its nose's ring is `snout` further on, and its
    /// top never under the straight line from the brow, where the muzzle begins, to
    /// the nose, as a reptile's or a bull's is.
    pub(super) fn with_snout(mut self, frame: &Frame, snout: i32) -> HeadShape {
        let face = frame.head.1;
        let nose = self.rings[HEAD_RINGS - 1];
        if nose.along <= face || snout <= 0 {
            return self;
        }
        for ring in self.rings.iter_mut().filter(|ring| ring.along > face) {
            ring.along = face + (ring.along - face) * snout / (nose.along - face);
        }
        let nose = self.rings[HEAD_RINGS - 1];
        let start = face * MUZZLE / 1000;
        let brow = (frame.height - frame.chin) * BROW / 1000 - UNDER_BROW;
        let long = (nose.along - start).max(1);
        for ring in self.rings[..HEAD_RINGS - 1]
            .iter_mut()
            .filter(|ring| ring.along > start)
        {
            let line = brow + (nose.back - brow) * (ring.along - start) / long;
            ring.back = ring.back.max(line);
        }
        self
    }
}
