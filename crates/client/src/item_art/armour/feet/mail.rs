//! Mail shoes: mail over the whole shoe and up round the ankle, in rows, with a strap
//! under it or a leather sole. Ring mail is rings sewn on leather; kusari tabi are
//! Japanese mail on a cloth sock with the big toe apart.

use std::f32::consts::PI;

use shared::armour::Construction;
use shared::armour::feet::FeetForm;

use super::last::{Foot, Last};
use super::parts::{ankle_band, foot_band, sheet, sole};
use crate::item_art::armour::legs::Leg;
use crate::item_art::armour::surface::{lines, patch, studs};
use crate::item_art::armour::{Model, Stuff};

/// How far round the mail reaches: nearly under the shoe.
const UNDER: f32 = PI * 0.88;

pub(super) fn add_mail_shoe(model: &mut Model, form: &FeetForm, last: &Last, foot: &Foot) {
    let whole = last.patch((last.back(), last.tip()), UNDER);
    let leg = Leg {
        frame: foot.frame,
        side: foot.side,
    };
    let ankle = foot.frame.ankle as f32;
    let radius = foot.frame.ankle_radius as f32 + last.lift;
    let cuff = (58.0, ankle + 35.0);
    if form.construction == Construction::RingMail {
        sheet(model, Stuff::Backing, &whole, 2.0);
        // Rings over the top and sides, not under the shoe.
        let upper = last.patch((last.back(), last.tip()), 2.0);
        studs(model, Stuff::Main, &upper, (6, 10), 1.5, 2.6);
        ankle_band(model, Stuff::Backing, &leg, cuff, radius, PI, 2.0);
    } else {
        sheet(model, Stuff::Main, &whole, 2.0);
        lines(model, &whole, 14, 1.2);
        let band = if form.japanese() {
            Stuff::Cloth
        } else {
            Stuff::Main
        };
        ankle_band(model, band, &leg, cuff, radius, PI, 2.0);
    }
    if form.japanese() {
        // The split between the big toe and the rest, on the inside.
        let toes = foot.toes() - 45.0;
        let from = (toes - last.back()) / (last.tip() - last.back());
        patch(
            model,
            Stuff::Cloth,
            &whole,
            (0.4, 0.43),
            (from, 1.0),
            1.4,
            1.0,
        );
    }
    if form.soled || form.construction == Construction::RingMail {
        sole(model, last);
    } else {
        foot_band(model, Stuff::Backing, last, 40.0, 14.0);
    }
}
