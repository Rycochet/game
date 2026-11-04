//! What goes round the hips in hoops: war girdles and the mitra, faulds and culets, and
//! what plate faulds may have with them: a plain groin plate and a mail skirt beneath.

use std::f32::consts::{PI, TAU};

use shared::armour::waist::{WaistForm, WaistPiece};
use shared::armour::{Construction, Style};

use super::cover::cover;
use super::hips::{BACK, FRONT, Hips, between};
use super::{RIGID_OUT, STANDOFF};
use crate::item_art::armour::surface::{lines, patch};
use crate::item_art::armour::{Model, Stuff};

/// How far round each side of the front a fauld reaches, in radians.
const FAULD_REACH: f32 = 1.35;
/// How far round each side of the back a culet reaches.
const CULET_REACH: f32 = 1.0;

/// A broad band round the waist, over a mail or cloth torso piece, and the mitra's
/// crescent over the belly when it has one.
pub(super) fn girdle(model: &mut Model, form: &WaistForm, hips: &Hips) {
    let (top, bottom) = (hips.waist() + 40.0, hips.waist() - 90.0);
    let band = |u: f32, v: f32| {
        let angle = between(FRONT - PI, FRONT + PI, u);
        hips.at(top + (bottom - top) * v, angle, RIGID_OUT + 10.0)
    };
    cover(model, form, &band, 1, (900.0, top - bottom));
    if form.front {
        let reach = 0.6;
        let plate = |u: f32, v: f32| {
            let across = u * 2.0 - 1.0;
            // A crescent: lowest in the middle.
            let low = hips.waist() - 210.0 + 60.0 * across * across;
            let high = hips.waist() - 30.0;
            let angle = between(FRONT - reach, FRONT + reach, u);
            hips.at(high + (low - high) * v, angle, RIGID_OUT + 16.0 + v * 6.0)
        };
        cover(model, form, &plate, 1, (240.0, 180.0));
    }
}

/// Hoops round the front of the hips, from under the torso piece down to `bottom`, the
/// lowest cut up between the legs when `arch` is more than nothing.
pub(super) fn fauld(
    model: &mut Model,
    form: &WaistForm,
    hips: &Hips,
    hoops: u8,
    bottom: f32,
    arch: f32,
) {
    let top = hips.top();
    let surface = |u: f32, v: f32| {
        let angle = between(FRONT - FAULD_REACH, FRONT + FAULD_REACH, u);
        // Raised in the middle of the front, between the legs.
        let middle = (1.0 - ((u - 0.5) / 0.18).powi(2)).max(0.0);
        let low = bottom + arch * middle;
        hips.at(top + (low - top) * v, angle, STANDOFF + 4.0 + v * 14.0)
    };
    cover(model, form, &surface, hoops as usize, (560.0, top - bottom));
}

/// Hoops over the buttocks behind.
pub(super) fn culet(model: &mut Model, form: &WaistForm, hips: &Hips) {
    let (top, bottom) = (hips.waist() - 40.0, hips.hip() - 40.0);
    let surface = |u: f32, v: f32| {
        let angle = between(BACK - CULET_REACH, BACK + CULET_REACH, u);
        hips.at(top + (bottom - top) * v, angle, STANDOFF + 8.0 + v * 10.0)
    };
    cover(
        model,
        form,
        &surface,
        form.culet as usize,
        (380.0, top - bottom),
    );
}

/// A plain plate over the groin, rounded below (never a codpiece's shape).
pub(super) fn groin(model: &mut Model, form: &WaistForm, hips: &Hips) {
    let (top, bottom) = (hips.crotch() + 60.0, hips.crotch() - 70.0);
    let plate = |u: f32, v: f32| {
        let across = u * 2.0 - 1.0;
        let low = bottom + 50.0 * across * across;
        let angle = between(FRONT - 0.3, FRONT + 0.3, u);
        hips.at(top + (low - top) * v, angle, STANDOFF + 8.0)
    };
    let mut plain = *form;
    plain.style = Style::White;
    cover(model, &plain, &plate, 1, (120.0, 130.0));
}

/// A mail skirt showing below a plate fauld or tassets, all round.
pub(super) fn mail_beneath(model: &mut Model, hips: &Hips) {
    let (top, bottom) = (hips.hip() + 40.0, hips.crotch() - 60.0);
    let skirt = |u: f32, v: f32| {
        let angle = between(FRONT, FRONT + TAU, u);
        hips.at(top + (bottom - top) * v, angle, STANDOFF + 2.0 + v * 8.0)
    };
    patch(
        model,
        Stuff::Backing,
        &skirt,
        (0.0, 1.0),
        (0.0, 1.0),
        0.0,
        2.2,
    );
    lines(model, &skirt, 7, 1.3);
}

/// The fauld a piece hangs from, if it has one, with its culet, groin plate and mail.
pub(super) fn add_hoops(model: &mut Model, form: &WaistForm, hips: &Hips) {
    match form.piece {
        WaistPiece::Girdle => girdle(model, form, hips),
        WaistPiece::Fauld => fauld(model, form, hips, form.lames, hips.crotch(), 70.0),
        // A short fauld of three hoops above tassets.
        WaistPiece::Tassets if form.construction == Construction::Plate => {
            fauld(model, form, hips, 3, hips.hip() + 20.0, 0.0)
        }
        WaistPiece::Tassets => fauld(model, form, hips, 1, hips.hip() + 20.0, 0.0),
        _ => {}
    }
    if form.culet > 0 {
        culet(model, form, hips);
    }
    if form.groin {
        groin(model, form, hips);
    }
    if form.mail_beneath {
        mail_beneath(model, hips);
    }
}
