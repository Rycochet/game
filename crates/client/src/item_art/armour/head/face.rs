//! What covers the face: nasals, spectacles, bars, cheek pieces, the T-opening's
//! cheeks, faceplates with their slits and breaths, visors of five shapes, masks, and a
//! bevored helm's throat plates.

use std::f32::consts::PI;

use bevy::prelude::*;
use shared::armour::Style;
use shared::armour::head::{Face, HeadForm, HeadKind, Visor};

use super::bowl::bowl_patch;
use super::shell::Shell;
use super::{THICK, sheet};
use crate::item_art::armour::{Model, Stuff};
use crate::item_art::body::blob;

/// Levels on the face, as shares of the head's height up from the chin.
const EYES: f32 = 0.5;
const BROW: f32 = 0.6;
const NOSE: f32 = 0.27;
const MOUTH: f32 = 0.18;

fn hump(t: f32, width: f32) -> f32 {
    (-(t / width).powi(2)).exp()
}

/// Both sides' angle ranges from `near` to `far` either side of the front.
fn both(near: f32, far: f32) -> [(f32, f32); 2] {
    [(near, far), (-far, -near)]
}

pub(super) fn add_face(model: &mut Model, form: &HeadForm, shell: &Shell) {
    let level = |share| shell.level(share);
    let (brow, eyes, chin) = (level(BROW), level(EYES), level(0.0));
    if form.cheeks {
        // Hung from inside the bowl's edge, however high a brim keeps it.
        let top = level(0.42).max(shell.edge_at(1.25) + 12.0);
        for angles in both(0.75, 1.75) {
            bowl_patch(model, Stuff::Main, shell, angles, (chin + 10.0, top), 2.0);
        }
    }
    match form.face {
        Face::Open => {}
        Face::Nasal => nasal(model, shell, brow, level(NOSE) - 8.0),
        Face::Spectacles => {
            nasal(model, shell, brow, level(NOSE));
            for (near, far) in both(0.08, 0.6) {
                bowl_patch(
                    model,
                    Stuff::Main,
                    shell,
                    (near, far),
                    (eyes - 24.0, eyes - 14.0),
                    3.0,
                );
                let outer = if near > 0.0 { far } else { near };
                bowl_patch(
                    model,
                    Stuff::Main,
                    shell,
                    (outer - 0.06, outer + 0.06),
                    (eyes - 24.0, brow),
                    3.0,
                );
            }
        }
        Face::Bars => {
            for angle in [-0.2, 0.0, 0.2] {
                sheet(
                    model,
                    Stuff::Main,
                    shell,
                    (angle - 0.025, angle + 0.025),
                    (chin + 10.0, brow),
                    &|_, v| 30.0 * v.sqrt(),
                    THICK,
                    (6, 1),
                );
            }
        }
        Face::Opening => {
            // The cheeks come round the face, leaving a T to see and breathe through.
            for angles in both(0.13, 1.6) {
                bowl_patch(
                    model,
                    Stuff::Main,
                    shell,
                    angles,
                    (chin - 5.0, eyes - 8.0),
                    0.5,
                );
            }
            nasal(model, shell, brow, eyes - 8.0);
        }
        Face::Faceplate => {
            slits(model, shell, eyes, 0.5);
            // A cross of reinforces, and breaths on the right.
            bowl_patch(model, Stuff::Main, shell, (-0.04, 0.04), (chin, brow), 2.0);
            for row in 0..4 {
                for column in 0..3 {
                    let (point, out) = shell.point(
                        -0.25 - column as f32 * 0.12,
                        level(0.35) - row as f32 * 14.0,
                    );
                    blob(model.of(Stuff::Shade), point + out, Vec3::splat(3.0));
                }
            }
        }
        Face::Slit => {
            // The lower front stands well out, under one wide slit.
            sheet(
                model,
                Stuff::Main,
                shell,
                (-1.1, 1.1),
                (chin - 15.0, eyes - 6.0),
                &|u, v| 26.0 * (u * PI).sin() * (1.0 - v * 0.4),
                THICK,
                (4, 16),
            );
            slits(model, shell, eyes, 1.0);
        }
        Face::Visor(visor) => add_visor(model, shell, visor, form.style),
        Face::Mask => add_mask(model, shell, form.style == Style::Japanese),
    }
    if form.kind == HeadKind::BevoredHelm {
        // Throat plates down to the neck, under the face.
        let (low, high) = (form.edge(shell.frame).front as f32, chin + 15.0);
        sheet(
            model,
            Stuff::Main,
            shell,
            (-1.9, 1.9),
            (low, high),
            &|_, v| 22.0 - 16.0 * v,
            THICK,
            (4, 20),
        );
    }
}

