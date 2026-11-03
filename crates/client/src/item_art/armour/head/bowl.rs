//! The bowl over the skull, and what shows how it was built: a spangenhelm's bands,
//! a ridge, a kabuto's riveted plates, lamellar rows, rattan's weave, rivets for the
//! lining, a style's flutes and etching.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use shared::armour::head::{Bowl, HeadForm};
use shared::armour::{Finish, Style};
use shared::material::Material;

use super::shell::Shell;
use super::{THICK, sheet};
use crate::item_art::armour::surface::{lines, ring_grid, scales, studs};
use crate::item_art::armour::{Model, Stuff, plate};

/// Rows up and points round the bowl.
const ROWS: usize = 16;
const ROUND: usize = 48;

/// How far a style's flutes and keel raise the bowl here.
fn relief(style: Style, angle: f32, v: f32, point: Vec3) -> f32 {
    match style {
        Style::Maximilian if v < 0.9 => 2.5 * (angle * 28.0).cos().max(0.0),
        Style::Gothic => {
            let keel = if v > 0.3 {
                6.0 * (1.0 - point.x.abs() / 14.0).max(0.0)
            } else {
                0.0
            };
            keel + 2.5 * (angle * 14.0).cos().max(0.0) * v
        }
        _ => 0.0,
    }
}

/// The bowl and its marks.
pub(super) fn add_bowl(model: &mut Model, form: &HeadForm, shell: &Shell) {
    let surface = |u: f32, v: f32| {
        let (point, out) = shell.bowl(u, v);
        (point + out * relief(form.style, u * TAU, v, point), out)
    };
    plate(
        model.of(Stuff::Main),
        &ring_grid(&surface, ROWS, ROUND),
        THICK,
        true,
    );
    let on_bowl = |u: f32, v: f32| shell.bowl(u, v);
    match form.bowl {
        Bowl::Segmented => {
            for at in 0..4 {
                meridian(model, Stuff::Trim, shell, at as f32 * PI / 2.0, 0.07, 2.0);
            }
            brow_band(model, Stuff::Trim, shell, 22.0);
        }
        Bowl::Ridged => {
            for angle in [0.0, PI] {
                meridian(model, Stuff::Main, shell, angle, 0.05, 3.0);
            }
            brow_band(model, Stuff::Main, shell, 18.0);
        }
        Bowl::Plates => {
            for at in 0..24 {
                meridian(
                    model,
                    Stuff::Shade,
                    shell,
                    at as f32 * TAU / 24.0,
                    0.012,
                    0.8,
                );
            }
            brow_band(model, Stuff::Trim, shell, 14.0);
        }
        Bowl::Laced => {
            lines(model, &on_bowl, 7, 1.0);
            for at in 0..16 {
                meridian(
                    model,
                    Stuff::Cloth,
                    shell,
                    at as f32 * TAU / 16.0,
                    0.01,
                    1.2,
                );
            }
        }
        Bowl::Woven => {
            lines(model, &on_bowl, 10, 0.8);
            for at in 0..20 {
                meridian(
                    model,
                    Stuff::Shade,
                    shell,
                    at as f32 * TAU / 20.0,
                    0.012,
                    0.8,
                );
            }
        }
        Bowl::Moulded if form.tusks() => {
            let upper = |u: f32, v: f32| shell.bowl(u, 0.05 + v * 0.8);
            scales(model, Stuff::Bone, &upper, 4, 9);
        }
        _ => {}
    }
    if form.studded {
        let upper = |u: f32, v: f32| shell.bowl(u, 0.1 + v * 0.6);
        studs(model, Stuff::Rivet, &upper, (3, 14), 2.0, 3.0);
    }
    let bronze = matches!(form.material, Material::Bronze | Material::Copper);
    if form.bowl == Bowl::Raised && !bronze {
        // The lining's rivets, just above the edge.
        let rim = |u: f32, v: f32| shell.bowl(u, 0.04 + v * 0.01);
        studs(model, Stuff::Rivet, &rim, (1, 18), 1.0, 2.5);
    }
    match form.style {
        Style::Dwarven => brow_band(model, Stuff::Trim, shell, 26.0),
        Style::Elven => brow_band(model, Stuff::Trim, shell, 6.0),
        _ => {}
    }
    if form.finish == Finish::Etched {
        for at in 0..6 {
            meridian(model, Stuff::Etch, shell, at as f32 * TAU / 6.0, 0.08, 0.6);
        }
    }
}

/// A strip from the edge up over the bowl at this angle, `half` either side.
pub(super) fn meridian(
    model: &mut Model,
    stuff: Stuff,
    shell: &Shell,
    angle: f32,
    half: f32,
    lift: f32,
) {
    let centre = angle / TAU;
    let surface = |u: f32, v: f32| {
        let (point, out) = shell.bowl(centre + (u * 2.0 - 1.0) * half / TAU, v * 0.97);
        (point + out * lift, out)
    };
    plate(
        model.of(stuff),
        &crate::item_art::armour::surface::grid(&surface, ROWS, 2),
        1.0,
        false,
    );
}

/// A band right round just above the edge, this tall.
pub(super) fn brow_band(model: &mut Model, stuff: Stuff, shell: &Shell, tall: f32) {
    let surface = |u: f32, v: f32| {
        let angle = u * TAU;
        let (point, out) = shell.point(angle, shell.edge_at(angle) + tall * v);
        (point + out * 1.5, out)
    };
    plate(model.of(stuff), &ring_grid(&surface, 1, ROUND), 1.5, true);
}

/// A plate on part of the bowl, for parts that only need a rough patch.
pub(super) fn bowl_patch(
    model: &mut Model,
    stuff: Stuff,
    shell: &Shell,
    angles: (f32, f32),
    heights: (f32, f32),
    lift: f32,
) {
    sheet(
        model,
        stuff,
        shell,
        angles,
        heights,
        &|_, _| lift,
        THICK,
        (2, 8),
    );
}
