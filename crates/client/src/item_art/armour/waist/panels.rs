//! What hangs in separate pieces all round from a cord or belt: hip panels (kusazuri,
//! steppe and Chinese flaps) and pteruges in rows of strips, or the Roman apron's few
//! strips over the front with discs at their ends.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use shared::armour::waist::{Length, WaistForm};

use super::cover::cover;
use super::hips::{FRONT, Hips, between};
use super::{SOFT_OUT, STANDOFF};
use crate::item_art::armour::surface::{patch, studs};
use crate::item_art::armour::{Model, Stuff};
use crate::item_art::body::blob;

/// The cord or belt it hangs from, right round the waist.
fn belt(model: &mut Model, hips: &Hips, stuff: Stuff, depth: f32) {
    let top = hips.top();
    let band = |u: f32, v: f32| {
        let angle = between(FRONT, FRONT + TAU, u);
        hips.at(top - depth * v, angle, STANDOFF + 3.0)
    };
    patch(model, stuff, &band, (0.0, 1.0), (0.0, 1.0), 0.0, 4.0);
}

/// Panels hung all round, flaring a little; two big flaps part at the front and back.
pub(super) fn add_panels(model: &mut Model, form: &WaistForm, hips: &Hips, bottom: f32) {
    belt(model, hips, Stuff::Cloth, 40.0);
    let top = hips.top() - 40.0;
    let count = form.hangs.max(1) as f32;
    let width = TAU / count;
    // Two flaps hang at the sides, so they part at the front; more have one in front.
    let first = if form.hangs == 2 { 0.0 } else { FRONT };
    let flare = if form.length == Length::Knee {
        40.0
    } else {
        25.0
    };
    for panel in 0..form.hangs {
        let centre = first + width * panel as f32;
        let (from, to) = (centre - width / 2.0 + 0.03, centre + width / 2.0 - 0.03);
        let surface = |u: f32, v: f32| {
            let angle = between(from, to, u);
            hips.at(top + (bottom - top) * v, angle, SOFT_OUT + 4.0 + flare * v)
        };
        let wide = 170.0 * (to - from);
        cover(
            model,
            form,
            &surface,
            form.lames as usize,
            (wide, top - bottom),
        );
    }
}

/// Rows of strips, the outer rows shorter, each strip parted from the next; over the
/// front only on the Roman apron, with studs and a disc at each end.
pub(super) fn add_pteruges(model: &mut Model, form: &WaistForm, hips: &Hips, bottom: f32) {
    belt(model, hips, Stuff::Main, 50.0);
    let top = hips.top() - 45.0;
    let rows = form.lames.max(1) as usize;
    let (from, to, strip) = if form.front {
        (FRONT - 0.55, FRONT + 0.55, 45.0)
    } else {
        (FRONT - PI, FRONT + PI, 70.0)
    };
    let round = 180.0 * (to - from);
    let count = ((round / strip) as usize).max(3);
    for row in 0..rows {
        // The outer rows end higher, so the inner ones show below.
        let outer = rows - 1 - row;
        let low = bottom + outer as f32 * 70.0;
        let out = SOFT_OUT + 6.0 + outer as f32 * 5.0;
        // Each row's strips over the gaps of the next.
        let offset = if row % 2 == 0 { 0.0 } else { 0.5 };
        for at in 0..count {
            let centre = (at as f32 + 0.5 + offset) / count as f32;
            if centre > 1.0 {
                continue;
            }
            let half = 0.42 / count as f32;
            let surface = |u: f32, v: f32| {
                let angle = between(from, to, centre - half + 2.0 * half * u);
                hips.at(top + (low - top) * v, angle, out + 12.0 * v)
            };
            cover(model, form, &surface, 1, (strip, top - low));
            if form.front {
                studs(model, Stuff::Rivet, &surface, (4, 1), 2.5, 3.5);
                let (end, way) = surface(0.5, 1.0);
                blob(
                    model.of(Stuff::Trim),
                    end + way * 3.0 - Vec3::Y * 10.0,
                    Vec3::new(14.0, 14.0, 3.0),
                );
            }
        }
    }
}
