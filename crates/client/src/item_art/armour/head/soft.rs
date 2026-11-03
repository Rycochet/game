//! Caps and coifs: a quilted cap close over the skull with a strap under the chin, and
//! a hood of mail or quilting, open for the face, whose skirt spreads to the shoulders.

use std::f32::consts::TAU;

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::head::{Edge, HeadForm};
use shared::body::Frame;

use super::bowl::meridian;
use super::shell::Shell;
use crate::item_art::armour::surface::{lines, ring_grid};
use crate::item_art::armour::{Model, Stuff, plate};
use crate::item_art::body::limb;

/// Rows up and points round.
const ROWS: usize = 12;
const ROUND: usize = 40;

pub(super) fn add_cap(model: &mut Model, form: &HeadForm, frame: &Frame) {
    let shell = Shell::new(form, frame, 6.0);
    let surface = |u: f32, v: f32| shell.bowl(u, v);
    plate(
        model.of(Stuff::Cloth),
        &ring_grid(&surface, ROWS, ROUND),
        4.0,
        true,
    );
    quilting(model, &shell);
    let (ear, chin) = (shell.level(0.35), frame.chin as f32);
    let across = frame.head.0 as f32 + 6.0;
    limb(
        model.of(Stuff::Shade),
        &[
            (Vec3::new(across, ear, 0.0), 4.0),
            (Vec3::new(across * 0.6, chin, 30.0), 4.0),
            (Vec3::new(0.0, chin - 12.0, 45.0), 4.0),
            (Vec3::new(-across * 0.6, chin, 30.0), 4.0),
            (Vec3::new(-across, ear, 0.0), 4.0),
        ],
    );
}

/// Stitched lines from the edge to the crown.
fn quilting(model: &mut Model, shell: &Shell) {
    for at in 0..12 {
        meridian(
            model,
            Stuff::Shade,
            shell,
            at as f32 * TAU / 12.0,
            0.015,
            2.5,
        );
    }
}

pub(super) fn add_coif(model: &mut Model, form: &HeadForm, frame: &Frame) {
    let stuff = if form.construction == Construction::Padded {
        Stuff::Cloth
    } else {
        Stuff::Main
    };
    let out = 18.0;
    let chin = frame.chin as f32;
    let mut hood = Shell::new(form, frame, out);
    let below = frame.chin - 10;
    hood.edge = Edge {
        front: frame.chin + (frame.height - frame.chin) * 62 / 100,
        side: below,
        back: below,
    };
    let top = |u: f32, v: f32| hood.bowl(u, v);
    plate(model.of(stuff), &ring_grid(&top, ROWS, ROUND), 3.0, true);
    // The skirt, from under the chin to the shoulders.
    let (head_across, head_deep) = (frame.head.0 as f32 + out, frame.head.1 as f32 + out);
    let (low_across, low_deep) = (
        frame.shoulder_half as f32 * 0.8,
        frame.chest_half.1 as f32 + 15.0,
    );
    let (high, low) = (chin - 5.0, frame.shoulder as f32 + 20.0);
    let skirt = |u: f32, v: f32| {
        let angle = u * TAU;
        let t = v * v;
        let across = head_across + (low_across - head_across) * t;
        let deep = head_deep + (low_deep - head_deep) * t;
        let point = Vec3::new(
            across * angle.sin(),
            high + (low - high) * v,
            deep * angle.cos(),
        );
        let out =
            Vec3::new(angle.sin() / across, 0.4 * v, angle.cos() / deep).normalize_or(Vec3::Z);
        (point, out)
    };
    plate(model.of(stuff), &ring_grid(&skirt, 6, ROUND), 3.0, true);
    if form.construction == Construction::Padded {
        quilting(model, &hood);
    } else {
        lines(model, &top, 10, 1.6);
        lines(model, &skirt, 8, 1.6);
    }
}
