//! The shell torso armour is built over: a surface round the trunk from just below the
//! waist to the shoulders, cut low for the neck in front and round each arm, and shaped
//! by the breast. Agreed edges: the hem at `waist` with a flange to `waist - 20`, the
//! shoulder straps between about `neck × 1.9` and `shoulder_half - 50` from the middle,
//! the arm holes from `shoulder - 150` up.

use std::f32::consts::PI;

use bevy::prelude::*;
use shared::armour::torso::Breast;
use shared::body::Frame;

use crate::item_art::body::trunk_half;

/// Which part of the way round the shell covers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Span {
    /// The breast, lapping a little round each side.
    Front,
    /// The back, under the breast's edges.
    Back,
    /// Right round, as mail and padding go.
    Round,
}

impl Span {
    /// Angles round from the body's left (x), through its front (z), as u goes 0 to 1.
    fn angles(self) -> (f32, f32) {
        match self {
            Self::Front => (-0.22, PI + 0.22),
            Self::Back => (PI - 0.12, 2.0 * PI + 0.12),
            Self::Round => (0.0, 2.0 * PI),
        }
    }
}

/// A shell: how far out from the skin, what shape of breast, and where it stops.
#[derive(Clone, Copy)]
pub(super) struct Shell<'a> {
    pub frame: &'a Frame,
    pub span: Span,
    /// Out from the skin, in millimetres.
    pub out: f32,
    pub breast: Option<Breast>,
    /// How far below the waist the hem hangs.
    pub hem: f32,
    /// Its top at most this far up its usual edge, 0 to 1: a plackart's or a band's.
    pub reach: Option<&'a dyn Fn(f32) -> f32>,
}

fn hump(t: f32, centre: f32, width: f32) -> f32 {
    (-((t - centre) / width).powi(2)).exp()
}

impl Shell<'_> {
    /// How high the edge is at this angle: the neck dips in front, the arm holes at
    /// the sides, the straps between.
    pub fn top(&self, angle: f32) -> f32 {
        let frame = self.frame;
        let front = angle.sin() > 0.0;
        let side = angle.cos().abs();
        let neck = frame.neck_base as f32 - if front { 60.0 } else { 15.0 };
        let strap = frame.shoulder as f32 - 5.0;
        let pit = frame.shoulder as f32 - 150.0;
        if side < 0.5 {
            let t = side / 0.5;
            neck + (strap - neck) * t * t * (3.0 - 2.0 * t)
        } else {
            strap - (strap - pit) * ((side - 0.5) / 0.5).powf(1.6)
        }
    }

    /// How low the hem is at this angle: some breasts dip to a point over the belly.
    pub fn bottom(&self, angle: f32) -> f32 {
        let front = angle.sin().max(0.0);
        let dip = match self.breast {
            Some(Breast::Peascod) => 50.0 * front.powi(6),
            Some(Breast::Muscle) => 40.0 * front.powi(3),
            Some(Breast::Leaf) => 70.0 * front.powi(10),
            Some(Breast::Gothic) => 20.0 * front.powi(4),
            _ => 0.0,
        };
        self.frame.waist as f32 - 20.0 - self.hem - dip
    }

    /// How far a breast's shape stands out at this angle and height.
    fn shaping(&self, angle: f32, y: f32, x: f32) -> f32 {
        let frame = self.frame;
        let front = angle.sin().max(0.0);
        let t = (y - frame.waist as f32) / (frame.chest - frame.waist) as f32;
        let chest = frame.chest as f32;
        let Some(breast) = self.breast else {
            return 0.0;
        };
        // Round the back only a little.
        if angle.sin() < 0.0 {
            return if breast == Breast::Tanko {
                10.0 - 15.0 * hump(t, 0.2, 0.4)
            } else {
                0.0
            };
        }
        let f2 = front * front;
        match breast {
            Breast::Round => 18.0 * f2 * hump(t, 0.6, 0.7),
            Breast::Globose => 38.0 * f2 * hump(t, 0.35, 0.8),
            Breast::Gothic => 15.0 * f2 * hump(t, 0.85, 0.6) - 10.0 * (0.5 - t).max(0.0),
            Breast::Box => {
                let forward = if t < 1.05 {
                    40.0
                } else {
                    40.0 * (1.0 - (t - 1.05) / 0.9).max(0.0)
                };
                forward * front.powf(1.5)
            }
            Breast::Peascod => 32.0 * front.powi(4) * hump(t, 0.1, 0.6),
            Breast::Munition => 8.0 * f2,
            Breast::Muscle => {
                let pec = |side: f32| hump(x, side * 70.0, 50.0) * hump(y, chest + 25.0, 55.0);
                let furrow = 5.0 * hump(x, 0.0, 12.0) * (t < 1.0) as u8 as f32;
                14.0 * f2 * hump(t, 0.5, 1.0) + 14.0 * (pec(1.0) + pec(-1.0)) - furrow
                    + 3.0
                        * ((t * 3.0 * PI).sin().abs())
                        * hump(x, 0.0, 60.0)
                        * (t < 0.9) as u8 as f32
            }
            Breast::Bell => 10.0 * f2 * hump(t, 0.8, 0.6),
            Breast::Tanko => 10.0 - 15.0 * hump(t, 0.2, 0.4),
            Breast::Leaf => 10.0 * f2 * hump(t, 0.7, 0.7),
            Breast::Panel => 26.0 * front.powf(0.7),
            Breast::Discs => 0.0,
        }
    }

    /// A point on it and the way out from it: u round, v up from the hem to the edge.
    pub fn at(&self, u: f32, v: f32) -> (Vec3, Vec3) {
        let (from, to) = self.span.angles();
        let angle = from + (to - from) * u;
        let (bottom, top) = (self.bottom(angle), self.top(angle));
        let top = match self.reach {
            Some(reach) => bottom + (top - bottom) * reach(angle),
            None => top,
        };
        let y = bottom + (top - bottom) * v;
        let (across, deep) = trunk_half(self.frame, y.min(self.frame.shoulder as f32 - 40.0));
        let x = across * angle.cos();
        // Bell cuirasses flare out at the hem, all round.
        let flare = if self.breast == Some(Breast::Bell) {
            30.0 * (1.0 - v / 0.2).max(0.0).powi(2)
        } else {
            0.0
        };
        let out = self.out + self.shaping(angle, y, x) + flare;
        let (a, b) = (across + out, deep + out);
        let point = Vec3::new(a * angle.cos(), y, b * angle.sin());
        let normal = Vec3::new(angle.cos() / a, 0.0, angle.sin() / b).normalize_or(Vec3::Z);
        (point, normal)
    }
}
