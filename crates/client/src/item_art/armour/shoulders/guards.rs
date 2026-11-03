//! Shoulder guards.

use shared::armour::Construction;
use shared::armour::shoulders::{Finish, Lacing, ShouldersForm, ShouldersPiece};
use shared::body::Frame;

use super::super::surface::{bands, lines, patch, scales, studs};
use super::super::{Model, Stuff};
use super::{Arm, guard_surface, sheet};

pub(super) fn add_guards(model: &mut Model, form: &ShouldersForm, frame: &Frame, arm: &Arm) {
    let ShouldersPiece::Guards {
        size,
        shape,
        rows,
        lacing,
    } = form.piece
    else {
        return;
    };
    let japanese = form.japanese();
    let scale = frame.elbow_length as f32 / 300.0;
    let (wide, high) = size.guard();
    let size = (wide as f32 * scale, high as f32 * scale);
    let surface = guard_surface(arm, frame, size, shape);
    let rows = rows as usize;
    // A Japanese guard's solid top plate, a little flared.
    let body = |u: f32, v: f32| surface(u, if japanese { 0.1 + 0.9 * v } else { v });
    if japanese {
        let top = |u: f32, v: f32| {
            let (point, out) = surface(u, v * 0.1);
            (point + out * (6.0 - v * 4.0), out)
        };
        sheet(model, Stuff::Main, &top, 3.0);
    }
    match form.construction {
        Construction::Lamellar | Construction::Laminar => {
            bands(model, Stuff::Main, &body, rows, 1.8, 2.0);
            let (count, width) = match (lacing, form.construction) {
                (Lacing::Dense, Construction::Lamellar) => (14, 0.035),
                (Lacing::Dense, _) => (9, 0.04),
                (Lacing::Spaced, _) => (5, 0.025),
            };
            for column in 0..count {
                let u = (column as f32 + 0.5) / count as f32;
                let pair = if lacing == Lacing::Spaced {
                    [-0.02, 0.02]
                } else {
                    [0.0, 0.0]
                };
                for shift in pair
                    .into_iter()
                    .take(if lacing == Lacing::Spaced { 2 } else { 1 })
                {
                    let u = u + shift;
                    patch(
                        model,
                        Stuff::Cloth,
                        &body,
                        (u - width / 2.0, u + width / 2.0),
                        (0.0, 1.0),
                        rows as f32 * 1.8 + 1.6,
                        0.8,
                    );
                }
            }
            if japanese {
                // The bottom row's cross-knots in a band of their own.
                patch(
                    model,
                    Stuff::Cloth,
                    &body,
                    (0.0, 1.0),
                    (0.93, 1.0),
                    2.6,
                    1.0,
                );
            }
        }
        Construction::Scale => {
            sheet(model, Stuff::Backing, &body, 1.5);
            scales(model, Stuff::Main, &body, rows * 2, 12);
        }
        Construction::PlatesInCloth => {
            sheet(model, Stuff::Cloth, &body, 3.0);
            studs(model, Stuff::Rivet, &body, (rows, 8), 1.5, 3.0);
        }
        Construction::Mail => {
            sheet(model, Stuff::Cloth, &body, 1.5);
            patch(
                model,
                Stuff::Main,
                &body,
                (0.08, 0.92),
                (0.05, 0.95),
                1.5,
                1.2,
            );
            lines(
                model,
                &|u, v| body(0.08 + 0.84 * u, 0.05 + 0.9 * v),
                rows * 4,
                2.4,
            );
        }
        _ => {
            bands(model, Stuff::Main, &body, rows, 2.0, 3.5);
            match form.finish {
                Finish::Studded => studs(
                    model,
                    Stuff::Rivet,
                    &body,
                    (rows, 7),
                    rows as f32 * 2.0 + 1.8,
                    3.0,
                ),
                Finish::RingMail => lines(model, &body, rows * 3, rows as f32 * 2.0 + 2.0),
                Finish::Tooled => lines(model, &body, rows * 2, rows as f32 * 2.0 + 1.9),
                Finish::Plain => {}
            }
        }
    }
}
