//! Gorgets: a collar of lames (or a low rim) round the neck, plates over the top of the
//! chest and back, lames over the shoulders, or the nodowa's bib of lacquered lames.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use shared::armour::neck::{NeckForm, Plates};
use shared::armour::{Construction, Style};

use super::around::{Around, Profile, dir};
use super::curtain::{Curtain, dress};
use super::hanging::Fall;
use crate::item_art::armour::surface::{Surface, bands, grid, patch, studs};
use crate::item_art::armour::{Model, Stuff, face, plate};
use crate::item_art::body::Triangle;

/// Under a breastplate (16th century), clear of the clothes under it.
const UNDER_OFF: f32 = 12.0;
/// Over a breastplate (17th century): outside its 25 mm.
const OVER_OFF: f32 = 32.0;
const THICK: f32 = 1.6;

fn ring((across, deep): (f32, f32), y: f32, angle: f32) -> Vec3 {
    Vec3::new(across * angle.sin(), y, deep * angle.cos())
}

/// A curtain all the way round, the seam behind.
fn whole<'a>(around: &'a Around<'a>, profile: &'a dyn Fn(f32) -> Profile, off: f32) -> Curtain<'a> {
    Curtain {
        from: -PI,
        to: PI,
        profile,
        hem: shared::armour::neck::Hem::Straight,
        depth: 0.0,
        clear: (around, off),
    }
}

pub(super) fn add_gorget(model: &mut Model, form: &NeckForm, around: &Around) {
    if form.plates == Plates::Bib {
        add_bib(model, form, around);
        return;
    }
    let off = if form.over_breastplate() {
        OVER_OFF
    } else {
        UNDER_OFF
    };
    let (nb, n) = (around.neck_base(), around.neck());
    let low = nb + 5.0;
    let high = if form.high {
        around.chin() - 20.0
    } else {
        nb + 35.0
    };
    let base = (around.base_across() + off, n * 1.2 + off);
    let top = if form.high {
        (n + 14.0, n + 14.0)
    } else {
        (base.0 - 8.0, base.1 - 8.0)
    };
    let collar = |angle| Profile::straight(ring(top, high, angle), ring(base, low, angle));
    let curtain = whole(around, &collar, off);
    let surface = curtain.surface(0.0);
    let rigid = matches!(
        form.construction,
        Construction::Plate | Construction::Leather
    );
    if rigid && form.lames > 1 {
        curtain.bands(model, Stuff::Main, form.lames as usize, 1.0, THICK);
    } else {
        dress(model, &curtain, form.construction, form.studded);
    }
    // A rolled edge along the top.
    let edge = if form.trim.is_some() {
        Stuff::Trim
    } else {
        Stuff::Main
    };
    curtain.band(model, edge, (0.0, 0.08), 2.0, 2.5);
    if form.spikes {
        add_spikes(model.of(Stuff::Rivet), &surface);
    }
    if form.plates == Plates::Collar {
        return;
    }
    let fall = Fall {
        front: nb - 90.0,
        back: nb - 70.0,
        side_out: around.base_across() + 35.0,
        cape: false,
        off,
    };
    let plates = |angle| fall.profile(around, angle, collar(angle).bottom);
    let curtain = whole(around, &plates, off);
    if rigid {
        curtain.sheet(model, Stuff::Main, 0.0, THICK);
        let surface = curtain.surface(0.0);
        let rim = |u: f32, v: f32| surface(u, 0.92 + v * 0.08);
        let stuff = if form.studded {
            Stuff::Rivet
        } else {
            Stuff::Main
        };
        studs(model, stuff, &rim, (1, 18), 1.2, 2.2);
        add_style(model, form, &curtain);
    } else {
        dress(model, &curtain, form.construction, form.studded);
    }
    if form.plates == Plates::Shoulders {
        for side in [1.0, -1.0] {
            add_shoulder_lames(model, around, off, side);
        }
    }
}

