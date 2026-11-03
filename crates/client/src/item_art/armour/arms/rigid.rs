//! Plate and hardened leather on one arm: bracers, vambraces, couters and rerebraces in
//! each style, bazubands, bronze forearm guards, spikes and studs.

use std::f32::consts::PI;

use shared::armour::arms::{ArmsForm, Couter, Reach, Spikes};
use shared::armour::{Construction, Style};

use super::RIGID_LIFT;
use super::path::{Arm, BACK, INSIDE, WING};
use super::shapes::{Tube, cup, fan, flat, ring, spike, strips};
use crate::item_art::armour::surface::studs;
use crate::item_art::armour::{Model, Stuff};

/// Rigid pieces on one arm, as the form's reach says.
pub(super) fn add_rigid(model: &mut Model, form: &ArmsForm, arm: &Arm) {
    let thick = if form.construction == Construction::Leather {
        4.0
    } else {
        1.4
    };
    let (top, bottom) = form.reach.span(arm.frame);
    let (top, bottom) = (top as f32, bottom as f32);
    let elbow = arm.elbow();
    if form.reach == Reach::Wrist {
        add_bracer(model, form, arm, (top, bottom), thick);
        return;
    }
    if form.bazubands() {
        add_bazuband(model, form, arm, bottom);
        return;
    }
    let turn = if form.closed {
        (-PI, PI)
    } else {
        (-0.75 * PI, 0.75 * PI)
    };
    if form.reach.forearm() {
        let start = if form.reach == Reach::Forearm {
            top
        } else {
            elbow + 40.0
        };
        let elven = move |angle: f32| start - 45.0 * angle.cos().max(0.0).powi(6);
        let plain = flat(start);
        let top_edge: &dyn Fn(f32) -> f32 = if form.style == Style::Elven {
            &elven
        } else {
            &plain
        };
        let tube = Tube {
            arm,
            top: top_edge,
            bottom,
            turn,
            lift: RIGID_LIFT,
            style: form.style,
        };
        tube.add(model, Stuff::Main, thick);
        add_style(model, form, &tube);
        edges(model, form, arm, (start, bottom));
        if form.studded {
            studs(
                model,
                Stuff::Rivet,
                &|u, v| tube.at(u, v),
                (4, 10),
                3.0,
                2.5,
            );
        }
        if form.spikes == Spikes::Ridge {
            for at in 1..=3 {
                let down = start + (bottom - start) * at as f32 / 4.0;
                let (base, out) = arm.at(down, WING, RIGID_LIFT + 1.0);
                spike(model, Stuff::Trim, base, out, 30.0);
            }
        }
    }
    if form.reach.upper() {
        let tube = Tube {
            arm,
            top: &flat(top),
            bottom: elbow - 45.0,
            turn,
            lift: RIGID_LIFT - 2.0,
            style: form.style,
        };
        tube.add(model, Stuff::Main, thick);
        add_style(model, form, &tube);
        // The top edge turned out, under the pauldron's lames.
        ring(
            model,
            Stuff::Main,
            arm,
            (top, top + 8.0),
            RIGID_LIFT + 1.0,
            form.style,
            2.0,
        );
    }
    if form.couter != Couter::None {
        add_couter(model, form, arm);
    }
}

/// The elbow's cup, lames above and below it, and a fan when winged.
pub(super) fn add_couter(model: &mut Model, form: &ArmsForm, arm: &Arm) {
    let elbow = arm.elbow();
    cup(model, Stuff::Main, arm, RIGID_LIFT + 2.0, form.style);
    for (top, bottom) in [(elbow - 72.0, elbow - 52.0), (elbow + 52.0, elbow + 72.0)] {
        ring(
            model,
            Stuff::Main,
            arm,
            (top, bottom),
            RIGID_LIFT + 1.0,
            form.style,
            1.4,
        );
    }
    if form.couter == Couter::Winged {
        let (size, pointed) = match form.style {
            Style::Milanese => (140.0, false),
            Style::Gothic | Style::Elven => (110.0, true),
            Style::Dwarven | Style::Kastenbrust => (100.0, false),
            _ => (90.0, false),
        };
        fan(model, Stuff::Main, arm, RIGID_LIFT, size, pointed);
    }
    if form.spikes == Spikes::Elbow {
        let (base, out) = arm.at(elbow, BACK, RIGID_LIFT + 14.0);
        spike(model, Stuff::Trim, base, out, 45.0);
    }
}

