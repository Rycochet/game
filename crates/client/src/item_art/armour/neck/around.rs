//! Where things are round the neck, in millimetres on a body frame: the head, the
//! trunk and the tops of the shoulders that neck pieces lie over, and the curtains
//! ("skirts") that hang from a ring at the top down over them. Angles go round from
//! the front (+z) towards the body's left (+x).

use bevy::prelude::*;
use shared::body::{Frame, Joint, Side};

use crate::item_art::body::trunk_half;

/// A horizontal direction this far round from the front.
pub(super) fn dir(angle: f32) -> Vec3 {
    Vec3::new(angle.sin(), 0.0, angle.cos())
}

/// A point on an ellipse round the middle at a height, pushed out from it by `off`.
fn on_ellipse((across, deep): (f32, f32), y: f32, angle: f32, off: f32) -> Vec3 {
    let point = Vec3::new(across * angle.sin(), y, deep * angle.cos());
    let out = Vec3::new(
        angle.sin() / across.max(1.0),
        0.0,
        angle.cos() / deep.max(1.0),
    );
    point + out.normalize_or_zero() * off
}

/// The body round the neck, from its frame.
pub(in crate::item_art) struct Around<'a> {
    pub frame: &'a Frame,
}

impl Around<'_> {
    fn f(value: i32) -> f32 {
        value as f32
    }

    pub fn neck(&self) -> f32 {
        Self::f(self.frame.neck)
    }

    pub fn neck_base(&self) -> f32 {
        Self::f(self.frame.neck_base)
    }

    pub fn chin(&self) -> f32 {
        Self::f(self.frame.chin)
    }

    pub fn shoulder(&self) -> f32 {
        Self::f(self.frame.shoulder)
    }

    pub fn shoulder_half(&self) -> f32 {
        Self::f(self.frame.shoulder_half)
    }

    pub fn upper_arm(&self) -> f32 {
        Self::f(self.frame.upper_arm)
    }

    /// From the chin to the crown.
    pub fn head_height(&self) -> f32 {
        Self::f(self.frame.height - self.frame.chin)
    }

    /// How wide the trunk's top is across, where the neck rises from it.
    pub fn base_across(&self) -> f32 {
        self.neck() * 1.6
    }

    /// On the trunk at this height, `off` out from it: its front half in front, its
    /// back half behind.
    pub fn trunk(&self, angle: f32, y: f32, off: f32) -> Vec3 {
        let girth = trunk_half(self.frame, y);
        let half = (girth.across, girth.deep(angle.cos() >= 0.0));
        on_ellipse(half, y, angle, off)
    }

    /// On the head at this height, `off` out from it: its section there is the
    /// ellipsoid's, never smaller than the neck.
    pub fn head(&self, angle: f32, y: f32, off: f32) -> Vec3 {
        let half_high = self.head_height() / 2.0;
        let centre = self.chin() + half_high;
        let shrink = (1.0 - ((y - centre) / half_high).powi(2)).max(0.0).sqrt();
        let (across, deep) = (Self::f(self.frame.head.0), Self::f(self.frame.head.1));
        let half = (
            (across * shrink).max(self.neck()),
            (deep * shrink).max(self.neck()),
        );
        on_ellipse(half, y, angle, off)
    }

    /// How high the body is at this distance out from the middle, at the side: the
    /// slope of the trunk from the neck to the shoulder, or the top of the upper arm.
    pub fn shoulder_top(&self, out: f32) -> f32 {
        let (base, half) = (self.base_across(), self.shoulder_half());
        let slope = if out <= base {
            self.neck_base()
        } else {
            let t = ((out - base) / (half - base)).min(1.0);
            self.neck_base() + (self.shoulder() - self.neck_base()) * t
        };
        let from_joint = out - half;
        let arm = self.upper_arm();
        let over_arm = if from_joint.abs() < arm {
            self.shoulder() + (arm * arm - from_joint * from_joint).sqrt()
        } else {
            f32::MIN
        };
        slope.max(over_arm)
    }

    /// The point moved out, if need be, to stand at least `off` clear of the neck, the
    /// trunk and the upper arms.
    pub fn clear(&self, point: Vec3, off: f32) -> Vec3 {
        let mut point = point;
        let (neck_low, neck_high) = (self.neck_base() - 20.0, self.chin() + 20.0);
        let across = Vec2::new(point.x, point.z);
        if point.y > neck_low && point.y < neck_high && across.length() < self.neck() + off {
            let out = across.normalize_or(Vec2::Y) * (self.neck() + off);
            point = Vec3::new(out.x, point.y, out.y);
        }
        if point.y <= self.neck_base() {
            let girth = trunk_half(self.frame, point.y);
            let (a, d) = (girth.across, girth.deep(point.z >= 0.0));
            let ellipse = Vec2::new(point.x / (a + off), point.z / (d + off)).length();
            if ellipse < 1.0 && ellipse > 0.0 {
                point = Vec3::new(point.x / ellipse, point.y, point.z / ellipse);
            }
        }
        for side in Side::BOTH {
            let joint = |joint: Joint| {
                let [x, y, z] = self.frame.joint(joint);
                Vec3::new(x as f32, y as f32, z as f32)
            };
            let (from, to) = (joint(Joint::Shoulder(side)), joint(Joint::Elbow(side)));
            let along =
                ((point - from).dot(to - from) / (to - from).length_squared()).clamp(0.0, 1.0);
            let nearest = from.lerp(to, along);
            let away = point - nearest;
            let reach = self.upper_arm() + off;
            if away.length() < reach {
                point = nearest + away.normalize_or(Vec3::Y) * reach;
            }
        }
        point
    }

    /// A point over the side of the shoulder, this far out from the middle in the
    /// direction of `angle`, `off` above the body.
    pub fn over_shoulder(&self, angle: f32, out: f32, off: f32) -> Vec3 {
        let y = self.shoulder_top(out * angle.sin().abs()) + off;
        dir(angle) * out + Vec3::Y * y
    }
}

