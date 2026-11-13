//! A head's own outline, for an animal whose head is measured (`measured_heads.rs`):
//! rings along the head's line from the back of the skull to the nose, the same
//! `Ring`s a trunk has. The line starts where the head sits on the neck (the neck's
//! top, `Layout::chin`) and runs forwards, leaning `pitch` degrees nose down; a ring's
//! `belly` is how far below the line its underside is, `back` how far above it its
//! top. A head without one (a person's) is an oval with a snout, as always drawn.

use super::layout::Head;
use super::parts::{Parts, Snout};
use super::rings::{HEAD_RINGS, Ring};

/// A head's outline: its lean and its rings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HeadShape {
    /// How far the head's line leans down from level, in degrees: 0 looks straight
    /// ahead, 90 straight down.
    pub pitch: i32,
    pub rings: [Ring; HEAD_RINGS],
}

impl HeadShape {
    /// The rings of a head with no outline of its own: its oval (centred half its
    /// length above the neck's top) and its snout, so it can be blended into one that
    /// has an outline.
    pub fn plain(head: &Head, parts: &Parts) -> HeadShape {
        let (across, deep) = head.half;
        let up = head.length / 2;
        let snout = if parts.snout == Snout::Flat {
            0
        } else {
            parts.snout_length
        };
        let length = 2 * deep + snout;
        let rings = std::array::from_fn(|at| {
            let along = -deep + length * at as i32 / (HEAD_RINGS as i32 - 1);
            // The oval's share at this far along, in thousandths.
            let inside = deep.max(1);
            let share = (1_000_000 - (along * 1000 / inside).pow(2)).max(0).isqrt();
            let (half, high) = (across * share / 1000, up * share / 1000);
            let mut ring = Ring {
                along,
                half,
                belly: high - up,
                back: up + high,
            };
            if snout > 0 && along > deep / 2 {
                // Tapering from half the oval's width to a sixth, a little low.
                let t = ((along - deep / 2) * 1000 / (length - deep / 2).max(1)).min(1000);
                let radius = across / 2 - (across / 2 - across / 6) * t / 1000;
                let middle = -up * 3 / 10;
                ring.half = ring.half.max(radius);
                ring.belly = ring.belly.max(radius - middle);
                ring.back = ring.back.max(middle + radius);
            }
            ring
        });
        HeadShape { pitch: 0, rings }
    }

    /// How far it reaches from its underside to its top, and its half widths across
    /// and from the back of the skull to the nose: what a head without an outline
    /// says, for whatever goes by those (a mane's end, the model's head bone).
    pub fn bounds(&self) -> (i32, (i32, i32)) {
        let most = |of: fn(&Ring) -> i32| self.rings.iter().map(of).max().unwrap_or(0);
        let (below, above) = (most(|ring| ring.belly), most(|ring| ring.back));
        let long = self.rings[HEAD_RINGS - 1].along - self.rings[0].along;
        (below + above, (most(|ring| ring.half), long / 2))
    }
}

#[cfg(test)]
mod tests;
