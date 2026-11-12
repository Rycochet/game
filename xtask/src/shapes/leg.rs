//! A standing beast's legs, measured at a level above the ground: how thick
//! each is there, and where it is, which also shows how the leg bends.

use super::cut::{Plane, cut};
use super::mesh::Mesh;

/// One leg at one level, in the mesh's own units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegCut {
    /// Half its thickness from side to side, and from front to back.
    pub half: (f64, f64),
    /// Its middle: out from the body's middle line, and forwards.
    pub middle: (f64, f64),
}

/// The left foreleg and hind leg at `height`, where each stands clear of the
/// trunk: `None` for one that has joined the trunk, or isn't there.
///
/// A level plane cuts each leg as an outline of its own (a paw's toes as
/// several, taken together), wholly on one side of the body; those in front
/// of `halfway` along the body are the foreleg's.
pub fn legs(mesh: &Mesh, height: f64, halfway: f64) -> (Option<LegCut>, Option<LegCut>) {
    let plane = Plane::facing([0.0, height, 0.0], [0.0, 1.0, 0.0]);
    let mut fore = Bounds::default();
    let mut hind = Bounds::default();
    for outline in cut(mesh, &plane) {
        let across = outline.span(0);
        if across.0 <= 0.0 {
            continue;
        }
        // The plane measures towards the back; legs are told apart forwards.
        let (back, front) = outline.span(1);
        let forwards = (-front, -back);
        let leg = if (forwards.0 + forwards.1) / 2.0 > halfway {
            &mut fore
        } else {
            &mut hind
        };
        leg.add(across, forwards);
    }
    (fore.leg(), hind.leg())
}

/// The box round everything found of one leg.
#[derive(Default)]
struct Bounds(Option<[(f64, f64); 2]>);

impl Bounds {
    fn add(&mut self, across: (f64, f64), forwards: (f64, f64)) {
        let wider = |a: (f64, f64), b: (f64, f64)| (a.0.min(b.0), a.1.max(b.1));
        self.0 = Some(match self.0 {
            Some([a, f]) => [wider(a, across), wider(f, forwards)],
            None => [across, forwards],
        });
    }

    fn leg(&self) -> Option<LegCut> {
        let [across, forwards] = self.0?;
        Some(LegCut {
            half: ((across.1 - across.0) / 2.0, (forwards.1 - forwards.0) / 2.0),
            middle: ((across.0 + across.1) / 2.0, (forwards.0 + forwards.1) / 2.0),
        })
    }
}
