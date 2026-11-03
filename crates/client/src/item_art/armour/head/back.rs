//! What guards the neck behind (flares, guards, a sallet's tail, lames, hanging flaps)
//! and the brims and peaks that stand out from the edge.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use shared::armour::Style;
use shared::armour::head::{Back, Brim, HeadForm};

use super::shell::Shell;
use super::{THICK, sheet};
use crate::item_art::armour::surface::{lines, ring_grid};
use crate::item_art::armour::{Model, Stuff, plate};

/// Points round a brim.
const ROUND: usize = 48;

/// A strip hanging below the back of the edge, `half` either side of the back, from
/// `drop` below it up to `rise` above, standing out by `lift`.
#[allow(clippy::too_many_arguments)]
fn below(
    model: &mut Model,
    stuff: Stuff,
    shell: &Shell,
    half: f32,
    (drop, rise): (f32, f32),
    lift: &dyn Fn(f32) -> f32,
    rows: usize,
) {
    let surface = |u: f32, v: f32| {
        let angle = PI + (u * 2.0 - 1.0) * half;
        let edge = shell.edge_at(angle);
        let (point, out) = shell.point(angle, edge - drop + (drop + rise) * v);
        (point + out * lift(v), out)
    };
    plate(
        model.of(stuff),
        &crate::item_art::armour::surface::grid(&surface, rows, 24),
        THICK,
        false,
    );
}

pub(super) fn add_back(model: &mut Model, form: &HeadForm, shell: &Shell) {
    match form.back {
        Back::None => {}
        Back::Flare => below(
            model,
            Stuff::Main,
            shell,
            1.4,
            (22.0, 4.0),
            &|v| 15.0 * (1.0 - v),
            2,
        ),
        Back::Guard => below(
            model,
            Stuff::Main,
            shell,
            1.3,
            (45.0, 4.0),
            &|v| 8.0 + 45.0 * (1.0 - v),
            3,
        ),
        Back::Tail => below(
            model,
            Stuff::Main,
            shell,
            1.7,
            (110.0, 4.0),
            &|v| 6.0 + 70.0 * (1.0 - v).powf(1.3),
            6,
        ),
        Back::Lames => {
            let japanese = form.style == Style::Japanese;
            let half = if japanese { 2.4 } else { 1.5 };
            let count = if japanese { 5 } else { 4 };
            for lame in 0..count {
                let top = lame as f32 * 26.0;
                let out = 3.0 + lame as f32 * if japanese { 11.0 } else { 7.0 };
                let lift = move |v: f32| out + 6.0 * (1.0 - v);
                below(
                    model,
                    Stuff::Main,
                    shell,
                    half,
                    (top + 30.0, -top + 2.0),
                    &lift,
                    2,
                );
                if japanese {
                    // Lacing down each lame.
                    below(
                        model,
                        Stuff::Cloth,
                        shell,
                        half * 0.98,
                        (top + 28.0, -top - 2.0),
                        &move |v| lift(v) + 1.5,
                        1,
                    );
                }
            }
            if japanese {
                // The fukigaeshi: the top lame turned back at the face.
                for side in [-1.0, 1.0] {
                    let angle = side * (PI - half) - side * 0.05;
                    let edge = shell.edge_at(angle);
                    sheet(
                        model,
                        Stuff::Main,
                        shell,
                        (angle - 0.25, angle + 0.25),
                        (edge - 30.0, edge + 10.0),
                        &|u, _| 8.0 + 30.0 * (if side > 0.0 { 1.0 - u } else { u }),
                        THICK,
                        (2, 6),
                    );
                }
            }
        }
        Back::Flaps => {
            let reach = shell.frame.shoulder as f32 + 30.0;
            for (centre, half) in [(PI, 0.75), (1.35, 0.4), (-1.35, 0.4)] {
                let surface = |u: f32, v: f32| {
                    let angle = centre + (u * 2.0 - 1.0) * half;
                    let edge = shell.edge_at(angle) + 4.0;
                    let (point, out) = shell.point(angle, reach + (edge - reach) * v);
                    (point + out * (5.0 + 40.0 * (1.0 - v)), out)
                };
                plate(
                    model.of(Stuff::Main),
                    &crate::item_art::armour::surface::grid(&surface, 4, 10),
                    4.0,
                    false,
                );
                lines(model, &surface, 5, 2.5);
            }
        }
    }
}

pub(super) fn add_brim(model: &mut Model, form: &HeadForm, shell: &Shell) {
    if form.brim == Brim::Peak {
        let surface = |u: f32, v: f32| {
            let angle = (u * 2.0 - 1.0) * 1.1;
            let edge = shell.edge_at(angle);
            let (point, out) = shell.point(angle, edge);
            let level = Vec3::new(out.x, 0.0, out.z).normalize_or(Vec3::Z);
            let reach = 50.0 * angle.cos().max(0.0).sqrt() * v;
            (point + level * reach - Vec3::Y * 8.0 * v, Vec3::Y)
        };
        plate(
            model.of(Stuff::Main),
            &crate::item_art::armour::surface::grid(&surface, 2, 20),
            THICK,
            false,
        );
        return;
    }
    if !form.brim.round() {
        return;
    }
    let japanese = form.style == Style::Japanese;
    let (wide, slope) = match form.brim {
        Brim::Narrow => (30.0, 0.15),
        Brim::Wide if japanese => (140.0, 0.5),
        Brim::Wide => (110.0, 0.35),
        _ => (60.0, 0.0),
    };
    let boat = form.brim == Brim::Boat;
    let surface = |u: f32, v: f32| {
        let angle = u * TAU;
        let (point, out) = shell.point(angle, shell.edge_at(angle));
        let level = Vec3::new(out.x, 0.0, out.z).normalize_or(Vec3::Z);
        let fore_aft = angle.cos().abs();
        let reach = (wide + if boat { 25.0 * fore_aft } else { 0.0 }) * v;
        let up = if boat {
            120.0 * fore_aft.powi(3) * v
        } else {
            0.0
        };
        (
            point + level * reach + Vec3::Y * (up - slope * reach),
            Vec3::Y,
        )
    };
    plate(
        model.of(Stuff::Main),
        &ring_grid(&surface, 3, ROUND),
        THICK,
        true,
    );
}
