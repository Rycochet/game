//! What hangs rather than holds its shape: mail and ring mail, padding and the arming
//! vest, buff leather, and their short sleeves. Each goes right round the body.

use std::f32::consts::PI;

use bevy::prelude::*;
use shared::armour::torso::{Quilting, TorsoBody, TorsoForm, Weave};
use shared::body::{Frame, Joint, Side};
use shared::material::Material;

use super::ROWS;
use super::shell::{Shell, Span};
use crate::item_art::armour::surface::{grid, lines, patch, studs};
use crate::item_art::armour::{Model, Stuff, plate};
use crate::item_art::body::limb;

/// Points round: enough for quilting to show.
const ROUND: usize = 48;
/// How far mail, padding and the vest stand off the skin.
pub(super) const MAIL_OUT: f32 = 12.0;
const PADDED_OUT: f32 = 15.0;
const VEST_OUT: f32 = 6.0;

fn round_shell(frame: &Frame, out: f32, hem: f32) -> Shell<'_> {
    Shell {
        frame,
        span: Span::Round,
        out,
        breast: None,
        hem,
        reach: None,
    }
}

/// A shell right round, raised in places by `puff`, `thick` millimetres.
fn wrap(
    model: &mut Model,
    stuff: Stuff,
    shell: &Shell,
    thick: f32,
    puff: &dyn Fn(f32, f32) -> f32,
) {
    let surface = |u: f32, v: f32| {
        let (point, out) = shell.at(u, v);
        (point + out * puff(u, v), out)
    };
    plate(
        model.of(stuff),
        &grid(&surface, ROWS * 2, ROUND),
        thick,
        false,
    );
}

/// A short sleeve round the top of each arm, as far as the middle of the upper arm.
fn sleeves(model: &mut Model, stuff: Stuff, frame: &Frame, out: f32) {
    for side in Side::BOTH {
        let shoulder = frame.joint(Joint::Shoulder(side));
        let elbow = frame.joint(Joint::Elbow(side));
        let at = |mm: [i32; 3]| Vec3::new(mm[0] as f32, mm[1] as f32, mm[2] as f32);
        let (top, end) = (at(shoulder), at(shoulder).lerp(at(elbow), 0.5));
        let radius = frame.upper_arm as f32 + out;
        limb(model.of(stuff), &[(top, radius * 1.15), (end, radius)]);
    }
}

/// A mail shirt: rings shaded in rows, a brass border if it has one, and Japanese mail
/// sewn on cloth.
pub(super) fn add_mail(model: &mut Model, form: &TorsoForm, frame: &Frame, out: f32) {
    let shell = round_shell(frame, out, 10.0);
    let japanese = matches!(
        form.body,
        TorsoBody::Mail {
            weave: Weave::Japanese,
            ..
        }
    );
    let surface = |u: f32, v: f32| shell.at(u, v);
    if japanese {
        wrap(model, Stuff::Cloth, &shell, 2.0, &|_, _| 0.0);
        let net = Shell {
            out: out + 1.5,
            ..shell
        };
        let net_surface = |u: f32, v: f32| net.at(u, v);
        lines(model, &net_surface, 16, 0.0);
        for column in 0..32 {
            let u = column as f32 / 32.0;
            patch(
                model,
                Stuff::Shade,
                &net_surface,
                (u - 0.003, u + 0.003),
                (0.0, 1.0),
                0.0,
                0.8,
            );
        }
    } else {
        wrap(model, Stuff::Main, &shell, 3.0, &|_, _| 0.0);
        lines(model, &surface, 26, 1.2);
    }
    if form.trim.is_some() {
        patch(
            model,
            Stuff::Trim,
            &surface,
            (0.0, 1.0),
            (0.0, 0.05),
            1.5,
            2.5,
        );
    }
    if form.sleeves() {
        sleeves(
            model,
            if japanese { Stuff::Cloth } else { Stuff::Main },
            frame,
            10.0,
        );
    }
}

/// Rings sewn flat on a leather coat, in rows.
pub(super) fn add_ring_mail(model: &mut Model, form: &TorsoForm, frame: &Frame) {
    let shell = round_shell(frame, MAIL_OUT, 5.0);
    wrap(model, Stuff::Backing, &shell, 3.0, &|_, _| 0.0);
    let surface = |u: f32, v: f32| shell.at(u, v);
    studs(model, Stuff::Main, &surface, (10, 34), 2.5, 4.5);
    if form.sleeves() {
        sleeves(model, Stuff::Backing, frame, 10.0);
    }
}

/// How padding puffs up between its stitching.
fn quilted(quilting: Quilting) -> impl Fn(f32, f32) -> f32 {
    move |u: f32, v: f32| {
        let (across, down) = (u * 40.0 * PI, v * 9.0 * PI);
        3.0 * match quilting {
            Quilting::Channels => across.sin().abs(),
            Quilting::Squares => (across.sin() * down.sin()).abs(),
            Quilting::Diamonds => ((across + down).sin() * (across - down).sin()).abs(),
            Quilting::Laminated => 0.0,
        }
    }
}

/// A padded jack, linothorax, coir armour or arming vest: thick cloth puffed between
/// its stitching, with painted borders on laminated linen.
pub(super) fn add_padded(model: &mut Model, form: &TorsoForm, frame: &Frame) {
    let TorsoBody::Padded { quilting, .. } = form.body else {
        return;
    };
    let (out, thick) = if form.vest {
        (VEST_OUT, 6.0)
    } else {
        (PADDED_OUT, 10.0)
    };
    let shell = round_shell(frame, out, 0.0);
    let stuff = if form.material == Material::Coir {
        Stuff::Main
    } else {
        Stuff::Cloth
    };
    wrap(model, stuff, &shell, thick, &quilted(quilting));
    let surface = |u: f32, v: f32| shell.at(u, v);
    if quilting == Quilting::Laminated {
        for v in [(0.0, 0.07), (0.93, 1.0)] {
            patch(
                model,
                Stuff::Trim,
                &surface,
                (0.0, 1.0),
                v,
                thick / 2.0,
                1.0,
            );
        }
    }
    // Closed down the front.
    patch(
        model,
        Stuff::Shade,
        &surface,
        (0.245, 0.255),
        (0.0, 1.0),
        thick / 2.0,
        1.0,
    );
    if form.sleeves() {
        sleeves(model, stuff, frame, 14.0);
    }
}

/// A buff jerkin: soft thick leather, hooked down the front.
pub(super) fn add_buff(model: &mut Model, frame: &Frame) {
    let shell = round_shell(frame, MAIL_OUT, 0.0);
    wrap(model, Stuff::Main, &shell, 4.0, &|_, _| 0.0);
    let surface = |u: f32, v: f32| shell.at(u, v);
    patch(
        model,
        Stuff::Shade,
        &surface,
        (0.245, 0.255),
        (0.0, 1.0),
        2.0,
        1.0,
    );
}
