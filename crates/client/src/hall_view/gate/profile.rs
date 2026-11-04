//! The shape of a gate's opening, by its design: a polyline from the top of
//! the left jamb, over the top, to the top of the right jamb, in cells, with
//! the jambs' tops at `(±jamb, SPRING)`.

use bevy::prelude::*;
use shared::hall::GateDesign;
use std::f32::consts::PI;

/// Half the opening's width: the frame stands on its ledge, inside the tile
/// (owner).
pub(super) const HALF_OPENING: f32 = 0.45;
/// How high the jambs stand before the top's shape starts.
pub(super) const SPRING: f32 = 0.9;
/// Points along each curve.
const STEPS: usize = 16;

/// Half the width between a design's jambs: some narrow below their tops.
pub(super) fn jamb(design: GateDesign) -> f32 {
    let u = HALF_OPENING;
    match design {
        GateDesign::HorseshoeArch => 0.8 * u,
        GateDesign::Trefoil => 0.78 * u,
        GateDesign::Keyhole => 0.4 * u,
        _ => u,
    }
}

/// The top's points, left to right, relative to `(0, SPRING)`: each further
/// round from the left as seen from there, so the frame can be filled to
/// them.
pub(super) fn top(design: GateDesign) -> Vec<Vec2> {
    let u = HALF_OPENING;
    let arc = |centre: Vec2, radius: f32, from: f32, to: f32| -> Vec<Vec2> {
        (0..=STEPS)
            .map(|i| {
                let angle = from + (to - from) * i as f32 / STEPS as f32;
                centre + radius * Vec2::new(angle.cos(), angle.sin())
            })
            .collect()
    };
    match design {
        GateDesign::RoundArch => arc(Vec2::ZERO, u, PI, 0.0),
        GateDesign::SquareLintel => vec![
            Vec2::new(-u, 0.0),
            Vec2::new(-u, 0.3 * u),
            Vec2::new(u, 0.3 * u),
            Vec2::new(u, 0.0),
        ],
        GateDesign::PointedArch => {
            // Two arcs, each centred inside the other jamb.
            let radius = 1.6 * u;
            let apex = (radius * radius - (0.6 * u).powi(2)).sqrt();
            let left_to = apex.atan2(-0.6 * u);
            let mut points = arc(Vec2::new(0.6 * u, 0.0), radius, PI, left_to);
            let right = arc(Vec2::new(-0.6 * u, 0.0), radius, PI - left_to, 0.0);
            points.extend(right.into_iter().skip(1));
            points
        }
        GateDesign::HorseshoeArch => {
            let jamb = jamb(design);
            let rise = (u * u - jamb * jamb).sqrt();
            let start = (-rise).atan2(-jamb);
            arc(Vec2::new(0.0, rise), u, start + 2.0 * PI, -start - PI)
        }
        GateDesign::Ogee => {
            // Out, then in to a point: the jamb's line swings to the middle.
            let rise = 1.4 * u;
            let side = |sign: f32| {
                (0..=STEPS).map(move |i| {
                    let s = i as f32 / STEPS as f32;
                    Vec2::new(sign * u * (0.5 + 0.5 * (PI * s).cos()), rise * s)
                })
            };
            let mut points: Vec<Vec2> = side(-1.0).collect();
            points.extend(side(1.0).collect::<Vec<_>>().into_iter().rev().skip(1));
            points
        }
        GateDesign::Trefoil => {
            // Three lobes, meeting in points.
            let jamb = jamb(design);
            (0..=STEPS * 3)
                .map(|i| {
                    let angle = PI * (1.0 - i as f32 / (STEPS * 3) as f32);
                    let radius = jamb + (u - jamb) * (3.0 * angle).sin().abs() * 1.6;
                    radius * Vec2::new(angle.cos(), angle.sin())
                })
                .collect()
        }
        GateDesign::SteppedCorbel => {
            let (a, b, c) = (0.7 * u, 0.4 * u, 0.25 * u);
            vec![
                Vec2::new(-u, 0.0),
                Vec2::new(-u, c),
                Vec2::new(-a, c),
                Vec2::new(-a, 2.0 * c),
                Vec2::new(-b, 2.0 * c),
                Vec2::new(-b, 3.0 * c),
                Vec2::new(b, 3.0 * c),
                Vec2::new(b, 2.0 * c),
                Vec2::new(a, 2.0 * c),
                Vec2::new(a, c),
                Vec2::new(u, c),
                Vec2::new(u, 0.0),
            ]
        }
        GateDesign::Keyhole => {
            // A round head on a narrow slot.
            let jamb = jamb(design);
            let radius = 0.8 * u;
            let rise = (radius * radius - jamb * jamb).sqrt();
            let start = (-rise).atan2(-jamb);
            arc(Vec2::new(0.0, rise), radius, start + 2.0 * PI, -start - PI)
        }
    }
}

/// How high the opening reaches above the ground.
pub(super) fn height(design: GateDesign) -> f32 {
    SPRING + top(design).iter().map(|p| p.y).fold(0.0, f32::max)
}
