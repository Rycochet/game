//! Tassets hanging from the fauld over each thigh: single plates (pointed in Gothic and
//! Elven styles), laminated ones, small ones at the sides, the Dendra's lower pairs,
//! and cuirassier tassets wrapping the thigh down to the legs' own knee cop.

use std::f32::consts::PI;

use shared::armour::legs::{Cover, Knee, LegsForm, LegsKind, Ornament, Reinforce};
use shared::armour::waist::{Length, WaistForm, WaistKind};
use shared::armour::{Style, arms::Sides};
use shared::body::{Frame, Side};

use super::HANGING_OUT;
use super::cover::cover;
use crate::item_art::armour::legs::{Leg, add_knee};
use crate::item_art::armour::{Model, Stuff};

/// How far cuirassier tassets stand off the thigh at the knee, meeting the knee cop
/// (the legs' rigid pieces stand 15 mm off).
const WRAPPED_OUT: f32 = 17.0;
/// How much flatter than the thigh hanging tassets curve.
const FLAT: f32 = 2.0;

/// One tasset on a leg: from `top` down to `bottom`, round the thigh from `from` to
/// `to` (0 its front, a quarter turn its outside), standing `out` off it at the top and
/// `low_out` at the bottom; `point` drops its middle that far below its corners.
#[allow(clippy::too_many_arguments)]
fn tasset(
    model: &mut Model,
    form: &WaistForm,
    leg: &Leg,
    (top, bottom): (f32, f32),
    (from, to): (f32, f32),
    (out, low_out): (f32, f32),
    point: f32,
    lames: usize,
) {
    let skin_top = leg.skin(top);
    let surface = |u: f32, v: f32| {
        let across = (u * 2.0 - 1.0).abs();
        let low = bottom + point * across;
        let height = top + (low - top) * v;
        let angle = from + (to - from) * u;
        match form.kind {
            // Wrapped ones follow the thigh in.
            WaistKind::Cuirassier => {
                leg.round(height, angle, leg.skin(height) + out + (low_out - out) * v)
            }
            // Hanging ones fall straight from the fauld, curved more gently than the
            // thigh: as wide round, on a circle twice as big touching it in front.
            WaistKind::Armour => {
                let radius = skin_top + out + (low_out - out) * v;
                let (front, _) = leg.round(height, 0.0, radius);
                let centre = front - leg.out(0.0) * radius * FLAT;
                let way = leg.out(angle / FLAT);
                (centre + way * radius * FLAT, way)
            }
        }
    };
    let wide = leg.skin(top) * (to - from);
    cover(model, form, &surface, lames, (wide, top - bottom));
}

/// The legs' form for the knee cop under cuirassier tassets.
fn knee_form(form: &WaistForm) -> LegsForm {
    LegsForm {
        kind: LegsKind::Armour,
        construction: form.construction,
        style: form.style,
        cover: Cover::ThighAndKnee,
        closed: true,
        knee: Knee::Winged,
        reinforce: Reinforce::None,
        sides: Sides::Both,
        ornament: Ornament::None,
        studded: false,
        material: form.material,
        finish: form.finish,
        cloth: None,
        trim: form.trim,
    }
}

/// The tassets on one leg, as worn.
pub(super) fn add_tassets(model: &mut Model, form: &WaistForm, frame: &Frame, side: Side) {
    let leg = Leg { frame, side };
    let top = frame.hip as f32 + 40.0;
    let bottom = form.length.bottom(frame) as f32;
    let lames = form.lames as usize;
    if form.kind == WaistKind::Cuirassier {
        let bottom = frame.knee as f32 + 60.0;
        let reach = (-0.75 * PI, 0.8 * PI);
        tasset(
            model,
            form,
            &leg,
            (top, bottom),
            reach,
            (HANGING_OUT, WRAPPED_OUT),
            0.0,
            lames,
        );
        add_knee(
            model,
            &knee_form(form),
            &leg,
            WRAPPED_OUT - 2.0,
            Stuff::Main,
        );
        return;
    }
    let point = match form.style {
        Style::Gothic if lames == 1 => 60.0,
        Style::Elven if lames == 1 => 90.0,
        _ => 0.0,
    };
    let front = (-0.45, 0.85);
    if form.style == Style::Classical && form.hangs == 6 {
        // The Dendra's three pairs, each hung over the one below.
        let step = (top - bottom) / 3.0;
        for pair in 0..3 {
            let high = top - step * pair as f32;
            let out = HANGING_OUT + 6.0 * (2 - pair) as f32;
            tasset(
                model,
                form,
                &leg,
                (high, high - step - 30.0),
                front,
                (out, out + 4.0),
                0.0,
                1,
            );
        }
        return;
    }
    let flare = if form.length == Length::Knee {
        14.0
    } else {
        8.0
    };
    let out = (HANGING_OUT, HANGING_OUT + flare);
    tasset(model, form, &leg, (top, bottom), front, out, point, lames);
    if form.hangs == 4 {
        // A small one at the side, behind the front one's edge.
        let side_bottom = top - (top - bottom) * 0.6;
        let reach = (0.95, 1.6);
        let side_out = (HANGING_OUT - 4.0, HANGING_OUT + flare - 4.0);
        tasset(
            model,
            form,
            &leg,
            (top - 10.0, side_bottom),
            reach,
            side_out,
            point * 0.5,
            1,
        );
    }
}
