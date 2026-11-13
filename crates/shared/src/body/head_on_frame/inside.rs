//! A beast people's head kept where helmets fit it: the skull inside the frame's oval
//! head, which every helmet and coif is built round, and the muzzle under the brow,
//! where a helmet's bowl or brim comes down to.

use super::super::Frame;
use super::super::head_shape::HeadShape;
use super::super::trig::{ONE, cos, sin};
use super::{BROW, MUZZLE, UNDER_BROW};

impl HeadShape {
    /// The same head with each ring behind the muzzle (`MUZZLE` of the way from the
    /// oval's middle to its front) cut down to the oval's slice where it stands, and
    /// each ring of the muzzle with its top under the brow.
    pub(super) fn inside(mut self, frame: &Frame) -> HeadShape {
        let (across, deep) = frame.head;
        let half_up = (frame.height - frame.chin) / 2;
        let brow = half_up * 2 * BROW / 1000 - UNDER_BROW;
        let (sine, cosine) = (sin(self.pitch), cos(self.pitch));
        for ring in self.rings.iter_mut() {
            // The ring's middle on the head's line and up from it, then where that is
            // from the chin: forwards and up (`HeadAxes` in the client).
            let middle = (ring.back - ring.belly) / 2;
            let deep_half = (ring.back + ring.belly) / 2;
            let z = (ring.along * cosine + middle * sine) / ONE;
            let y = (-ring.along * sine + middle * cosine) / ONE;
            let (top, bottom) = (y + deep_half, y - deep_half);
            let (top, bottom) = if z > deep * MUZZLE / 1000 {
                let top = top.min(brow);
                (top, bottom.min(top))
            } else {
                // The oval's slice at this far forwards, in thousandths of its middle's.
                let slice = (1_000_000 - (z.clamp(-deep, deep) * 1000 / deep).pow(2)).isqrt();
                let (wide, high) = (across * slice / 1000, half_up * slice / 1000);
                let top = top.min(half_up + high);
                let bottom = bottom.max(half_up - high).min(top);
                // The ring is widest at its middle: as wide as the slice at that height.
                let off = (((top + bottom) / 2 - half_up) * 1000 / high.max(1)).clamp(-1000, 1000);
                ring.half = ring.half.min(wide * (1_000_000 - off * off).isqrt() / 1000);
                (top, bottom)
            };
            let (y, deep_half) = ((top + bottom) / 2, (top - bottom) / 2);
            let middle = (y * ONE + ring.along * sine) / cosine;
            ring.back = middle + deep_half;
            ring.belly = deep_half - middle;
        }
        self
    }
}
