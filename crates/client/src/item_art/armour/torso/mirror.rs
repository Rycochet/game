//! Mirror armour: large polished plates strapped over its own mail shirt, a round one
//! on breast and back, or four (char-aina) on breast, back and both sides.

use shared::armour::torso::{Mirrors, TorsoBody, TorsoForm};
use shared::body::Frame;

use super::RIGID_OUT;
use super::plate::{add_disc, add_straps};
use super::shell::{Shell, Span};
use super::soft::{MAIL_OUT, add_mail, sleeves};
use crate::item_art::armour::surface::patch;
use crate::item_art::armour::{Model, Stuff};

pub(super) fn add_mirrors(model: &mut Model, form: &TorsoForm, frame: &Frame) {
    let TorsoBody::Mirror { mirrors } = form.body else {
        return;
    };
    add_mail(model, form, frame, MAIL_OUT);
    sleeves(model, Stuff::Main, frame, 10.0);
    let front = Shell {
        frame,
        span: Span::Front,
        out: RIGID_OUT,
        breast: None,
        hem: 0.0,
        reach: None,
    };
    let back = Shell {
        span: Span::Back,
        out: RIGID_OUT - 3.0,
        ..front
    };
    match mirrors {
        Mirrors::Discs => {
            for shell in [&front, &back] {
                let (point, out) = shell.at(0.5, 0.6);
                add_disc(model, point, out, 110.0);
            }
        }
        Mirrors::Four => {
            for shell in [&front, &back] {
                plate(model, shell, (0.3, 0.7), (0.15, 0.85));
            }
            // The side plates, cut low under the arms.
            plate(model, &front, (0.0, 0.16), (0.1, 0.55));
            plate(model, &front, (0.84, 1.0), (0.1, 0.55));
        }
    }
    add_straps(model, &front, true);
}

/// One polished plate, its edges inlaid.
fn plate(model: &mut Model, shell: &Shell, across: (f32, f32), up: (f32, f32)) {
    let surface = |u: f32, v: f32| shell.at(u, v);
    patch(model, Stuff::Main, &surface, across, up, 0.0, 2.5);
    for edge in [(up.0, up.0 + 0.03), (up.1 - 0.03, up.1)] {
        patch(model, Stuff::Trim, &surface, across, edge, 1.5, 1.0);
    }
}