/// A bar down the nose.
fn nasal(model: &mut Model, shell: &Shell, from: f32, to: f32) {
    bowl_patch(
        model,
        Stuff::Main,
        shell,
        (-0.1, 0.1),
        (to, from + 10.0),
        3.0,
    );
}

/// Dark sight slits either side.
fn slits(model: &mut Model, shell: &Shell, eyes: f32, lift: f32) {
    for angles in both(0.06, 0.55) {
        bowl_patch(
            model,
            Stuff::Shade,
            shell,
            angles,
            (eyes - 2.0, eyes + 6.0),
            lift + 0.8,
        );
    }
}

/// A visor over the face, standing out from the bowl and pivoting at the temples.
fn add_visor(model: &mut Model, shell: &Shell, visor: Visor, style: Style) {
    let (chin, eyes, brow) = (
        shell.level(-0.05),
        shell.level(EYES),
        shell.level(BROW + 0.05),
    );
    let low = if visor == Visor::Klapp {
        shell.level(0.05)
    } else {
        chin
    };
    let snout = |u: f32, v: f32| {
        let across = (u - 0.5) * 2.0;
        let front = (1.0 - across * across).max(0.0);
        let height = low + (brow - low) * v;
        12.0 + match visor {
            Visor::Pointed => 85.0 * front.powi(3) * hump(height - shell.level(0.3), 70.0),
            Visor::Rounded | Visor::Grotesque => 8.0 * front,
            Visor::Bellows => 10.0 * front + 6.0 * ((v * 3.0 * PI).sin()).max(0.0),
            Visor::Klapp => 4.0 * front,
        }
    };
    sheet(
        model,
        Stuff::Main,
        shell,
        (-1.35, 1.35),
        (low, brow),
        &snout,
        THICK,
        (10, 20),
    );
    let lift = |angle: f32, height: f32| {
        let u = (angle + 1.35) / 2.7;
        snout(u, (height - low) / (brow - low)) + 1.0
    };
    if visor == Visor::Grotesque {
        let bone = if style == Style::Dwarven {
            Stuff::Main
        } else {
            Stuff::Bone
        };
        for side in [-1.0, 1.0] {
            let (point, out) = shell.point(side * 0.32, eyes);
            blob(
                model.of(Stuff::Shade),
                point + out * lift(side * 0.32, eyes),
                Vec3::new(14.0, 11.0, 8.0),
            );
        }
        for at in 0..7 {
            let angle = (at as f32 - 3.0) * 0.09;
            let (point, out) = shell.point(angle, shell.level(MOUTH));
            blob(
                model.of(bone),
                point + out * lift(angle, shell.level(MOUTH)),
                Vec3::new(5.0, 9.0, 5.0),
            );
        }
    } else {
        for angles in both(0.05, 0.6) {
            let height = eyes + 2.0;
            let (from, to) = angles;
            let middle = (from + to) / 2.0;
            sheet(
                model,
                Stuff::Shade,
                shell,
                angles,
                (height - 3.0, height + 3.0),
                &|_, _| lift(middle, height) + 0.5,
                0.8,
                (1, 4),
            );
        }
    }
    // The pivots at the temples.
    for side in [-1.0, 1.0] {
        let (point, out) = shell.point(side * 1.4, brow - 12.0);
        blob(model.of(Stuff::Rivet), point + out * 14.0, Vec3::splat(6.0));
    }
}

/// A face: a Roman cavalry mask over all of it, or a Japanese one from the nose down.
fn add_mask(model: &mut Model, shell: &Shell, japanese: bool) {
    let top = shell.level(if japanese { 0.32 } else { BROW });
    let chin = shell.level(-0.04);
    let nose = shell.level(0.35);
    let face = |u: f32, v: f32| {
        let across = (u - 0.5) * 2.0;
        let height = chin + (top - chin) * v;
        4.0 + 18.0 * hump(across, 0.12) * hump(height - nose, 30.0)
            + 6.0 * hump(height - shell.level(MOUTH), 8.0) * hump(across, 0.4)
    };
    sheet(
        model,
        Stuff::Main,
        shell,
        (-1.1, 1.1),
        (chin, top),
        &face,
        THICK,
        (10, 16),
    );
    let holes: &[(f32, f32)] = if japanese {
        &[(0.0, MOUTH)]
    } else {
        &[(-0.3, EYES), (0.3, EYES), (0.0, MOUTH)]
    };
    for &(angle, share) in holes {
        let (point, out) = shell.point(angle, shell.level(share));
        blob(
            model.of(Stuff::Shade),
            point + out * 6.0,
            Vec3::new(12.0, 5.0, 5.0),
        );
    }
}
