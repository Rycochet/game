//! Plate cuirasses and breastplates (and hardened leather made the same way): breast
//! and back over the shell, plackart, ridge and flutes, straps over the shoulders,
//! discs, tankō bands, the lance rest, and edges of brass.

use std::f32::consts::TAU;

use bevy::prelude::*;
use shared::armour::Finish;
use shared::armour::torso::{Breast, Fluting, Plackart, TorsoBody, TorsoForm};
use shared::body::Frame;

use super::shell::{Shell, Span};
use super::{RIGID_OUT, ROWS};
use crate::item_art::armour::surface::{Surface, grid, lines, patch, studs};
use crate::item_art::armour::{Model, Stuff, plate};
use crate::item_art::body::limb;

/// Points round a breast: enough for its curves and flutes to show.
const ACROSS: usize = 36;

/// A plate over a shell, `thick` millimetres.
pub(super) fn sheet(model: &mut Model, stuff: Stuff, shell: &Shell, thick: f32) {
    let surface = |u: f32, v: f32| shell.at(u, v);
    plate(model.of(stuff), &grid(&surface, ROWS, ACROSS), thick, false);
}

/// A shaped breast, and a back if it has one, of plate or hardened leather.
pub(super) fn add_shell(model: &mut Model, form: &TorsoForm, frame: &Frame, thick: f32) {
    let breast = form.breast().unwrap_or(Breast::Round);
    let front = Shell {
        frame,
        span: Span::Front,
        out: RIGID_OUT,
        breast: Some(breast),
        hem: 0.0,
        reach: None,
    };
    if breast == Breast::Discs {
        add_discs(model, &front, form.back);
    } else {
        let surface = |u: f32, v: f32| {
            let (point, out) = front.at(u, v);
            (point + out * ridges(form, point), out)
        };
        plate(
            model.of(Stuff::Main),
            &grid(&surface, ROWS, ACROSS),
            thick,
            false,
        );
        add_details(model, form, &front);
        if form.back {
            let back = Shell {
                span: Span::Back,
                out: RIGID_OUT - thick - 1.0,
                ..front
            };
            sheet(model, Stuff::Main, &back, thick);
            if breast == Breast::Tanko {
                add_bands(model, &back);
            }
        }
    }
    add_straps(model, &front, form.back);
}

/// How far a ridge down the middle and flutes raise the breast here.
fn ridges(form: &TorsoForm, point: Vec3) -> f32 {
    let TorsoBody::Plate { ridge, fluting, .. } = form.body else {
        return 0.0;
    };
    let middle = if ridge {
        8.0 * (1.0 - point.x.abs() / 30.0).max(0.0)
    } else {
        0.0
    };
    middle
        + match fluting {
            Fluting::None => 0.0,
            Fluting::All => 4.0 * (point.x / 22.0 * TAU).cos().max(0.0),
            // Fanning up from below the waist.
            Fluting::Ridges => {
                let fan = point.x.atan2(point.y - 900.0);
                3.5 * (fan * 22.0).cos().max(0.0)
            }
        }
}

