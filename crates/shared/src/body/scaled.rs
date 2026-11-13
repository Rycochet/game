//! Bodies at other sizes. A biped's layout is at a person's scale (its frame is always
//! a person's height) and a beast's at its own, so two forms are blended at their real
//! sizes (`Layout::at_size`), or a halfling turning into a horse wouldn't grow.

use super::blend::{WHOLE, scaled_frame, scaled_parts};
use super::layout::{Girdle, Layout, Upper};
use super::rings::Ring;
use super::{Frame, HUMAN};
use crate::creature::CreatureKind;

impl Layout {
    /// A kind's body at the size it usually stands: a biped's scaled from a person's
    /// height to its own, anything else as it is.
    pub fn at_size(kind: CreatureKind) -> Option<Layout> {
        let layout = Layout::of(kind)?;
        if layout.frame().is_none() {
            return Some(layout);
        }
        Some(layout.scaled(Frame::usual_height(kind) * WHOLE / HUMAN.height))
    }

    /// The same body this many thousandths of its size, standing on the same ground.
    pub fn scaled(&self, share: i32) -> Layout {
        let size = |x: i32| x * share / WHOLE;
        let mut layout = *self;
        let spine = &mut layout.spine;
        spine.base = spine.base.map(size);
        for station in [&mut spine.hips, &mut spine.waist, &mut spine.chest] {
            station.along = size(station.along);
            station.half = (size(station.half.0), size(station.half.1));
        }
        spine.shoulders = size(spine.shoulders);
        spine.neck_base = size(spine.neck_base);
        for ring in &mut spine.rings {
            *ring = Ring {
                along: size(ring.along),
                half: size(ring.half),
                belly: size(ring.belly),
                back: size(ring.back),
            };
        }
        layout.neck.length = size(layout.neck.length);
        layout.neck.radius = size(layout.neck.radius);
        layout.head.length = size(layout.head.length);
        layout.head.half = (size(layout.head.half.0), size(layout.head.half.1));
        if let Some(shape) = &mut layout.head.shape {
            shape.rings = shape.rings.map(|ring| Ring {
                along: size(ring.along),
                half: size(ring.half),
                belly: size(ring.belly),
                back: size(ring.back),
            });
        }
        for limb in layout.limbs.iter_mut().flatten() {
            limb.apart = size(limb.apart);
            limb.foot = size(limb.foot);
            if let Girdle::Along(along) = limb.girdle {
                limb.girdle = Girdle::Along(size(along));
            }
            for bone in limb.bones.iter_mut().flatten() {
                bone.span = size(bone.span);
                bone.radius = size(bone.radius);
                bone.end_radius = size(bone.end_radius);
            }
        }
        layout.parts = scaled_parts(&layout.parts, share);
        layout.upper = self.upper.map(|upper| Upper {
            frame: scaled_frame(&upper.frame, share),
            at: upper.at.map(size),
        });
        layout
    }
}

#[cfg(test)]
mod tests;
