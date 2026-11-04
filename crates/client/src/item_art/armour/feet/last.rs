//! The shape foot armour is built over: the bare foot as `item_art/body.rs` draws it
//! (a stretched ball from heel to toes), made bigger by a lift for the shoe, with its
//! front drawn out to the toe a piece has. A shoemaker's last, in millimetres.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use shared::armour::feet::Toe;
use shared::body::{Frame, Joint, Side};

/// Where the toe's own shape starts, behind the toes' tips.
const TOE_ZONE: f32 = 50.0;
/// Room kept between a broad toe and the middle line, so a pair never meet.
const BETWEEN: f32 = 4.0;

/// One foot of a frame.
pub(in crate::item_art) struct Foot<'a> {
    pub frame: &'a Frame,
    pub side: Side,
}

impl Foot<'_> {
    fn point(&self, joint: Joint) -> Vec3 {
        let [x, y, z] = self.frame.joint(joint);
        Vec3::new(x as f32, y as f32, z as f32)
    }

    /// Across the body: the foot's middle line.
    pub fn x(&self) -> f32 {
        self.point(Joint::Ankle(self.side)).x
    }

    /// The bare foot's back and front, along z.
    pub fn heel(&self) -> f32 {
        self.point(Joint::Heel(self.side)).z
    }

    pub fn toes(&self) -> f32 {
        self.point(Joint::Toes(self.side)).z
    }

    /// The bare foot's half width, half height and middle height: the ball's half
    /// sizes, as the body draws it.
    fn ball(&self) -> (f32, f32, f32) {
        let half_height = self.frame.ankle as f32 / 2.0;
        let half_width = self.frame.ankle_radius as f32 * 1.3;
        (half_width, half_height, half_height)
    }

    /// How far the bare foot reaches out from its middle line at this z and height,
    /// for tests: negative outside the foot.
    #[cfg(test)]
    pub fn bare_reach(&self, z: f32, y: f32) -> f32 {
        let (wide, high, middle) = self.ball();
        let half_long = (self.toes() - self.heel()) / 2.0;
        let along = (z - (self.toes() + self.heel()) / 2.0) / half_long;
        let up = (y - middle) / high;
        let left = 1.0 - along * along - up * up;
        if left <= 0.0 {
            -1.0
        } else {
            wide * left.sqrt()
        }
    }
}

/// The surface of a piece over one foot: `lift` millimetres out from the bare foot,
/// with the front shaped to its toe.
pub(in crate::item_art) struct Last<'a> {
    pub foot: &'a Foot<'a>,
    pub lift: f32,
    pub toe: Toe,
    /// Millimetres a pointed toe reaches past the round one.
    pub point: f32,
    /// Hundredths of the foot's width at a broad toe.
    pub width: f32,
    /// How far a point curves up at its tip (elven).
    pub up: f32,
}

