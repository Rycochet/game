//! Surfaces armour is built over, and what goes on them: plates over part of one,
//! bands, scales, lines of shade and studs. Shared by every location's models.

use bevy::prelude::*;

use super::{Model, Stuff, plate};
use crate::item_art::body::blob;

/// Points across a patch, at most.
const ACROSS: usize = 16;

/// A point on a surface and the way out from it, by two numbers from 0 to 1.
pub(in crate::item_art) type Surface<'a> = &'a dyn Fn(f32, f32) -> (Vec3, Vec3);

/// Rows of points over a surface: `rows` + 1 down it, `columns` + 1 across.
pub(in crate::item_art) fn grid(
    surface: Surface,
    rows: usize,
    columns: usize,
) -> Vec<Vec<(Vec3, Vec3)>> {
    (0..=rows)
        .map(|row| {
            (0..=columns)
                .map(|column| surface(column as f32 / columns as f32, row as f32 / rows as f32))
                .collect()
        })
        .collect()
}

/// The same, going right round, so the last column meets the first.
pub(in crate::item_art) fn ring_grid(
    surface: Surface,
    rows: usize,
    columns: usize,
) -> Vec<Vec<(Vec3, Vec3)>> {
    (0..=rows)
        .map(|row| {
            (0..columns)
                .map(|column| surface(column as f32 / columns as f32, row as f32 / rows as f32))
                .collect()
        })
        .collect()
}

/// A plate over part of a surface, lifted off it by `lift`.
pub(in crate::item_art) fn patch(
    model: &mut Model,
    stuff: Stuff,
    surface: Surface,
    (u0, u1): (f32, f32),
    (v0, v1): (f32, f32),
    lift: f32,
    thick: f32,
) {
    let part = |u: f32, v: f32| {
        let (point, out) = surface(u0 + (u1 - u0) * u, v0 + (v1 - v0) * v);
        (point + out * lift, out)
    };
    let columns = (((u1 - u0) * ACROSS as f32).ceil() as usize).max(2);
    plate(model.of(stuff), &grid(&part, 2, columns), thick, false);
}

/// Horizontal bands down a surface, each standing out over the one below so its edge
/// shows: lames, lamellar rows, planks.
pub(in crate::item_art) fn bands(
    model: &mut Model,
    stuff: Stuff,
    surface: Surface,
    count: usize,
    step: f32,
    thick: f32,
) {
    for band in 0..count {
        let v0 = band as f32 / count as f32;
        let v1 = ((band + 1) as f32 / count as f32 + 0.12).min(1.0);
        let lift = (count - band) as f32 * step;
        patch(model, stuff, surface, (0.0, 1.0), (v0, v1), lift, thick);
    }
}

/// Overlapping scales over a surface, each row over the top of the one below.
pub(in crate::item_art) fn scales(
    model: &mut Model,
    stuff: Stuff,
    surface: Surface,
    rows: usize,
    columns: usize,
) {
    for row in 0..rows {
        let offset = if row % 2 == 0 { 0.0 } else { 0.5 };
        let v0 = row as f32 / rows as f32;
        let v1 = ((row as f32 + 1.5) / rows as f32).min(1.0);
        let lift = 1.5 + (rows - row) as f32 * 0.9;
        for column in 0..=columns {
            let centre = (column as f32 + offset) / columns as f32;
            let half = 0.55 / columns as f32;
            if centre - half < 0.0 || centre + half > 1.0 {
                continue;
            }
            // Rounded at the bottom.
            let scale = |u: f32, v: f32| {
                let round = (1.0 - ((v - 0.45).max(0.0) / 0.55).powi(2))
                    .max(0.05)
                    .sqrt();
                let (point, out) =
                    surface(centre + (u * 2.0 - 1.0) * half * round, v0 + (v1 - v0) * v);
                (point + out * (lift + v * 1.5), out)
            };
            plate(model.of(stuff), &grid(&scale, 3, 3), 1.2, false);
        }
    }
}

/// Thin darker lines across a surface: rows of mail or rings, or quilting.
pub(in crate::item_art) fn lines(model: &mut Model, surface: Surface, count: usize, lift: f32) {
    for line in 1..count {
        let v = line as f32 / count as f32;
        patch(
            model,
            Stuff::Shade,
            surface,
            (0.0, 1.0),
            (v - 0.012, v + 0.012),
            lift,
            0.6,
        );
    }
}

/// Small round heads over a surface: rivets and studs.
pub(in crate::item_art) fn studs(
    model: &mut Model,
    stuff: Stuff,
    surface: Surface,
    (rows, columns): (usize, usize),
    lift: f32,
    radius: f32,
) {
    for row in 0..rows {
        for column in 0..columns {
            let u = (column as f32 + 0.5) / columns as f32;
            let v = (row as f32 + 0.5) / rows as f32;
            let (point, out) = surface(u, v);
            blob(model.of(stuff), point + out * lift, Vec3::splat(radius));
        }
    }
}
