//! Plate and hardened leather on one leg: cuisses, poleyns, greaves and schynbalds in
//! each style, Classical greaves and ankle guards, spikes, fins and studs.

use std::f32::consts::PI;

use shared::armour::legs::{Cover, Knee, LegsForm, Ornament};
use shared::armour::{Construction, Style};

use super::RIGID_LIFT;
use super::path::{FRONT, Leg, OUTSIDE, WING};
use super::shapes::{Tube, band, cop, fan, flat, spike, strips, turn};
use crate::item_art::armour::surface::{grid, studs};
use crate::item_art::armour::{Model, Stuff, plate};

/// Rigid pieces on one leg, as the form's cover says.
pub(super) fn add_rigid(model: &mut Model, form: &LegsForm, leg: &Leg) {
    let thick = if form.construction == Construction::Leather {
        4.0
    } else {
        1.3
    };
    if form.cover == Cover::Ankle {
        add_ankle(model, form, leg);
        return;
    }
    let knee = leg.knee();
    if form.cover.thigh() {
        add_cuisse(model, form, leg, thick);
    }
    if form.cover.shin() {
        let top = match form.style {
            // Knemides come up over the knee themselves.
            Style::Classical if form.cover.knee() => knee + 90.0,
            _ if form.knee != Knee::None => knee - 55.0,
            _ => knee - 40.0,
        };
        add_greave(model, form, leg, top, thick);
    }
    if form.knee != Knee::None {
        add_knee(model, form, leg, RIGID_LIFT, Stuff::Main);
    }
}

/// A cuisse: from the crotch at the front, rising to the belt on the outside and
/// dropping behind, down to the knee's top lame; open-backed unless closed.
fn add_cuisse(model: &mut Model, form: &LegsForm, leg: &Leg, thick: f32) {
    let (crotch, knee) = (leg.crotch(), leg.knee());
    let elven = form.style == Style::Elven;
    let top = move |angle: f32| {
        let outside = 60.0 * angle.sin().max(0.0);
        let behind = -50.0 * (-angle.cos()).max(0.0);
        let point = if elven {
            45.0 * angle.cos().max(0.0).powi(6)
        } else {
            0.0
        };
        crotch + outside + behind + point
    };
    let bottom = if form.cover.knee() {
        knee + 75.0
    } else {
        knee + 60.0
    };
    let tube = Tube {
        leg,
        top: &top,
        bottom,
        turn: turn(form.closed),
        lift: RIGID_LIFT,
        style: form.style,
        swell: 0.0,
    };
    tube.add(model, Stuff::Main, thick);
    add_style(model, form, &tube);
    edges(model, form, leg, &tube, bottom);
    if form.studded {
        studs(
            model,
            Stuff::Rivet,
            &|u, v| tube.at(u, v),
            (5, 10),
            3.0,
            2.5,
        );
    }
}

/// A greave or schynbald from `top` down to the ankle, flared over a sabaton's top.
fn add_greave(model: &mut Model, form: &LegsForm, leg: &Leg, top: f32, thick: f32) {
    let elven = form.style == Style::Elven;
    let pointed = move |angle: f32| top + 40.0 * angle.cos().max(0.0).powi(6);
    let plain = flat(top);
    let edge: &dyn Fn(f32) -> f32 = if elven { &pointed } else { &plain };
    let classical = form.style == Style::Classical;
    let tube = Tube {
        leg,
        top: edge,
        bottom: leg.ankle(),
        turn: turn(form.closed),
        lift: RIGID_LIFT + 2.0,
        style: form.style,
        swell: if classical { 12.0 } else { 0.0 },
    };
    tube.add(model, Stuff::Main, thick);
    // Maximilian harnesses were never fluted on the greaves.
    if form.style != Style::Maximilian {
        add_style(model, form, &tube);
    }
    if form.coiled() {
        for ring in 1..14 {
            let v = ring as f32 / 14.0;
            let ridge = |u: f32, w: f32| {
                let (point, out) = tube.at(u, v - 0.015 + 0.03 * w);
                (point + out * 3.0, out)
            };
            plate(model.of(Stuff::Main), &grid(&ridge, 1, 24), 1.5, false);
        }
    }
    if form.closed && !classical && form.construction == Construction::Plate {
        // The hinge down the outside.
        let hinge = Tube {
            turn: (OUTSIDE - 0.04, OUTSIDE + 0.04),
            lift: tube.lift + 1.5,
            ..tube
        };
        studs(
            model,
            Stuff::Rivet,
            &|u, v| hinge.at(u, v),
            (5, 1),
            1.0,
            3.0,
        );
    }
    edges(model, form, leg, &tube, leg.ankle());
    if form.studded {
        studs(
            model,
            Stuff::Rivet,
            &|u, v| tube.at(u, v),
            (6, 10),
            3.0,
            2.5,
        );
    }
    if form.ornament == Ornament::ShinFins {
        add_fin(model, leg, &tube);
    }
}

