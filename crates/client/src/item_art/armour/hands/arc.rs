//! Curved plates: arcs around the hand, plain or bumped, and ellipses.

use bevy::prelude::*;
use std::f32::consts::PI;

use super::super::{Model, Stuff, plate};
use super::{ACROSS, Axes, ROUND, across};

/// Points round a line between two angles from its out side, `radii` out and forwards,
/// with `bump` adding to the radius by angle (flutes).
pub(super) struct Arc {
    pub(super) from: Vec3,
    pub(super) to: Vec3,
    /// Radius out and forwards at each end.
    pub(super) start: (f32, f32),
    pub(super) end: (f32, f32),
    pub(super) angles: (f32, f32),
    pub(super) rows: usize,
}

pub(super) fn arc_grid(
    arc: &Arc,
    axes: &Axes,
    bump: &dyn Fn(f32) -> f32,
) -> Vec<Vec<(Vec3, Vec3)>> {
    let along = (arc.to - arc.from).normalize();
    let (out, forward) = across(along, axes);
    (0..=arc.rows)
        .map(|row| {
            let t = row as f32 / arc.rows as f32;
            let centre = arc.from.lerp(arc.to, t);
            let radius_out = arc.start.0 + (arc.end.0 - arc.start.0) * t;
            let radius_forward = arc.start.1 + (arc.end.1 - arc.start.1) * t;
            (0..=ACROSS)
                .map(|column| {
                    let angle = arc.angles.0
                        + (arc.angles.1 - arc.angles.0) * column as f32 / ACROSS as f32;
                    let grow = bump(angle);
                    let point = centre
                        + out * angle.cos() * (radius_out + grow)
                        + forward * angle.sin() * (radius_forward + grow);
                    let normal = (out * angle.cos() / radius_out.max(1.0)
                        + forward * angle.sin() / radius_forward.max(1.0))
                    .normalize();
                    (point, normal)
                })
                .collect()
        })
        .collect()
}

pub(super) fn add_arc(model: &mut Model, stuff: Stuff, arc: &Arc, axes: &Axes, thick: f32) {
    add_arc_bumped(model, stuff, arc, axes, thick, &|_| 0.0);
}

pub(super) fn add_arc_bumped(
    model: &mut Model,
    stuff: Stuff,
    arc: &Arc,
    axes: &Axes,
    thick: f32,
    bump: &dyn Fn(f32) -> f32,
) {
    plate(model.of(stuff), &arc_grid(arc, axes, bump), thick, false);
}

/// A flat ring round the hand's middle line at a distance down from the wrist.
pub(super) fn ellipse(axes: &Axes, down: f32, half_out: f32, half_forward: f32) -> Vec<Vec3> {
    (0..ROUND)
        .map(|at| {
            let angle = at as f32 / ROUND as f32 * 2.0 * PI;
            axes.at(down, half_out * angle.cos(), half_forward * angle.sin())
        })
        .collect()
}
