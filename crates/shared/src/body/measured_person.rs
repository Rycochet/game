//! People's trunks as measured from MakeHuman's base person (CC0) by `cargo xtask
//! shapes`, which `cargo xtask test` checks these figures still match. Every figure is in
//! thousandths of the person's height, as the tool prints it.
//!
//! Only the trunk's front and back come from the mesh: how far the chest stands out in
//! front of the shoulders, the belly, the small of the back. Widths are
//! still the frame's (hips, waist, chest, shoulders), and the crotch and neck rings are
//! as they always were, so legs, head and everything built on them stay where they are.

use super::Frame;
use super::measured::MeasuredRing;
use super::rings::{RINGS, Ring};

/// A body standing upright as measured: the heights of its crotch, hip joints,
/// shoulder joints and neck's base, and rings from the crotch to the neck's base, each
/// with its height (`along`), half width, and its front (`belly`) and back measured
/// from a line straight up through the hip joints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeasuredUpright {
    /// The mesh it was measured from, under `spare_assets/makehuman/`.
    pub source: &'static str,
    pub crotch: i32,
    pub hips: i32,
    pub shoulders: i32,
    pub neck: i32,
    pub rings: [MeasuredRing; RINGS],
}

const fn ring(along: i32, half: i32, belly: i32, back: i32) -> MeasuredRing {
    MeasuredRing {
        along,
        half,
        belly,
        back,
    }
}

/// MakeHuman's base person, which every people's trunk is shaped after.
pub const PERSON_TRUNK: MeasuredUpright = MeasuredUpright {
    source: "base.obj",
    crotch: 466,
    hips: 519,
    shoulders: 805,
    neck: 844,
    rings: [
        ring(466, 114, 32, 3),
        ring(508, 105, 70, 50),
        ring(550, 101, 78, 50),
        ring(592, 92, 77, 35),
        ring(634, 75, 81, 21),
        ring(676, 73, 88, 19),
        ring(718, 81, 82, 26),
        ring(760, 164, 67, 34),
        ring(802, 139, 51, 37),
        ring(844, 37, 35, 22),
    ],
};

/// `x` on the line through `points` (in order of their first figure), straight
/// between them and level past either end.
fn piecewise(x: i32, points: &[(i32, i32)]) -> i32 {
    let (first, last) = (points[0], points[points.len() - 1]);
    if x <= first.0 {
        return first.1;
    }
    if x >= last.0 {
        return last.1;
    }
    let pair = points
        .windows(2)
        .find(|pair| x <= pair[1].0)
        .expect("between the ends");
    let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
    y0 + (y1 - y0) * (x - x0) / (x1 - x0)
}

impl MeasuredUpright {
    /// Its rings on a frame: each measured ring at the height its level comes to on
    /// the frame (crotch, hip joints, shoulders and neck matched), as wide as the frame
    /// is there, and as deep as the frame's hips, waist and chest, its front and back
    /// split as measured. The trunk leans back towards the neck so the neck stays over
    /// the spine. The shape comes in from the hip joints, round as before, to the waist;
    /// the crotch ring is the hips' all round, and the neck's base as before.
    pub fn rings(&self, f: &Frame) -> [Ring; RINGS] {
        let crotch = f.hip - f.thigh;
        let levels = [
            (self.crotch, crotch),
            (self.hips, f.hip),
            (self.shoulders, f.shoulder),
            (self.neck, f.neck_base),
        ];
        let to_frame = |u: i32| piecewise(u, &levels);
        let to_measured = |y: i32| piecewise(y, &levels.map(|(u, y)| (y, u)));
        let rings = self.rings;
        let deep = |u: i32| {
            let front = piecewise(u, &rings.map(|r| (r.along, r.belly)));
            let back = piecewise(u, &rings.map(|r| (r.along, r.back)));
            (front + back).max(1)
        };
        // Millimetres per thousand measured units, so the frame's depths are kept.
        let scale = |y: i32, half: (i32, i32)| (y, 2 * half.1 * 1000 / deep(to_measured(y)));
        let scales = [
            scale(f.hip, f.hips_half),
            scale(f.waist, f.waist_half),
            scale(f.chest, f.chest_half),
        ];
        let widths = [
            (f.hip, f.hips_half.0),
            (f.waist, f.waist_half.0),
            (f.chest, f.chest_half.0),
            (f.shoulder, f.shoulder_half),
            (f.neck_base, f.neck * 16 / 10),
        ];
        // As deep as the frame, all round: what the trunk was before it was shaped.
        let depths = [
            (f.hip, f.hips_half.1),
            (f.waist, f.waist_half.1),
            (f.chest, f.chest_half.1),
        ];
        let top = rings[RINGS - 1];
        let lean = (top.belly - top.back) * piecewise(f.neck_base, &scales) / 2000;
        let mut shaped = rings.map(|r| {
            let y = to_frame(r.along);
            let k = piecewise(y, &scales);
            let shift = lean * (y - f.hip).max(0) / (f.neck_base - f.hip);
            // The shape comes in from the hip joints to the waist, so the hips stay round
            // where the legs and what hangs over them meet the trunk.
            let shaped = piecewise(y, &[(f.hip, 0), (f.waist, 1000)]);
            let round = piecewise(y, &depths);
            let blend = |measured: i32| round + (measured - round) * shaped / 1000;
            Ring {
                along: y - f.hip,
                half: piecewise(y, &widths),
                belly: blend(r.belly * k / 1000 - shift),
                back: blend(r.back * k / 1000 + shift),
            }
        });
        shaped[0] = Ring::round(-f.thigh, f.hips_half);
        shaped[RINGS - 1] = Ring::round(f.neck_base - f.hip, (f.neck * 16 / 10, f.neck * 12 / 10));
        shaped
    }
}

#[cfg(test)]
mod tests;