/// The knee's cop, a lame above and below, a fan when winged, a spike when spiked, of
/// `stuff`: the piece's own, or iron over padding.
pub(super) fn add_knee(model: &mut Model, form: &LegsForm, leg: &Leg, lift: f32, stuff: Stuff) {
    let knee = leg.knee();
    let size = if form.style == Style::IndoPersian {
        0.95
    } else {
        0.75
    };
    cop(model, stuff, leg, lift + 4.0, form.style, size);
    if form.construction == Construction::Plate && form.style != Style::Classical {
        let front = (FRONT - 0.45 * PI, FRONT + 0.45 * PI);
        for heights in [(knee + 88.0, knee + 68.0), (knee - 68.0, knee - 88.0)] {
            band(
                model,
                stuff,
                leg,
                heights,
                front,
                lift + 2.0,
                form.style,
                1.3,
            );
        }
    }
    if form.knee == Knee::Winged {
        let (size, pointed) = match form.style {
            Style::Milanese => (130.0, false),
            Style::Gothic | Style::Elven => (110.0, true),
            Style::Dwarven | Style::Kastenbrust => (90.0, false),
            _ => (95.0, false),
        };
        fan(model, stuff, leg, lift, size, pointed);
    }
    if form.ornament == Ornament::KneeSpikes {
        let (base, out) = leg.at(knee, FRONT, lift + 20.0);
        spike(model, Stuff::Trim, base, out, 50.0);
    }
}

/// What a style adds over a plate: Gothic's ridges, Maximilian's flutes, Greenwich's
/// etched bands, Dwarven rivets.
fn add_style(model: &mut Model, form: &LegsForm, tube: &Tube) {
    if form.construction != Construction::Plate {
        return;
    }
    let front = Tube {
        turn: (WING - 0.55 * PI, WING - 0.05 * PI),
        ..*tube
    };
    match form.style {
        Style::Gothic => strips(model, Stuff::Main, &front, 3, 0.06, 4.0),
        Style::Greenwich => strips(model, Stuff::Etch, &front, 2, 0.22, 0.6),
        Style::Maximilian => strips(model, Stuff::Main, tube, 9, 0.03, 3.5),
        Style::Dwarven => studs(model, Stuff::Rivet, &|u, v| tube.at(u, v), (2, 8), 2.0, 3.5),
        _ => {}
    }
}

/// Rolled edges at the top and bottom, in the trim's metal when it has one.
fn edges(model: &mut Model, form: &LegsForm, leg: &Leg, tube: &Tube, bottom: f32) {
    let stuff = if form.trim.is_some() {
        Stuff::Trim
    } else {
        Stuff::Main
    };
    let lift = tube.lift + 1.5;
    band(
        model,
        stuff,
        leg,
        (bottom + 6.0, bottom),
        tube.turn,
        lift,
        form.style,
        2.0,
    );
}

/// A blade-like ridge standing out down the front of the shin.
fn add_fin(model: &mut Model, leg: &Leg, tube: &Tube) {
    let fin = |u: f32, v: f32| {
        let (point, out) = tube.at(0.5, 0.15 + 0.7 * v);
        let height = 30.0 * (PI * v).sin().max(0.0).sqrt();
        (point + out * height * u, leg.across(FRONT))
    };
    plate(model.of(Stuff::Trim), &grid(&fin, 6, 1), 2.0, false);
}

/// Bronze ankle guards: round the ankle bones and up the back of the heel, open over
/// the instep, under a greave's flared edge.
fn add_ankle(model: &mut Model, form: &LegsForm, leg: &Leg) {
    let ankle = leg.ankle();
    let tube = Tube {
        leg,
        top: &flat(ankle + 50.0),
        bottom: ankle - 15.0,
        turn: (0.3 * PI, 1.7 * PI),
        lift: 10.0,
        style: form.style,
        swell: 0.0,
    };
    tube.add(model, Stuff::Main, 1.3);
}