impl Last<'_> {
    /// The round front, the bare foot's front with the lift.
    fn round_front(&self) -> f32 {
        self.foot.toes() + self.lift
    }

    /// The front of the piece along z.
    pub fn tip(&self) -> f32 {
        self.round_front()
            + match self.toe {
                Toe::Pointed => self.point,
                Toe::DuckBill => 18.0,
                Toe::BearPaw => 10.0,
                Toe::CowMouth => 6.0,
                Toe::Round | Toe::Claw => 0.0,
            }
    }

    /// The back of the piece along z.
    pub fn back(&self) -> f32 {
        self.foot.heel() - self.lift
    }

    /// The lifted ball's half width, half height and middle height at z.
    fn rounded(&self, z: f32) -> (f32, f32, f32) {
        let (wide, high, middle) = self.foot.ball();
        let half_long = (self.foot.toes() - self.foot.heel()) / 2.0 + self.lift;
        let along = (z - (self.foot.toes() + self.foot.heel()) / 2.0) / half_long;
        let s = (1.0 - along * along).max(0.0).sqrt();
        ((wide + self.lift) * s, (high + self.lift) * s, middle)
    }

    /// The section's half width, half height and middle height at z.
    pub fn section(&self, z: f32) -> (f32, f32, f32) {
        let start = self.foot.toes() - TOE_ZONE;
        if z <= start || matches!(self.toe, Toe::Round | Toe::Claw) {
            return self.rounded(z);
        }
        let (wide, high, middle) = self.rounded(start);
        let t = ((z - start) / (self.tip() - start)).clamp(0.0, 1.0);
        let shaped = match self.toe {
            Toe::Pointed => {
                let taper = (1.0 - t).powf(1.3);
                let tip_height = 18.0 + self.up * t * t;
                (
                    wide * taper + 1.5 * t,
                    high * (1.0 - t).powf(1.1) + 1.5 * t,
                    middle + (tip_height - middle) * t,
                )
            }
            _ => {
                // Broad: widening and flattening, then closing round or square.
                let (bare_wide, _, _) = self.foot.ball();
                let broad = bare_wide * self.width / 100.0 + self.lift;
                let grow = smooth(t / 0.6);
                let close_from = if self.toe == Toe::CowMouth { 0.9 } else { 0.6 };
                let closing = if t > close_from {
                    let k = (t - close_from) / (1.0 - close_from);
                    (1.0 - k * k).max(0.0).sqrt()
                } else {
                    1.0
                };
                let flat = 22.0 + self.lift / 2.0;
                (
                    (wide + (broad - wide) * grow) * closing,
                    (high + (flat - high) * grow) * closing.sqrt(),
                    middle + (24.0 - middle) * grow,
                )
            }
        };
        self.over_the_toes(z, shaped)
    }

    /// A shaped section raised and widened where it would cut into the toes: never
    /// smaller than the plain lifted foot, its lower edge kept where it was.
    fn over_the_toes(&self, z: f32, (wide, high, middle): (f32, f32, f32)) -> (f32, f32, f32) {
        let (round_wide, round_high, round_middle) = self.rounded(z);
        let bottom = middle - high;
        let top = (middle + high).max(round_middle + round_high);
        (
            wide.max(round_wide),
            (top - bottom) / 2.0,
            (top + bottom) / 2.0,
        )
    }

    /// A point on the surface at z and this far round from the top: minus a quarter
    /// turn is the inside edge level with the foot's middle, plus a quarter the outside.
    pub fn at(&self, z: f32, round: f32) -> Vec3 {
        let (wide, high, middle) = self.section(z);
        // A broad toe keeps clear of the other foot.
        let wide = if round < 0.0 {
            wide.min(self.foot.x().abs() - BETWEEN)
        } else {
            wide
        };
        let sign = self.foot.side.sign() as f32;
        let y = (middle + high * round.cos()).max(1.5);
        let point = Vec3::new(self.foot.x() + sign * wide * round.sin(), y, z);
        self.round_the_leg(point)
    }

    /// Where the leg comes down into the foot, a point is pushed out round it, so the
    /// piece makes a collar there rather than going through the ankle.
    fn round_the_leg(&self, point: Vec3) -> Vec3 {
        let frame = self.foot.frame;
        let radius = frame.ankle_radius as f32 + self.lift;
        let off = Vec2::new(point.x - self.foot.x(), point.z);
        if point.y < frame.ankle as f32 - 15.0 || off.length() >= radius {
            return point;
        }
        let off = off.normalize_or(Vec2::Y) * radius;
        Vec3::new(self.foot.x() + off.x, point.y, off.y)
    }

    /// The point and the way out from it, from z along and how far round.
    pub fn point_out(&self, z: f32, round: f32) -> (Vec3, Vec3) {
        let point = self.at(z, round);
        let step = 0.5;
        let across = self.at(z, round + 0.02) - self.at(z, round - 0.02);
        let along = self.at(z + step, round) - self.at(z - step, round);
        let (_, _, middle) = self.section(z);
        let from_middle = point - Vec3::new(self.foot.x(), middle, z);
        let mut out = across.cross(along).normalize_or(Vec3::ZERO);
        if out == Vec3::ZERO {
            out = from_middle.normalize_or(Vec3::Y);
        }
        if out.dot(from_middle) < 0.0 {
            out = -out;
        }
        (point, out)
    }

    /// A surface over part of it, for `surface`'s helpers: u across from `-round` to
    /// `round`, v along from `from` to `to`.
    pub fn patch(
        &self,
        (from, to): (f32, f32),
        round: f32,
    ) -> impl Fn(f32, f32) -> (Vec3, Vec3) + '_ {
        move |u, v| self.point_out(from + (to - from) * v, -round + 2.0 * round * u)
    }
}

/// How far round the sides a piece over the top of the foot reaches: a little past
/// level with the foot's middle.
pub(super) const OVER_TOP: f32 = FRAC_PI_2 * 1.18;

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