/// The plackart, tankō bands, Dwarven bands and Elven lines, etching and the lance rest.
fn add_details(model: &mut Model, form: &TorsoForm, front: &Shell) {
    let TorsoBody::Plate {
        breast,
        plackart,
        lance_rest,
        ..
    } = form.body
    else {
        return;
    };
    let reach_half = |angle: f32| 0.35 + 0.15 * angle.sin().max(0.0);
    let reach_point = |angle: f32| 0.15 + 0.7 * angle.sin().max(0.0).powi(8);
    let reach: Option<&dyn Fn(f32) -> f32> = match plackart {
        Plackart::None => None,
        Plackart::Half => Some(&reach_half),
        Plackart::Pointed => Some(&reach_point),
    };
    if let Some(reach) = reach {
        let lower = Shell {
            out: front.out + 4.0,
            reach: Some(reach),
            ..*front
        };
        sheet(model, Stuff::Main, &lower, 2.0);
    }
    let surface = |u: f32, v: f32| front.at(u, v);
    match breast {
        Breast::Tanko => add_bands(model, front),
        Breast::Panel => {
            for v in [0.2, 0.5, 0.8] {
                patch(
                    model,
                    Stuff::Trim,
                    &surface,
                    (0.0, 1.0),
                    (v - 0.03, v + 0.03),
                    3.0,
                    3.0,
                );
            }
            studs(model, Stuff::Rivet, &surface, (3, 7), 5.0, 4.0);
        }
        Breast::Leaf => lines(model, &surface, 7, 1.5),
        Breast::Bell => lines(model, &surface, 3, 1.0),
        _ => {}
    }
    if form.finish == Finish::Etched {
        for u in [0.32, 0.5, 0.68] {
            patch(
                model,
                Stuff::Etch,
                &surface,
                (u - 0.025, u + 0.025),
                (0.0, 0.9),
                1.0,
                0.6,
            );
        }
    }
    if form.trim.is_some() {
        patch(
            model,
            Stuff::Trim,
            &surface,
            (0.0, 1.0),
            (0.0, 0.04),
            1.5,
            3.0,
        );
        patch(
            model,
            Stuff::Trim,
            &surface,
            (0.0, 1.0),
            (0.96, 1.0),
            1.5,
            3.0,
        );
    }
    if lance_rest {
        // Bolted on the right breast, below the chest, pointing forwards.
        let (point, out) = front.at(0.72, 0.45);
        let tip = point + out * 45.0 + Vec3::Y * 10.0;
        limb(model.of(Stuff::Main), &[(point, 9.0), (tip, 7.0)]);
    }
}

/// Riveted bands round a tankō: lines of shade with rivets along them.
fn add_bands(model: &mut Model, shell: &Shell) {
    let surface = |u: f32, v: f32| shell.at(u, v);
    lines(model, &surface, 6, 1.5);
    studs(model, Stuff::Rivet, &surface, (5, 12), 2.5, 3.0);
}

/// Three domed discs on the breast, and on the back if it has one: two above, one below.
fn add_discs(model: &mut Model, front: &Shell, back: bool) {
    const PLACES: [(f32, f32, f32); 3] =
        [(0.36, 0.75, 70.0), (0.64, 0.75, 70.0), (0.5, 0.35, 85.0)];
    let behind = Shell {
        span: Span::Back,
        ..*front
    };
    for (u, v, radius) in PLACES {
        let (point, out) = front.at(u, v);
        add_disc(model, point, out, radius);
        if back {
            let (point, out) = behind.at(u, v);
            add_disc(model, point, out, radius);
        }
    }
}

fn add_disc(model: &mut Model, centre: Vec3, out: Vec3, radius: f32) {
    let across = out.cross(Vec3::Y).normalize_or(Vec3::X);
    let up = across.cross(out);
    let surface = |u: f32, v: f32| {
        let angle = u * TAU;
        let dome = 12.0 * (1.0 - v * v);
        let point = centre + (across * angle.cos() + up * angle.sin()) * radius * v + out * dome;
        (point, out)
    };
    let disc: Surface = &surface;
    plate(model.of(Stuff::Main), &grid(disc, 4, 24), 2.0, false);
}

/// Leather straps over each shoulder, from the breast's top to the back's, or crossing
/// behind when there is no back.
fn add_straps(model: &mut Model, front: &Shell, back: bool) {
    let behind = Shell {
        span: Span::Back,
        ..*front
    };
    for (u_front, u_back) in [(0.33, 0.67), (0.67, 0.33)] {
        let (start, _) = front.at(u_front, 1.0);
        let (end, _) = if back {
            behind.at(u_back, 1.0)
        } else {
            behind.at(u_back, 0.15)
        };
        let over = Vec3::new(start.x, start.y.max(end.y) + 22.0, 0.0);
        let path = [
            (start, 6.0),
            (start.lerp(over, 0.6) + Vec3::Y * 8.0, 6.0),
            (end.lerp(over, 0.6) + Vec3::Y * 8.0, 6.0),
            (end, 6.0),
        ];
        limb(model.of(Stuff::Backing), &path);
    }
}
