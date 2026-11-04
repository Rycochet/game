//! Neck pieces that hang: mail standards and mantles (an upright collar flaring into a
//! cape), aventails hung from a helmet's lower edge, and padded and kikko collars.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::head::OFF_HEAD;
use shared::armour::neck::{Edging, Front, Hem, NeckForm, NeckKind};

use super::around::{Around, Profile, dir, sideness};
use super::curtain::{Curtain, dress};
use crate::item_art::armour::surface::{patch, studs};
use crate::item_art::armour::{Model, Stuff};

/// Mail and padding stand this far off the clothes under them.
const SOFT_OFF: f32 = 10.0;
/// How deep a hem's points or scallops are.
const HEM_DEPTH: f32 = 18.0;

/// A point on a ring round the middle.
fn ring((across, deep): (f32, f32), y: f32, angle: f32) -> Vec3 {
    Vec3::new(across * angle.sin(), y, deep * angle.cos())
}

/// How a curtain falls from its top to the body below: to these heights in front and
/// behind, and at the sides this far out, or over the shoulder and down the outside of
/// the arm for a cape.
pub(super) struct Fall {
    pub front: f32,
    pub back: f32,
    pub side_out: f32,
    pub cape: bool,
    pub off: f32,
}

impl Fall {
    /// Its profile at an angle, from this top.
    pub fn profile(&self, around: &Around, angle: f32, top: Vec3) -> Profile {
        let height = if angle.cos() >= 0.0 {
            self.front
        } else {
            self.back
        };
        let front = around.trunk(angle, height, self.off);
        let (side, crest) = if self.cape {
            let out = around.shoulder_half() + around.upper_arm() + 30.0;
            let bottom = dir(angle) * out + Vec3::Y * (around.shoulder() - 25.0);
            let crest = around.over_shoulder(angle, around.shoulder_half(), self.off + 4.0);
            (bottom, crest)
        } else {
            let bottom = around.over_shoulder(angle, self.side_out, self.off);
            (bottom, (top + bottom) / 2.0)
        };
        let s = sideness(angle);
        let bottom = front.lerp(side, s);
        Profile {
            top,
            middle: ((top + bottom) / 2.0).lerp(crest, s),
            bottom,
        }
    }
}

/// An upright tube round the neck from `low` to `high`, flaring to the trunk's top.
fn tube(around: &Around, low: f32, high: f32, radius: f32, off: f32) -> impl Fn(f32) -> Profile {
    let base = (around.base_across() + off, around.neck() * 1.2 + off);
    move |angle| Profile::straight(ring((radius, radius), high, angle), ring(base, low, angle))
}

fn whole<'a>(
    around: &'a Around<'a>,
    profile: &'a dyn Fn(f32) -> Profile,
    hem: Hem,
    off: f32,
) -> Curtain<'a> {
    Curtain {
        from: -PI,
        to: PI,
        profile,
        hem,
        depth: HEM_DEPTH,
        clear: (around, off),
    }
}

/// Brass edges or patterned bands in mail.
fn add_edging(model: &mut Model, curtain: &Curtain, edging: Edging) {
    let bands: &[(f32, f32)] = match edging {
        Edging::None => &[],
        Edging::Brass => &[(0.9, 1.0)],
        Edging::Patterned => &[(0.3, 0.37), (0.6, 0.67), (0.9, 1.0)],
    };
    for &rows in bands {
        curtain.band(model, Stuff::Trim, rows, 1.6, 1.0);
    }
}

/// A mail standard or a bishop's mantle: an upright collar, then a cape.
pub(super) fn add_standard(model: &mut Model, form: &NeckForm, around: &Around) {
    let (nb, n) = (around.neck_base(), around.neck());
    let mantle = form.kind == NeckKind::Mantle;
    let high = around.chin() - if mantle { 30.0 } else { 15.0 };
    let low = nb + 5.0;
    let collar = tube(around, low, high, n + 12.0, SOFT_OFF);
    dress(
        model,
        &whole(around, &collar, Hem::Straight, SOFT_OFF),
        form.construction,
        false,
    );
    let fall = Fall {
        front: nb - if mantle { 110.0 } else { 70.0 },
        back: nb - if mantle { 100.0 } else { 60.0 },
        side_out: around.base_across() + 30.0,
        cape: form.cape,
        off: SOFT_OFF,
    };
    let top = |angle| collar(angle).bottom;
    let profile = |angle| fall.profile(around, angle, top(angle));
    let cape = whole(around, &profile, form.hem, SOFT_OFF);
    dress(model, &cape, form.construction, form.studded);
    add_edging(model, &cape, form.edging);
}

