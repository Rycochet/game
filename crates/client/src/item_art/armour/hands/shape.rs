//! The hand's axes and outline, which every part is built along.

use bevy::prelude::*;
use shared::armour::hands::Hand;

/// A hand's own directions: down to the fingertips, out of the back of the hand, and
/// forwards to the thumb's side.
pub(super) struct Axes {
    pub(super) wrist: Vec3,
    pub(super) down: Vec3,
    pub(super) out: Vec3,
    pub(super) forward: Vec3,
}

impl Axes {
    pub(super) fn of(hand: &Hand) -> Self {
        let point = |mm: [i32; 3]| Vec3::new(mm[0] as f32, mm[1] as f32, mm[2] as f32);
        let wrist = point(hand.wrist);
        let down = (point(hand.tip) - wrist).normalize();
        let side = Vec3::X * hand.side.sign() as f32;
        let out = (side - down * side.dot(down)).normalize();
        let forward = (Vec3::Z - down * Vec3::Z.dot(down) - out * Vec3::Z.dot(out)).normalize();
        Self {
            wrist,
            down,
            out,
            forward,
        }
    }

    /// So far down from the wrist, out from the hand's middle, and forwards.
    pub(super) fn at(&self, down: f32, out: f32, forward: f32) -> Vec3 {
        self.wrist + self.down * down + self.out * out + self.forward * forward
    }
}

/// A hand's sizes as numbers to build with, and where its fingers are.
pub(super) struct Shape {
    length: f32,
    pub(super) palm: f32,
    pub(super) breadth: f32,
    pub(super) thick: f32,
    pub(super) finger: f32,
    /// Each finger's line, knuckle to tip, from the thumb's side.
    pub(super) fingers: Vec<(Vec3, Vec3)>,
    /// The thumb's line, root to tip.
    pub(super) thumb_line: (Vec3, Vec3),
}

impl Shape {
    pub(super) fn of(hand: &Hand, axes: &Axes) -> Self {
        let f = |mm: i32| mm as f32;
        let (length, palm, breadth, thick) = (
            f(hand.length),
            f(hand.palm),
            f(hand.breadth),
            f(hand.thickness),
        );
        let count = hand.fingers.max(1) as usize;
        let finger = breadth / (2.0 * count as f32) * 0.85;
        let reach = length - palm;
        let fingers = (0..count)
            .map(|at| {
                let spread = if count == 1 {
                    0.0
                } else {
                    (breadth / 2.0 - finger * 1.1) * (1.0 - 2.0 * at as f32 / (count - 1) as f32)
                };
                // The middle finger longest, the little one shortest.
                let middle = (count - 1) as f32 * 0.33;
                let span = (count - 1).max(1) as f32;
                let long = 1.0 - 0.32 * (at as f32 - middle).abs() / span;
                (
                    axes.at(palm, 0.0, spread),
                    axes.at(palm + reach * long - finger * 0.5, 0.0, spread * 0.92),
                )
            })
            .collect();
        let thumb_from = f(hand.thumb_from);
        let root = axes.at(thumb_from + 12.0, -thick * 0.15, breadth / 2.0 - 4.0);
        let way = (axes.down * 0.8 + axes.forward * 0.45 - axes.out * 0.25).normalize();
        Self {
            length,
            palm,
            breadth,
            thick,
            finger,
            fingers,
            thumb_line: (root, root + way * f(hand.thumb)),
        }
    }

    /// How long the longest finger is, from the knuckle.
    pub(super) fn reach(&self) -> f32 {
        self.length - self.palm
    }
}

/// One way across a line: the out and forward directions square to it.
pub(super) fn across(along: Vec3, axes: &Axes) -> (Vec3, Vec3) {
    let out = (axes.out - along * axes.out.dot(along)).normalize();
    (
        out,
        along.cross(out).normalize() * along.cross(out).dot(axes.forward).signum(),
    )
}
