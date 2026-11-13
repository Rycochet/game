//! Blending one head into another: its sizes, and its outline ring by ring. A head
//! with no outline of its own (a person's) is blended as its plain oval and snout
//! (`HeadShape::plain`), so a person's face can grow a wolf's muzzle; each end is
//! still exactly its own head, outline or none.

use super::super::Parts;
use super::super::head_shape::HeadShape;
use super::super::layout::{Head, Layout};
use super::super::rings::Ring;
use super::{WHOLE, mix, mix2};

pub(super) fn blend(a: &Layout, b: &Layout, t: i32) -> Head {
    Head {
        length: mix(a.head.length, b.head.length, t),
        half: mix2(a.head.half, b.head.half, t),
        shape: shape(a, b, t),
    }
}

fn shape(a: &Layout, b: &Layout, t: i32) -> Option<HeadShape> {
    match (t, a.head.shape, b.head.shape) {
        (0, own, _) => own,
        (WHOLE, _, own) => own,
        (_, None, None) => None,
        _ => {
            let own = |layout: &Layout| outline(&layout.head, &layout.parts);
            let (a, b) = (own(a), own(b));
            Some(HeadShape {
                pitch: mix(a.pitch, b.pitch, t),
                rings: std::array::from_fn(|at| {
                    let (a, b) = (a.rings[at], b.rings[at]);
                    Ring {
                        along: mix(a.along, b.along, t),
                        half: mix(a.half, b.half, t),
                        belly: mix(a.belly, b.belly, t),
                        back: mix(a.back, b.back, t),
                    }
                }),
            })
        }
    }
}

/// A head's outline, its own or its plain one.
fn outline(head: &Head, parts: &Parts) -> HeadShape {
    head.shape.unwrap_or_else(|| HeadShape::plain(head, parts))
}
