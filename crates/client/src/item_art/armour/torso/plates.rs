//! Armour of many small plates: which builder makes each, the patterns they share
//! (lacing across rows, strips down), and scale shirts.

use shared::armour::torso::{Backing, PlateSize, ScaleShape, TorsoBody, TorsoForm};
use shared::body::Frame;

use super::soft::{MAIL_OUT, round_shell, sleeves, wrap};
use super::{cloth, mirror, rows, slats};
use crate::item_art::armour::surface::{Surface, lines, patch, scales, studs};
use crate::item_art::armour::{Model, Stuff};

/// How far each row stands out over the one below.
pub(super) const STEP: f32 = 1.4;

/// Scale, lamellar, laminar, plates in cloth, plated mail, mirrors, slats and rattan.
pub(super) fn add_plates(model: &mut Model, form: &TorsoForm, frame: &Frame) {
    match form.body {
        TorsoBody::Scale { .. } => add_scales(model, form, frame),
        TorsoBody::Lamellar { .. } => rows::add_lamellar(model, form, frame),
        TorsoBody::Laminar { .. } => rows::add_laminar(model, form, frame),
        TorsoBody::PlatesInCloth { .. } => cloth::add_brigandine(model, form, frame),
        TorsoBody::MailAndPlate { .. } => cloth::add_plated_mail(model, form, frame),
        TorsoBody::Mirror { .. } => mirror::add_mirrors(model, form, frame),
        TorsoBody::Splint { .. } => slats::add_slats(model, form, frame),
        TorsoBody::Woven => slats::add_rattan(model, frame),
        _ => {}
    }
}

/// Thin stripes of this stuff across a surface between `rows` rows, standing out with
/// them: lacing between lamellae and lames.
pub(super) fn stripes(model: &mut Model, stuff: Stuff, surface: Surface, rows: usize, lift: f32) {
    for row in 1..rows {
        let v = row as f32 / rows as f32;
        let lift = lift + (rows - row) as f32 * STEP;
        patch(
            model,
            stuff,
            surface,
            (0.0, 1.0),
            (v - 0.02, v + 0.02),
            lift,
            0.8,
        );
    }
}

/// Thin strips down a surface, `count` of them: lacing or stitching.
pub(super) fn strips(model: &mut Model, stuff: Stuff, surface: Surface, count: usize, lift: f32) {
    for column in 0..count {
        let u = (column as f32 + 0.5) / count as f32;
        patch(
            model,
            stuff,
            surface,
            (u - 0.003, u + 0.003),
            (0.0, 1.0),
            lift,
            0.8,
        );
    }
}

/// Rows of scales up the body, each over the one below, on cloth, leather or mail;
/// coins sewn on in rows.
fn add_scales(model: &mut Model, form: &TorsoForm, frame: &Frame) {
    let TorsoBody::Scale {
        shape,
        size,
        backing,
        sleeves: has_sleeves,
    } = form.body
    else {
        return;
    };
    let shell = round_shell(frame, MAIL_OUT, 10.0);
    let surface = |u: f32, v: f32| shell.at(u, v);
    let under = match backing {
        Backing::Cloth => Stuff::Cloth,
        Backing::Leather => Stuff::Backing,
        Backing::Mail => Stuff::Shade,
    };
    wrap(model, under, &shell, 2.0, &|_, _| 0.0);
    if backing == Backing::Mail {
        lines(model, &surface, 20, 1.0);
    }
    // Small scales would be lost at picture size, and cost the most: kept to a few.
    let rows = match size {
        PlateSize::Small => 12,
        PlateSize::Medium => 9,
        PlateSize::Large => 6,
    };
    if shape == ScaleShape::Coin {
        studs(model, Stuff::Main, &surface, (rows, rows * 3), 2.0, 9.0);
    } else {
        scales(model, Stuff::Main, &surface, rows, rows * 3);
        // A shadow under each row, so the rows read at picture size.
        lines(model, &surface, rows, 3.0 + rows as f32 * 0.9);
    }
    if has_sleeves {
        sleeves(model, Stuff::Main, frame, 10.0);
    }
}