/// Three lames out along the top of one shoulder, under where a pauldron goes.
fn add_shoulder_lames(model: &mut Model, around: &Around, off: f32, side: f32) {
    let inner = around.base_across() + 20.0;
    let outer = around.shoulder_half() - 40.0;
    let point = move |u: f32, v: f32| {
        let angle = side * (FRAC_PI_2 + (u - 0.5) * 1.3);
        let out = inner + (outer - inner) * v;
        around.over_shoulder(angle, out, off + 3.0)
    };
    let surface = move |u: f32, v: f32| (point(u, v), Vec3::Y);
    bands(model, Stuff::Main, &surface, 3, 1.2, THICK);
}

/// The nodowa: a collar round the front of the neck with lames hanging over the breast,
/// tied behind with cords.
fn add_bib(model: &mut Model, form: &NeckForm, around: &Around) {
    let (nb, n) = (around.neck_base(), around.neck());
    let off = UNDER_OFF;
    let collar = |angle| {
        Profile::straight(
            ring((n + 14.0, n + 14.0), around.chin() - 25.0, angle),
            ring((around.base_across() + off, n * 1.2 + off), nb + 5.0, angle),
        )
    };
    let front_half = Curtain {
        from: -1.75,
        to: 1.75,
        profile: &collar,
        hem: shared::armour::neck::Hem::Straight,
        depth: 0.0,
        clear: (around, off),
    };
    let laced = matches!(
        form.construction,
        Construction::Laminar | Construction::Lamellar
    );
    let rows = if laced { form.lames.max(2) as usize } else { 1 };
    front_half.bands(model, Stuff::Main, 2, 1.0, THICK);
    let hang = |u: f32, v: f32| {
        let angle = (u - 0.5) * 1.6;
        let y = nb + 5.0 - 85.0 * v;
        let point = if y >= nb {
            ring((around.base_across() + off, n * 1.2 + off), y, angle)
        } else {
            around.trunk(angle, y, off)
        };
        (point, dir(angle))
    };
    bands(model, Stuff::Main, &hang, rows, 1.2, THICK);
    // Lacing up the lames, and the cords (or a strap) round the back of the neck.
    for column in (1..6).filter(|_| laced) {
        let u = column as f32 / 6.0;
        patch(
            model,
            Stuff::Cloth,
            &hang,
            (u - 0.012, u + 0.012),
            (0.0, 1.0),
            4.0,
            1.0,
        );
    }
    let cord = |u: f32, v: f32| {
        let angle = 1.75 + (2.0 * PI - 3.5) * u;
        let radius = n + 4.0;
        (
            ring((radius, radius), nb + 25.0 - 6.0 * v, angle),
            dir(angle),
        )
    };
    let tie = if laced { Stuff::Cloth } else { Stuff::Backing };
    plate(model.of(tie), &grid(&cord, 1, 12), 2.0, false);
}

/// Cones standing out round the middle of a collar.
fn add_spikes(triangles: &mut Vec<Triangle>, surface: Surface) {
    const COUNT: usize = 10;
    for spike in 0..COUNT {
        let (base, out) = surface((spike as f32 + 0.5) / COUNT as f32, 0.5);
        let tip = base + out * 38.0;
        let side = out.cross(Vec3::Y).normalize_or_zero();
        let up = side.cross(out).normalize_or_zero();
        let corners: Vec<Vec3> = (0..6)
            .map(|k| {
                let a = k as f32 * PI / 3.0;
                base + (side * a.cos() + up * a.sin()) * 7.0
            })
            .collect();
        for k in 0..6 {
            let (a, b) = (corners[k], corners[(k + 1) % 6]);
            face(triangles, a, b, tip, (a + b) / 2.0 - base);
        }
    }
}

/// What a plate harness's style adds to the plates: Maximilian flutes running down
/// them, Greenwich's etched bands.
fn add_style(model: &mut Model, form: &NeckForm, curtain: &Curtain) {
    if form.construction != Construction::Plate {
        return;
    }
    match form.style {
        Style::Maximilian => {
            let surface = curtain.surface(0.0);
            for flute in 0..28 {
                let u = (flute as f32 + 0.5) / 28.0;
                patch(
                    model,
                    Stuff::Main,
                    &surface,
                    (u - 0.006, u + 0.006),
                    (0.1, 0.9),
                    1.8,
                    2.0,
                );
            }
        }
        Style::Greenwich => curtain.band(model, Stuff::Etch, (0.35, 0.6), 1.0, 0.8),
        _ => {}
    }
}