/// How far a direction is to the side: 0 straight ahead or behind, 1 to the side.
pub(super) fn sideness(angle: f32) -> f32 {
    angle.sin().powi(2)
}

/// A curtain at one angle: from its top, through a middle point, to its bottom.
#[derive(Clone, Copy)]
pub(super) struct Profile {
    pub top: Vec3,
    pub middle: Vec3,
    pub bottom: Vec3,
}

impl Profile {
    /// Straight from top to bottom.
    pub fn straight(top: Vec3, bottom: Vec3) -> Self {
        Self {
            top,
            middle: (top + bottom) / 2.0,
            bottom,
        }
    }

    /// A point along it: a curve through the middle point halfway.
    pub fn at(&self, v: f32) -> Vec3 {
        let control = self.middle * 2.0 - (self.top + self.bottom) / 2.0;
        let w = 1.0 - v;
        self.top * w * w + control * 2.0 * v * w + self.bottom * v * v
    }

    /// The same, carried on past the bottom by `extra` millimetres, for hems.
    pub fn longer(self, extra: f32) -> Self {
        let tail = (self.bottom - self.at(0.9)).normalize_or_zero();
        let bottom = self.bottom + tail * extra;
        Self {
            middle: self.middle + tail * extra / 2.0,
            bottom,
            ..self
        }
    }
}

/// The way out from a surface at a point, worked out from its neighbours: away from the
/// middle of the neck.
pub(super) fn out_at(point: impl Fn(f32, f32) -> Vec3, u: f32, v: f32) -> Vec3 {
    const STEP: f32 = 0.01;
    let (u0, u1) = ((u - STEP).max(0.0), (u + STEP).min(1.0));
    let (v0, v1) = ((v - STEP).max(0.0), (v + STEP).min(1.0));
    let across = point(u1, v) - point(u0, v);
    let down = point(u, v1) - point(u, v0);
    let normal = across.cross(down).normalize_or_zero();
    let here = point(u, v);
    let away = Vec3::new(here.x, 0.0, here.z);
    if normal.dot(away) < 0.0 {
        -normal
    } else {
        normal
    }
}
