//! A gate's frame: a slab round its opening, the opening's shape by design.
//!
//! The faces are filled between the opening and the slab's edge along rays
//! from the middle of the jambs' tops, which every design's top is shaped to
//! allow (each point further round than the last).

use bevy::prelude::*;
use shared::hall::GateDesign;

use super::profile::{SPRING, height, jamb, top};
use super::{DEPTH, GateModel, POST, RUNE_BAND, Rgb};

/// Above the opening, the band the runes are on.
pub(super) fn band(design: GateDesign) -> (f32, f32) {
    let from = height(design) + 0.04;
    (from, from + RUNE_BAND)
}

/// How far the frame reaches each way across, and how high.
pub(super) fn outline(design: GateDesign) -> (f32, f32) {
    (super::profile::HALF_OPENING + POST, band(design).1 + 0.04)
}

/// Where the ray from `(0, SPRING)` at `angle` leaves the frame's outline.
fn on_outline(angle: f32, half: f32, top: f32) -> Vec2 {
    let way = Vec2::new(angle.cos(), angle.sin());
    let to_side = if way.x.abs() > 1e-6 {
        half / way.x.abs()
    } else {
        f32::MAX
    };
    let to_top = if way.y > 1e-6 {
        (top - SPRING) / way.y
    } else {
        f32::MAX
    };
    Vec2::new(0.0, SPRING) + way * to_side.min(to_top)
}

/// The opening's top, with points added where the rays meet the outline's
/// corners, so the faces fill it exactly.
fn with_corners(points: Vec<Vec2>, half: f32, top: f32) -> Vec<Vec2> {
    let corners = [(top - SPRING).atan2(-half), (top - SPRING).atan2(half)];
    let angle = |p: Vec2| p.y.atan2(p.x);
    let mut out = vec![points[0]];
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        for corner in corners {
            let (from, to) = (angle(a), angle(b));
            if from > corner && corner > to {
                // Where the ray at that angle crosses the segment.
                let way = Vec2::new(corner.cos(), corner.sin());
                let t = way.perp_dot(a) / way.perp_dot(a - b);
                out.push(a.lerp(b, t));
            }
        }
        out.push(b);
    }
    out
}

pub(super) fn build(model: &mut GateModel, design: GateDesign, colour: Rgb) {
    let (half, top_y) = outline(design);
    let jamb = jamb(design);
    let opening: Vec<Vec2> = with_corners(top(design), half, top_y)
        .into_iter()
        .map(|p| p + Vec2::new(0.0, SPRING))
        .collect();
    let edge: Vec<Vec2> = opening
        .iter()
        .map(|p| on_outline((p.y - SPRING).atan2(p.x), half, top_y))
        .collect();
    let front = DEPTH / 2.0;
    let at = |p: Vec2, z: f32| Vec3::new(p.x, p.y, z);

    // The faces, front and back: the jambs below their tops, then round the top.
    let mut pairs = vec![(Vec2::new(-jamb, 0.0), Vec2::new(-half, 0.0))];
    pairs.extend(opening.iter().copied().zip(edge.iter().copied()));
    pairs.push((Vec2::new(jamb, 0.0), Vec2::new(half, 0.0)));
    for pair in pairs.windows(2) {
        let ((a, a_out), (b, b_out)) = (pair[0], pair[1]);
        for (z, normal) in [(front, Vec3::Z), (-front, -Vec3::Z)] {
            model.quad(
                [at(a, z), at(b, z), at(b_out, z), at(a_out, z)],
                normal,
                colour,
            );
        }
    }

    // Inside the opening, facing into it.
    let mut inner = vec![Vec2::new(-jamb, 0.0)];
    inner.extend(opening.iter().copied());
    inner.push(Vec2::new(jamb, 0.0));
    for pair in inner.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let along = b - a;
        if along.length_squared() < 1e-8 {
            continue;
        }
        // Left to right over the top, the opening is on the right of the way.
        let normal = Vec3::new(along.y, -along.x, 0.0).normalize();
        model.quad(
            [at(a, front), at(b, front), at(b, -front), at(a, -front)],
            normal,
            colour,
        );
    }

    // The outer sides and top.
    let corner = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
    let sides = [
        ([(-half, 0.0), (-half, top_y)], -Vec3::X),
        ([(-half, top_y), (half, top_y)], Vec3::Y),
        ([(half, top_y), (half, 0.0)], Vec3::X),
    ];
    for ([(x0, y0), (x1, y1)], normal) in sides {
        let quad = [
            corner(x0, y0, front),
            corner(x1, y1, front),
            corner(x1, y1, -front),
            corner(x0, y0, -front),
        ];
        model.quad(quad, normal, colour);
    }
}
