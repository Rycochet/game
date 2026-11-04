//! Plates under cloth or leather (brigandines, coats of plates, jacks, dingjia), and
//! plated mail: a mail shirt with plates set in it front and back.

use shared::armour::Style;
use shared::armour::torso::{PlateRows, PlateSize, Rivets, TorsoBody, TorsoForm};
use shared::body::Frame;

use super::RIGID_OUT;
use super::plate::add_disc;
use super::plates::strips;
use super::shell::{Shell, Span};
use super::soft::{MAIL_OUT, add_mail, round_shell, wrap};
use crate::item_art::armour::surface::{lines, patch, studs};
use crate::item_art::armour::{Model, Stuff};

/// How far a brigandine stands off the skin.
const CLOTH_OUT: f32 = 15.0;

pub(super) fn add_brigandine(model: &mut Model, form: &TorsoForm, frame: &Frame) {
    let TorsoBody::PlatesInCloth {
        size,
        rivets,
        leather,
    } = form.body
    else {
        return;
    };
    let shell = round_shell(frame, CLOTH_OUT, 10.0);
    let surface = |u: f32, v: f32| shell.at(u, v);
    let cover = if leather {
        Stuff::Backing
    } else {
        Stuff::Cloth
    };
    wrap(model, cover, &shell, 4.0, &|_, _| 0.0);
    let (rows, columns) = match size {
        PlateSize::Small => (12, 40),
        PlateSize::Medium => (8, 28),
        PlateSize::Large => (5, 16),
    };
    if rivets == Rivets::Hidden {
        // A jack: a grid of stitching holding each plate.
        lines(model, &surface, rows, 2.2);
        strips(model, Stuff::Shade, &surface, columns, 2.2);
    } else {
        let radius = if form.style == Style::Chinese {
            4.5
        } else {
            2.8
        };
        studs(model, Stuff::Rivet, &surface, (rows, columns), 2.0, radius);
    }
    // Closed down the front.
    patch(
        model,
        Stuff::Shade,
        &surface,
        (0.245, 0.255),
        (0.0, 1.0),
        2.5,
        1.0,
    );
    if form.style == Style::Chinese {
        // A round mirror over the heart.
        let (point, out) = shell.at(0.25, 0.65);
        add_disc(model, point + out * 3.0, out, 55.0);
    }
}

/// Plates in columns over a breast or back, standing out from the mail.
fn plate_rows(model: &mut Model, shell: &Shell, rows: PlateRows) {
    let surface = |u: f32, v: f32| shell.at(u, v);
    let (columns, down, gap) = match rows {
        PlateRows::Wide => (2, 6, 0.0),
        PlateRows::Narrow => (4, 9, 0.0),
        PlateRows::Grid => (3, 4, 0.04),
    };
    let (left, right) = (0.3, 0.7);
    let width = (right - left) / columns as f32;
    for column in 0..columns {
        let u0 = left + column as f32 * width;
        for row in 0..down {
            let v0 = 0.08 + row as f32 * 0.8 / down as f32;
            let v1 = v0 + 0.8 / down as f32 * 1.15 - gap;
            let lift = 3.0 + (down - row) as f32 * 0.8;
            let across = (u0 + 0.005, u0 + width - 0.005);
            patch(model, Stuff::Main, &surface, across, (v0, v1), lift, 1.8);
        }
    }
}

pub(super) fn add_plated_mail(model: &mut Model, form: &TorsoForm, frame: &Frame) {
    let TorsoBody::MailAndPlate { rows, .. } = form.body else {
        return;
    };
    add_mail(model, form, frame, MAIL_OUT);
    for span in [Span::Front, Span::Back] {
        let shell = Shell {
            frame,
            span,
            out: RIGID_OUT - 10.0,
            breast: None,
            hem: 0.0,
            reach: None,
        };
        plate_rows(model, &shell, rows);
    }
}
