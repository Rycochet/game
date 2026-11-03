//! Claws, knuckles and other extras; tekko and scale backs.

use bevy::prelude::*;
use shared::armour::hands::{HandsForm, Knuckles};
use shared::armour::{Construction, Style};
use std::f32::consts::PI;

use super::super::{Model, Stuff};
use super::{Arc, Axes, Shape, across, add_arc};
use crate::item_art::body::limb;

/// Spikes or studs on the knuckles, a bar, a disc, claws.
pub(super) fn add_extras(model: &mut Model, form: &HandsForm, shape: &Shape, axes: &Axes) {
    let out = shape.thick / 2.0 + 9.0;
    let knuckle_lines: Vec<Vec3> = shape.fingers.iter().map(|&(knuckle, _)| knuckle).collect();
    match form.knuckles {
        Knuckles::Plain => {}
        Knuckles::Spiked if form.construction == Construction::Leather => {
            for &knuckle in &knuckle_lines {
                let at = knuckle + axes.out * (shape.thick / 2.0 + 2.0);
                limb(
                    model.of(Stuff::Rivet),
                    &[(at, 4.0), (at + axes.out * 3.0, 2.5)],
                );
            }
        }
        Knuckles::Spiked => {
            for &knuckle in &knuckle_lines {
                let at = knuckle + axes.out * out;
                limb(
                    model.of(Stuff::Main),
                    &[(at, 4.5), (at + axes.out * 18.0, 0.6)],
                );
            }
        }
        Knuckles::Ridged => {
            let bar = if form.style == Style::Dwarven {
                6.0
            } else {
                3.5
            };
            let half = shape.breadth / 2.0;
            limb(
                model.of(Stuff::Main),
                &[
                    (axes.at(shape.palm - 2.0, out + 1.0, half), bar),
                    (axes.at(shape.palm - 2.0, out + 2.0, 0.0), bar),
                    (axes.at(shape.palm - 2.0, out + 1.0, -half), bar),
                ],
            );
        }
        Knuckles::Rondel => {
            let centre = axes.at(shape.palm * 0.5, shape.thick / 2.0 + 8.0, 0.0);
            limb(
                model.of(Stuff::Main),
                &[(centre, 24.0), (centre + axes.out * 3.0, 24.0)],
            );
        }
    }
    if form.claws {
        for &(_, tip) in &shape.fingers {
            let along = (tip - shape.fingers[0].0).normalize();
            let r = shape.finger * 0.55;
            limb(
                model.of(Stuff::Main),
                &[
                    (tip, r),
                    (tip + along * 20.0 - axes.out * 6.0, r * 0.6),
                    (tip + along * 32.0 - axes.out * 16.0, 0.5),
                ],
            );
        }
    }
}

/// A tekkō: a plate over the back of the hand on cloth, tied on by a loop round the
/// middle finger and a band round the wrist.
pub(super) fn add_tekko(model: &mut Model, shape: &Shape, axes: &Axes) {
    let base = Arc {
        from: axes.at(-8.0, 0.0, 0.0),
        to: axes.at(shape.palm + 8.0, 0.0, 0.0),
        start: (shape.thick / 2.0 + 2.0, shape.breadth * 0.38),
        end: (shape.thick / 2.0 + 2.0, shape.breadth / 2.0),
        angles: (-1.4, 1.4),
        rows: 4,
    };
    add_arc(model, Stuff::Cloth, &base, axes, 2.0);
    let top = Arc {
        from: axes.at(6.0, 0.0, 0.0),
        to: axes.at(shape.palm - 4.0, 0.0, 0.0),
        start: (shape.thick / 2.0 + 5.0, shape.breadth * 0.3),
        end: (shape.thick / 2.0 + 5.0, shape.breadth * 0.42),
        angles: (-1.0, 1.0),
        rows: 4,
    };
    add_arc(model, Stuff::Main, &top, axes, 2.0);
    // The loop, round the middle finger.
    let middle = shape.fingers[shape.fingers.len() / 3];
    let along = (middle.1 - middle.0).normalize();
    let (out, forward) = across(along, axes);
    let centre = middle.0 + along * 18.0;
    let mut path: Vec<(Vec3, f32)> = (0..12)
        .map(|at| {
            let angle = at as f32 / 12.0 * 2.0 * PI;
            (
                centre + (out * angle.cos() + forward * angle.sin()) * (shape.finger + 3.0),
                2.5,
            )
        })
        .collect();
    path.push(path[0]);
    limb(model.of(Stuff::Cloth), &path);
}

/// Rows of small plates over the back of the hand and down each finger.
pub(super) fn add_scales(model: &mut Model, shape: &Shape, axes: &Axes) {
    let rows = ((shape.palm + 8.0) / 11.0) as usize;
    for row in 0..rows {
        let down = 2.0 + row as f32 * 11.0;
        let lift = (rows - row) as f32 * 0.5;
        for column in -4..4 {
            let angle = column as f32 * 0.32 + if row % 2 == 1 { 0.16 } else { 0.0 };
            let arc = Arc {
                from: axes.at(down, 0.0, 0.0),
                to: axes.at(down + 15.0, 0.0, 0.0),
                start: (
                    shape.thick / 2.0 + 3.0 + lift,
                    shape.breadth / 2.0 + 3.0 + lift,
                ),
                end: (
                    shape.thick / 2.0 + 3.0 + lift,
                    shape.breadth / 2.0 + 3.0 + lift,
                ),
                angles: (angle - 0.2, angle + 0.2),
                rows: 1,
            };
            add_arc(model, Stuff::Main, &arc, axes, 1.5);
        }
    }
    let mut lines = shape.fingers.clone();
    lines.push(shape.thumb_line);
    for (root, tip) in lines {
        let length = root.distance(tip);
        let count = (length / 10.0) as usize;
        for at in 0..count {
            let from = root.lerp(tip, at as f32 / count as f32);
            let to = from + (tip - root).normalize() * 14.0;
            let r = shape.finger + 2.5 + (count - at) as f32 * 0.3;
            let arc = Arc {
                from,
                to,
                start: (r, r),
                end: (r, r),
                angles: (-1.4, 1.4),
                rows: 1,
            };
            add_arc(model, Stuff::Main, &arc, axes, 1.5);
        }
    }
}
