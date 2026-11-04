//! Skirts hung as one from a waistband, closed or split for riding or striding: tonlets
//! and steel bases belling out, mail, scale, lamellar, brigandine and quilted skirts;
//! and mail breeches round the hips with a short tube on each thigh.

use std::f32::consts::{PI, TAU};

use shared::armour::Construction;
use shared::armour::waist::{Opening, WaistForm};
use shared::body::{Frame, Side};

use super::cover::cover;
use super::hips::{BACK, FRONT, Hips, between};
use super::{SOFT_OUT, STANDOFF};
use crate::item_art::armour::Model;
use crate::item_art::armour::Stuff;
use crate::item_art::armour::legs::Leg;
use crate::item_art::armour::surface::{lines, patch};

/// Where a split skirt parts, as gaps in radians either side of these angles.
const GAP: f32 = 0.05;

/// The angles round the body each part of a skirt covers.
fn parts(opening: Opening) -> Vec<(f32, f32)> {
    match opening {
        Opening::Closed => vec![(FRONT, FRONT + TAU)],
        Opening::FrontAndBack => vec![(FRONT + GAP, BACK - GAP), (BACK + GAP, FRONT + TAU - GAP)],
        Opening::Sides => vec![(GAP, PI - GAP), (PI + GAP, TAU - GAP)],
    }
}

/// How far a skirt bells out at its hem, beyond where it starts.
fn flare(form: &WaistForm) -> f32 {
    match form.construction {
        Construction::Plate => 75.0,
        Construction::Mail | Construction::RingMail => 18.0,
        _ => 30.0,
    }
}

pub(super) fn add_skirt(model: &mut Model, form: &WaistForm, hips: &Hips, bottom: f32) {
    let top = hips.top();
    // The waistband, closed, which a split skirt parts below.
    let band_low = top - 55.0;
    let band = |u: f32, v: f32| {
        let angle = between(FRONT, FRONT + TAU, u);
        hips.at(top + (band_low - top) * v, angle, STANDOFF + 3.0)
    };
    let stuff = match form.construction {
        Construction::Plate | Construction::Laminar => Stuff::Main,
        _ => Stuff::Backing,
    };
    patch(model, stuff, &band, (0.0, 1.0), (0.0, 1.0), 0.0, 3.5);
    let rigid = form.construction == Construction::Plate;
    let out = if rigid { STANDOFF + 6.0 } else { SOFT_OUT };
    let flare = flare(form);
    for (from, to) in parts(form.opening) {
        let surface = |u: f32, v: f32| {
            let angle = between(from, to, u);
            let bell = flare * v.powf(1.4);
            hips.at(
                band_low + 10.0 + (bottom - band_low - 10.0) * v,
                angle,
                out + bell,
            )
        };
        let wide = 180.0 * (to - from);
        cover(
            model,
            form,
            &surface,
            form.lames as usize,
            (wide, band_low - bottom),
        );
    }
}

/// Mail shorts: round the hips to the crotch, then a short tube round each thigh.
pub(super) fn add_breeches(model: &mut Model, form: &WaistForm, frame: &Frame, hips: &Hips) {
    let (top, crotch) = (hips.top(), hips.crotch());
    let round = |u: f32, v: f32| {
        let angle = between(FRONT, FRONT + TAU, u);
        hips.at(top + (crotch - top) * v, angle, SOFT_OUT)
    };
    cover(model, form, &round, 0, (900.0, top - crotch));
    for side in Side::BOTH {
        let leg = Leg { frame, side };
        let (high, low) = (crotch + 40.0, crotch - 110.0);
        let tube = |u: f32, v: f32| {
            let height = high + (low - high) * v;
            leg.at(height, u * TAU, SOFT_OUT + 2.0)
        };
        patch(model, Stuff::Main, &tube, (0.0, 1.0), (0.0, 1.0), 0.0, 2.5);
        lines(model, &tube, 6, 1.4);
    }
}
