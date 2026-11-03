//! The glove under the plates, and rivets.

use bevy::prelude::*;

use super::super::{Model, Stuff};
use super::{Axes, Shape, ellipse};
use crate::item_art::body::{limb, loft};

/// A glove under everything, or the whole piece for mail, leather and padding:
/// palm, fingers (or one bag for them) and thumb. `bands` shades every other strip of
/// that many millimetres, as rows of mail or quilting.
pub(super) fn add_glove(
    model: &mut Model,
    shape: &Shape,
    axes: &Axes,
    mitten: bool,
    swell: f32,
    stuff: Stuff,
    bands: Option<f32>,
) {
    let mut rings = vec![
        (-6.0, shape.thick * 0.5, shape.breadth * 0.38),
        (shape.palm * 0.5, shape.thick / 2.0, shape.breadth * 0.47),
        (shape.palm + 4.0, shape.thick * 0.45, shape.breadth / 2.0),
    ];
    if mitten {
        let end = shape.palm + shape.reach() * 0.85;
        rings.extend([
            (end - 20.0, shape.thick * 0.38, shape.breadth / 2.0),
            (end - 6.0, shape.thick * 0.3, shape.breadth * 0.42),
            (end, shape.thick * 0.1, shape.breadth * 0.2),
        ]);
    }
    let rings: Vec<(f32, f32, f32)> = rings
        .into_iter()
        .map(|(down, out, forward)| (down, out + swell, forward + swell))
        .collect();
    let lines = if mitten {
        vec![shape.thumb_line]
    } else {
        let mut lines = shape.fingers.clone();
        lines.push(shape.thumb_line);
        lines
    };
    let band = bands.unwrap_or(f32::MAX);
    // The palm, in strips.
    let mut strip = 0;
    for pair in rings.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let steps = ((to.0 - from.0) / band).ceil().max(1.0) as usize;
        for step in 0..steps {
            let lerp = |t: f32| {
                let at = |a: f32, b: f32| a + (b - a) * t;
                ellipse(axes, at(from.0, to.0), at(from.1, to.1), at(from.2, to.2))
            };
            let (a, b) = (step as f32 / steps as f32, (step + 1) as f32 / steps as f32);
            let stuff = if strip % 2 == 1 { Stuff::Shade } else { stuff };
            loft(model.of(stuff), &[lerp(a), lerp(b)]);
            strip += 1;
        }
    }
    // Fingers and thumb, likewise.
    for (at, &(root, tip)) in lines.iter().enumerate() {
        let thumb = at == lines.len() - 1;
        let radius = shape.finger * if thumb { 1.15 } else { 1.0 } + swell;
        let length = root.distance(tip);
        let steps = (length / band).ceil().max(1.0) as usize;
        for step in 0..steps {
            let (a, b) = (step as f32 / steps as f32, (step + 1) as f32 / steps as f32);
            let narrow = |t: f32| radius * (1.0 - 0.15 * t);
            let stuff = if step % 2 == 1 { Stuff::Shade } else { stuff };
            limb(
                model.of(stuff),
                &[
                    (root.lerp(tip, a), narrow(a)),
                    (root.lerp(tip, b), narrow(b)),
                ],
            );
        }
    }
}

/// Small round heads in rows over the back of the hand.
pub(super) fn add_rivets(model: &mut Model, shape: &Shape, axes: &Axes, out: f32, stuff: Stuff) {
    for row in 0..3 {
        let down = 14.0 + row as f32 * (shape.palm - 24.0) / 2.0;
        for column in -1..=1 {
            let angle = column as f32 * 0.7;
            let at = axes.at(
                down,
                (shape.thick / 2.0 + out) * angle.cos(),
                shape.breadth * 0.45 * angle.sin(),
            );
            let way = (axes.out * angle.cos() + axes.forward * angle.sin()).normalize();
            limb(
                model.of(stuff),
                &[(at - way * 1.0, 3.0), (at + way * 1.5, 2.0)],
            );
        }
    }
}