/// Where an aventail's top runs: round the face as its front says, and round the head
/// at the helmet's edge (`rim`, heights by angle) at the sides and back.
fn aventail_top(form: &NeckForm, around: &Around, rim: &dyn Fn(f32) -> Vec3, angle: f32) -> Vec3 {
    let chin = around.chin();
    let own = match form.front {
        Front::Closed => chin + 75.0,
        Front::Open => rim(angle).y,
        _ => chin + 30.0,
    };
    let from_front = angle.rem_euclid(TAU);
    let from_front = from_front.min(TAU - from_front);
    let t = ((from_front - PI / 6.0) / (PI / 3.0)).clamp(0.0, 1.0);
    let y = own + (rim(angle).y - own) * t;
    let on_head = around.head(angle, y, OFF_HEAD as f32);
    on_head.lerp(rim(angle), t)
}

/// An aventail hung from a helmet's edge (`rim`), down over the neck and shoulders.
pub(super) fn add_aventail(
    model: &mut Model,
    form: &NeckForm,
    around: &Around,
    rim: &dyn Fn(f32) -> Vec3,
) {
    let nb = around.neck_base();
    let fall = Fall {
        front: nb - 60.0,
        back: nb - 50.0,
        side_out: around.base_across() + 30.0,
        cape: form.cape,
        off: SOFT_OFF,
    };
    let profile = |angle| fall.profile(around, angle, aventail_top(form, around, rim, angle));
    let open = form.front == Front::Open;
    let curtain = Curtain {
        from: if open { 0.7 } else { -PI },
        to: if open { TAU - 0.7 } else { PI },
        profile: &profile,
        hem: form.hem,
        depth: HEM_DEPTH,
        clear: (around, SOFT_OFF),
    };
    dress(model, &curtain, form.construction, form.studded);
    add_edging(model, &curtain, form.edging);
    let surface = curtain.surface(0.0);
    if form.front == Front::Ventail {
        patch(
            model,
            Stuff::Main,
            &surface,
            (0.43, 0.57),
            (0.0, 0.3),
            3.0,
            2.0,
        );
    }
    // The leather band it laces on by, and the staples through it.
    if !matches!(
        form.construction,
        Construction::Padded | Construction::Leather
    ) {
        curtain.band(model, Stuff::Backing, (0.0, 0.06), 1.8, 2.0);
        let band = |u: f32, v: f32| surface(u, v * 0.06);
        studs(model, Stuff::Rivet, &band, (1, 14), 3.0, 2.2);
    }
}

/// A padded collar, or a Japanese one lined with small plates: a soft tube with a
/// short bib in front and behind.
pub(super) fn add_soft_collar(model: &mut Model, form: &NeckForm, around: &Around) {
    let (nb, n) = (around.neck_base(), around.neck());
    let off = 6.0;
    let collar = tube(around, nb + 5.0, around.chin() - 25.0, n + 9.0, off);
    let fall = Fall {
        front: nb - 40.0,
        back: nb - 30.0,
        side_out: around.base_across() + 25.0,
        cape: false,
        off,
    };
    let bib = |angle| fall.profile(around, angle, collar(angle).bottom);
    let kikko = form.construction == Construction::PlatesInCloth;
    let cloth = if kikko { Stuff::Cloth } else { Stuff::Main };
    for curtain in [
        whole(around, &collar, Hem::Straight, off),
        whole(around, &bib, Hem::Straight, off),
    ] {
        curtain.sheet(model, cloth, 0.0, 5.0);
        let surface = curtain.surface(0.0);
        if kikko {
            studs(model, Stuff::Main, &surface, (3, 20), 3.0, 3.2);
        } else {
            curtain.lines(model, 5, 3.0);
        }
    }
}
