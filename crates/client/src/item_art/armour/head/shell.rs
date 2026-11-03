//! The surface every head piece is built over: the head's own shape grown outwards,
//! drawn up to the skull's point and kept upright below the middle of the head, as a
//! helmet hangs. Angles go round from the front towards the body's left.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::prelude::*;
use shared::armour::Style;
use shared::armour::head::{Edge, HeadForm, Skull};
use shared::body::Frame;

/// A shell round the head: how far out it stands and what shape it rises to.
#[derive(Clone, Copy)]
pub(super) struct Shell<'a> {
    pub frame: &'a Frame,
    pub skull: Skull,
    pub style: Style,
    /// Out from the head, in millimetres.
    pub out: f32,
    pub edge: Edge,
}

impl<'a> Shell<'a> {
    pub fn new(form: &HeadForm, frame: &'a Frame, out: f32) -> Self {
        Self {
            frame,
            skull: form.skull,
            style: form.style,
            out,
            edge: form.edge(frame),
        }
    }

    /// The middle of the head, where the shell is widest.
    pub fn centre(&self) -> f32 {
        (self.frame.height + self.frame.chin) as f32 / 2.0
    }

    /// The head's height, crown to chin.
    pub fn head(&self) -> f32 {
        (self.frame.height - self.frame.chin) as f32
    }

    /// A level on the face, as a share of the head's height up from the chin: eyes 0.5,
    /// brow 0.6, nose 0.27.
    pub fn level(&self, share: f32) -> f32 {
        self.frame.chin as f32 + self.head() * share
    }

    /// How high its point is.
    pub fn apex(&self) -> f32 {
        let rise = match self.skull {
            Skull::Round | Skull::Low => 0.0,
            Skull::Conical | Skull::Ogival => 70.0,
            Skull::Flat => 8.0,
            Skull::Onion => 60.0,
            Skull::Forward => 40.0,
        } + match self.style {
            Style::Elven => 30.0,
            _ => 0.0,
        };
        self.frame.height as f32 + self.out + rise
    }

    /// Where the lower edge is, this far round from the front.
    pub fn edge_at(&self, angle: f32) -> f32 {
        let angle = angle.rem_euclid(TAU);
        let from_front = angle.min(TAU - angle).to_degrees().round() as i32;
        self.edge.height(from_front) as f32
    }

    /// How much of its full width it has at a height: rounded above the middle of the
    /// head by the skull's shape, upright below.
    fn width(&self, y: f32) -> f32 {
        let centre = self.centre();
        if y <= centre {
            return 1.0;
        }
        let t = ((y - centre) / (self.apex() - centre)).clamp(0.0, 1.0);
        let round = (1.0 - t * t).max(0.0).sqrt();
        match self.skull {
            Skull::Round => round,
            Skull::Low => (1.0 - t.powi(4)).max(0.0).sqrt(),
            Skull::Conical | Skull::Ogival | Skull::Forward => round.min(1.7 * (1.0 - t)),
            Skull::Flat if t < 0.9 => 1.0,
            Skull::Flat => (1.0 - ((t - 0.9) / 0.1).powi(2)).max(0.0).sqrt(),
            Skull::Onion => (round * (1.0 + 0.18 * (PI * t * 1.1).sin())).min(2.2 * (1.0 - t)),
        }
    }

    /// How far the shape leans forward or back at a height: an ogival point lies back, a
    /// Phrygian top forward.
    fn lean(&self, y: f32) -> f32 {
        let centre = self.centre();
        let t = ((y - centre) / (self.apex() - centre)).clamp(0.0, 1.0);
        match self.skull {
            Skull::Ogival => -30.0 * t * t,
            Skull::Forward => 70.0 * t * t,
            _ => 0.0,
        }
    }

    /// Its half sizes across and front to back, out from the head.
    fn half(&self) -> (f32, f32) {
        let grow = match self.style {
            Style::Dwarven => 1.06,
            Style::Elven => 0.97,
            _ => 1.0,
        };
        let (across, deep) = (self.frame.head.0 as f32, self.frame.head.1 as f32);
        ((across + self.out) * grow, (deep + self.out) * grow)
    }

    /// The point at this angle round and this height, without its way out.
    pub fn place(&self, angle: f32, y: f32) -> Vec3 {
        let (across, deep) = self.half();
        let width = self.width(y);
        Vec3::new(
            across * width * angle.sin(),
            y,
            deep * width * angle.cos() + self.lean(y),
        )
    }

    /// The point at this angle and height, and the way out from it.
    pub fn point(&self, angle: f32, y: f32) -> (Vec3, Vec3) {
        let here = self.place(angle, y);
        let step = 0.01;
        let round = self.place(angle + step, y) - self.place(angle - step, y);
        let up = self.place(angle, y + 2.0) - self.place(angle, y - 2.0);
        let mut out = round.cross(up).normalize_or(Vec3::Y);
        // Near the point the sides meet; outwards is then up.
        if self.width(y) < 0.05 || out.dot(here - Vec3::new(0.0, self.centre(), 0.0)) < 0.0 {
            out = if self.width(y) < 0.05 { Vec3::Y } else { -out };
        }
        (here, out)
    }

    /// A point on the bowl: `u` round from the front, `v` from its edge up to its point,
    /// closer together near the top where it curves.
    pub fn bowl(&self, u: f32, v: f32) -> (Vec3, Vec3) {
        let angle = u * TAU;
        let bottom = self.edge_at(angle);
        let y = bottom + (self.apex() - bottom) * (v * FRAC_PI_2).sin();
        self.point(angle, y)
    }
}
