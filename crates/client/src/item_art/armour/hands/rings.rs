//! Ring mail: flat rings sewn in rows on a leather glove, over the back of the hand and
//! down the fingers, as the other locations' ring mail is drawn (round studs).

use bevy::prelude::*;

use super::super::{Model, Stuff};
use super::{Axes, Shape, across};
use crate::item_art::body::limb;

/// Between rings, down and across.
const STEP: f32 = 8.0;
const RADIUS: f32 = 2.8;

/// One flat ring at a point, facing `way`.
fn ring(model: &mut Model, at: Vec3, way: Vec3) {
    limb(
        model.of(Stuff::Main),
        &[(at - way * 0.6, RADIUS), (at + way * 0.6, RADIUS)],
    );
}

/// Rows of rings over the back of the hand and the fingers (or the mitten's bag), on a
/// glove swollen by `swell`.
pub(super) fn add_rings(model: &mut Model, shape: &Shape, axes: &Axes, mitten: bool, swell: f32) {
    let end = if mitten {
        shape.palm + shape.reach() * 0.8
    } else {
        shape.palm
    };
    let rows = (end / STEP) as usize;
    for row in 0..rows {
        let down = 4.0 + row as f32 * STEP;
        // A mitten's bag is flatter than the palm.
        let deep = if down > shape.palm { 0.38 } else { 0.5 };
        let out = shape.thick * deep + swell + 1.0;
        let half = shape.breadth * 0.5;
        let columns = (half * 2.0 / STEP) as i32;
        for column in -columns / 2..=columns / 2 {
            let angle = column as f32 * STEP / half + if row % 2 == 1 { 0.12 } else { 0.0 };
            if angle.abs() > 1.3 {
                continue;
            }
            let way = (axes.out * angle.cos() + axes.forward * angle.sin()).normalize();
            let at = axes.at(down, out * angle.cos(), (half + swell) * angle.sin());
            ring(model, at, way);
        }
    }
    let mut lines = if mitten {
        Vec::new()
    } else {
        shape.fingers.clone()
    };
    lines.push(shape.thumb_line);
    for (root, tip) in lines {
        let along = (tip - root).normalize();
        let (out, forward) = across(along, axes);
        let count = (root.distance(tip) / STEP) as usize;
        let radius = shape.finger + swell + 1.0;
        for at in 0..count {
            let centre = root + along * (4.0 + at as f32 * STEP);
            for angle in [-0.9_f32, 0.0, 0.9] {
                let way = out * angle.cos() + forward * angle.sin();
                ring(model, centre + way * radius, way);
            }
        }
    }
}