/// What a style adds over a plate tube: Gothic's ridge, Maximilian's flutes,
/// Greenwich's etched bands, Dwarven rivets, Classical rings.
fn add_style(model: &mut Model, form: &ArmsForm, tube: &Tube) {
    if form.construction != Construction::Plate {
        return;
    }
    // Gothic ridges and Greenwich bands down the outer front; Maximilian flutes all
    // round.
    let outer = Tube {
        turn: (WING - 0.25 * PI, WING + 0.25 * PI),
        ..*tube
    };
    match form.style {
        Style::Gothic => strips(model, Stuff::Main, &outer, 3, 0.07, 4.0),
        Style::Greenwich => strips(model, Stuff::Etch, &outer, 2, 0.25, 0.6),
        Style::Maximilian => strips(model, Stuff::Main, tube, 8, 0.03, 3.5),
        _ => {}
    }
    if form.style == Style::Dwarven {
        studs(model, Stuff::Rivet, &|u, v| tube.at(u, v), (2, 8), 2.0, 3.5);
    }
    if form.style == Style::Classical {
        // Bronze forearm guards ringed like coils.
        for band in 1..8 {
            let v = band as f32 / 8.0;
            let ridge = |u: f32, w: f32| {
                let (point, out) = tube.at(u, v - 0.012 + 0.024 * w);
                (point + out * 2.5, out)
            };
            crate::item_art::armour::plate(
                model.of(Stuff::Main),
                &crate::item_art::armour::surface::grid(&ridge, 1, 16),
                1.5,
                false,
            );
        }
    }
}

/// Rolled edges at the top and bottom, in the trim's metal when it has one.
fn edges(model: &mut Model, form: &ArmsForm, arm: &Arm, (top, bottom): (f32, f32)) {
    let stuff = if form.trim.is_some() {
        Stuff::Trim
    } else {
        Stuff::Main
    };
    let lift = RIGID_LIFT + 1.5;
    if form.closed {
        ring(
            model,
            stuff,
            arm,
            (bottom - 6.0, bottom),
            lift,
            form.style,
            2.0,
        );
        if form.style != Style::Elven {
            ring(model, stuff, arm, (top, top + 6.0), lift, form.style, 2.0);
        }
    }
}

/// A bracer: a curved sheet over the inside of the forearm above the wrist, strapped.
fn add_bracer(
    model: &mut Model,
    form: &ArmsForm,
    arm: &Arm,
    (top, bottom): (f32, f32),
    thick: f32,
) {
    let elven = move |angle: f32| top - 40.0 * (angle - INSIDE).cos().max(0.0).powi(4);
    let plain = flat(top);
    let top_edge: &dyn Fn(f32) -> f32 = if form.style == Style::Elven {
        &elven
    } else {
        &plain
    };
    let lift = RIGID_LIFT - 6.0;
    let sheet = Tube {
        arm,
        top: top_edge,
        bottom,
        turn: (INSIDE - 0.65 * PI, INSIDE + 0.65 * PI),
        lift,
        style: form.style,
    };
    sheet.add(model, Stuff::Main, thick);
    for at in [0.3, 0.7] {
        let down = top + (bottom - top) * at;
        ring(
            model,
            Stuff::Backing,
            arm,
            (down - 9.0, down + 9.0),
            lift - 3.0,
            form.style,
            2.0,
        );
    }
    if form.studded || form.style == Style::Dwarven {
        studs(
            model,
            Stuff::Rivet,
            &|u, v| sheet.at(u, v),
            (3, 4),
            thick,
            2.5,
        );
    }
}

/// A bazuband: one long outer plate from the wrist up past the elbow to a rounded
/// point, a narrow inner plate, and bands of gilt or brass round the wrist.
fn add_bazuband(model: &mut Model, form: &ArmsForm, arm: &Arm, bottom: f32) {
    let elbow = arm.elbow();
    let rise = if form.reach == Reach::ForearmAndElbow {
        130.0
    } else {
        40.0
    };
    let point = move |angle: f32| elbow + 40.0 - rise * (angle - BACK).cos().max(0.0).powi(3);
    let outer = Tube {
        arm,
        top: &point,
        bottom,
        turn: (BACK - 0.55 * PI, BACK + 0.85 * PI),
        lift: RIGID_LIFT,
        style: form.style,
    };
    outer.add(model, Stuff::Main, 1.6);
    let inner = Tube {
        arm,
        top: &flat(elbow + 80.0),
        bottom,
        turn: (INSIDE - 0.2 * PI, INSIDE + 0.35 * PI),
        lift: RIGID_LIFT - 2.0,
        style: form.style,
    };
    inner.add(model, Stuff::Main, 1.4);
    for (top, end) in [
        (bottom - 30.0, bottom - 22.0),
        (bottom - 12.0, bottom - 4.0),
    ] {
        let band = Tube {
            arm,
            top: &flat(top),
            bottom: end,
            turn: outer.turn,
            lift: RIGID_LIFT + 1.2,
            style: form.style,
        };
        band.add(model, Stuff::Trim, 1.0);
    }
}
