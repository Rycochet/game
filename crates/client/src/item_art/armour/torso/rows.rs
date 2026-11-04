//! Lamellar and laminar cuirasses: rows of laced lamellae, and hoops of lames riveted
//! to leathers or laced, each row standing over the one below.

use shared::armour::Style;
use shared::armour::torso::{Lacing, LamellarCut, PlateSize, TorsoBody, TorsoForm};
use shared::body::Frame;

use super::RIGID_OUT;
use super::plate::add_straps;
use super::plates::{STEP, stripes, strips};
use super::shell::{Shell, Span};
use super::soft::{round_shell, wrap};
use crate::item_art::armour::surface::{bands, lines, patch, studs};
use crate::item_art::armour::{Model, Stuff};

/// How far lamellar stands off the skin: over a padded coat.
const LAMELLAR_OUT: f32 = 16.0;

/// A breast plate of stamped leather at the top, as on Japanese dō.
fn top_plate(model: &mut Model, shell: &Shell) {
    let surface = |u: f32, v: f32| shell.at(u, v);
    // The front's middle: halfway round a breast, a quarter round the whole body.
    let (middle, half) = if shell.span == Span::Round {
        (0.25, 0.1)
    } else {
        (0.5, 0.2)
    };
    let across = (middle - half, middle + half);
    patch(
        model,
        Stuff::Backing,
        &surface,
        across,
        (0.86, 1.0),
        6.0,
        2.0,
    );
}

pub(super) fn add_lamellar(model: &mut Model, form: &TorsoForm, frame: &Frame) {
    let TorsoBody::Lamellar { cut, size, lacing } = form.body else {
        return;
    };
    let rows = match size {
        PlateSize::Small => 9,
        PlateSize::Medium => 7,
        PlateSize::Large => 6,
    };
    // Boxed ō-yoroi: breast and back stand off square, with a plate on the right side.
    let shells: Vec<Shell> = if cut == LamellarCut::Boxed {
        let front = Shell {
            frame,
            span: Span::Front,
            out: RIGID_OUT,
            breast: None,
            hem: 20.0,
            reach: None,
        };
        vec![
            front,
            Shell {
                span: Span::Back,
                out: RIGID_OUT - 3.0,
                ..front
            },
        ]
    } else {
        vec![round_shell(frame, LAMELLAR_OUT, 15.0)]
    };
    for shell in &shells {
        let surface = |u: f32, v: f32| shell.at(u, v);
        if cut == LamellarCut::Mountain {
            // Small interlocking plates: a close pattern of heads over the surface.
            wrap(model, Stuff::Main, shell, 2.0, &|_, _| 0.0);
            studs(model, Stuff::Shade, &surface, (14, 40), 1.5, 3.5);
        } else {
            bands(model, Stuff::Main, &surface, rows, STEP, 2.0);
            stripes(model, Stuff::Cloth, &surface, rows, 1.2);
            if lacing == Lacing::Full {
                let lift = rows as f32 * STEP + 1.5;
                strips(model, Stuff::Cloth, &surface, 30, lift);
            }
        }
    }
    if cut == LamellarCut::Boxed {
        let side = |u: f32, v: f32| shells[0].at(u, v);
        patch(model, Stuff::Main, &side, (0.88, 1.0), (0.0, 0.7), 8.0, 2.5);
        add_straps(model, &shells[0], true);
    }
    if form.style == Style::Japanese {
        top_plate(model, &shells[0]);
    }
}

pub(super) fn add_laminar(model: &mut Model, form: &TorsoForm, frame: &Frame) {
    let TorsoBody::Laminar { lames, riveted } = form.body else {
        return;
    };
    let lames = lames as usize;
    let shell = round_shell(frame, RIGID_OUT, 0.0);
    let surface = |u: f32, v: f32| shell.at(u, v);
    bands(model, Stuff::Main, &surface, lames, 1.6, 1.6);
    let lift = lames as f32 * 1.6 + 1.0;
    // The lames' edges, so the hoops read at picture size.
    lines(model, &surface, lames, lift - 0.5);
    if riveted {
        studs(model, Stuff::Rivet, &surface, (lames, 20), lift, 2.5);
    } else {
        stripes(model, Stuff::Cloth, &surface, lames, 1.2);
    }
    if form.style == Style::Classical {
        // Lorica segmentata: brass fittings closing each hoop down the front.
        for lame in 0..lames {
            let v = (lame as f32 + 0.5) / lames as f32;
            patch(
                model,
                Stuff::Trim,
                &surface,
                (0.23, 0.27),
                (v - 0.03, v + 0.03),
                lift,
                2.0,
            );
        }
    }
    if form.style == Style::Japanese {
        top_plate(model, &shell);
    }
    if form.trim.is_some() {
        patch(
            model,
            Stuff::Trim,
            &surface,
            (0.0, 1.0),
            (0.95, 1.0),
            lift,
            2.4,
        );
    }
}
