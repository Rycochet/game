//! Blending one body into another, for shapeshifting: every number of a layout moves
//! from one form's to the other's, limbs paired by what they become (arms into
//! forelegs, legs into hind legs), anything with no partner growing from or shrinking
//! to a point. Both ends are exactly the two layouts, so a change starts and finishes
//! on the forms themselves. Blend layouts at their real sizes (`Layout::at_size`).

use super::layout::{Head, Layout, Neck, Spine, Station, Upper};

mod frame;
mod limbs;
mod parts;

pub(super) use frame::{scaled as scaled_frame, zip};
pub(super) use parts::scaled as scaled_parts;

/// A whole blend, in thousandths: 0 is the first body, `WHOLE` the second.
pub const WHOLE: i32 = 1000;
/// Halfway, where what can't be blended (the kind of snout, feet, a limb's role)
/// changes from the first body's to the second's.
const HALF: i32 = WHOLE / 2;

/// `t` thousandths of the way from `a` to `b`: exactly `a` at 0 and `b` at `WHOLE`.
fn mix(a: i32, b: i32, t: i32) -> i32 {
    a + (b - a) * t / WHOLE
}

fn mix3(a: [i32; 3], b: [i32; 3], t: i32) -> [i32; 3] {
    [mix(a[0], b[0], t), mix(a[1], b[1], t), mix(a[2], b[2], t)]
}

fn mix2(a: (i32, i32), b: (i32, i32), t: i32) -> (i32, i32) {
    (mix(a.0, b.0, t), mix(a.1, b.1, t))
}

impl Layout {
    /// `t` thousandths of the way from body `a` to body `b` (clamped to 0 to `WHOLE`):
    /// `a` itself at 0, `b` itself at `WHOLE`.
    pub fn blend(a: &Layout, b: &Layout, t: i32) -> Layout {
        let t = t.clamp(0, WHOLE);
        let spine = mix_spine(&a.spine, &b.spine, t);
        let mut layout = Layout {
            spine,
            neck: Neck {
                length: mix(a.neck.length, b.neck.length, t),
                radius: mix(a.neck.radius, b.neck.radius, t),
                pitch: mix(a.neck.pitch, b.neck.pitch, t),
            },
            head: Head {
                length: mix(a.head.length, b.head.length, t),
                half: mix2(a.head.half, b.head.half, t),
            },
            limbs: limbs::blend(a, b, t),
            parts: parts::blend(&a.parts, &b.parts, t),
            upper: None,
        };
        layout.upper = mix_upper(a, b, &layout, t);
        layout
    }
}

fn mix_spine(a: &Spine, b: &Spine, t: i32) -> Spine {
    let station = |a: Station, b: Station| Station {
        along: mix(a.along, b.along, t),
        half: mix2(a.half, b.half, t),
    };
    Spine {
        base: mix3(a.base, b.base, t),
        pitch: mix(a.pitch, b.pitch, t),
        hips: station(a.hips, b.hips),
        waist: station(a.waist, b.waist),
        chest: station(a.chest, b.chest),
        shoulders: mix(a.shoulders, b.shoulders, t),
        neck_base: mix(a.neck_base, b.neck_base, t),
    }
}

/// A taur's upper body: blended into another's, or, with none on the other side,
/// growing out of (or sinking into) the top of the lower body over its shoulders.
fn mix_upper(a: &Layout, b: &Layout, blended: &Layout, t: i32) -> Option<Upper> {
    match (a.upper, b.upper) {
        (Some(ua), Some(ub)) => Some(Upper {
            frame: zip(&ua.frame, &ub.frame, |x, y| mix(x, y, t)),
            at: mix3(ua.at, ub.at, t),
        }),
        (Some(upper), None) => grown(a, upper, blended, WHOLE - t),
        (None, Some(upper)) => grown(b, upper, blended, t),
        (None, None) => None,
    }
}

/// An upper body this many thousandths of its size, scaled about its hips, which
/// stand as far from the blended body's shoulders as they did on its own: nothing at 0.
fn grown(own: &Layout, upper: Upper, blended: &Layout, share: i32) -> Option<Upper> {
    if share <= 0 {
        return None;
    }
    let size = |x: i32| x * share / WHOLE;
    let frame = scaled_frame(&upper.frame, share);
    let own_shoulders = own.spine.at(own.spine.shoulders);
    let shoulders = blended.spine.at(blended.spine.shoulders);
    let at = [0, 1, 2].map(|i| shoulders[i] + size(upper.at[i] - own_shoulders[i]));
    Some(Upper { frame, at })
}

#[cfg(test)]
mod tests;
