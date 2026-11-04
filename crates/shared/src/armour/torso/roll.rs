//! Rolling a piece of torso armour, each choice from what the ones before allow, so
//! nothing is thrown away.

use super::materials::materials;
use super::roll_body::roll_body;
use super::tables::{VEST_MATERIALS, constructions, may_lack_back};
use super::{Construction, Dye, Finish, Quilting, TorsoBody, TorsoForm, TorsoKind};
use crate::armour::Style;
use crate::item::Quality;
use crate::material::Material;
use crate::rng::Rng;

/// One in so many.
pub(super) fn one_in(rng: &mut Rng, chance: u64) -> bool {
    rng.range_u64(0, chance) == 0
}

/// Rolls a piece of a kind. A set passes its style; a piece on its own rolls one.
pub fn roll(rng: &mut Rng, kind: TorsoKind, style: Option<Style>, quality: Quality) -> TorsoForm {
    let style = style.unwrap_or_else(|| Style::roll(rng));
    if kind == TorsoKind::ArmingVest {
        return vest(rng, style);
    }
    let construction = rng.pick_weighted(&constructions(style));
    let material = rng.pick_weighted(materials(construction, style));
    let mut form = TorsoForm {
        body: roll_body(rng, construction, style, material),
        construction,
        style,
        material,
        back: true,
        over_mail: false,
        lined: false,
        vest: false,
        finish: Finish::Bright,
        cover: None,
        lacing: None,
        trim: None,
    };
    form.back = !form.breast().is_some_and(may_lack_back) || !one_in(rng, 4);
    form.over_mail = form.may_have_mail() && one_in(rng, 4);
    form.lined = form.always_lined() || (form.may_be_lined() && one_in(rng, 3));
    form.finish = roll_finish(rng, &form, quality);
    form.cover = form.has_cover().then(|| Dye::roll(rng));
    form.lacing = form.has_lacing().then(|| lacing(rng, style));
    form.trim = (form.may_have_trim() && one_in(rng, 5)).then_some(Material::Brass);
    form
}

/// Metal finished as its style has it, mail more plainly, small plates of leather or
/// horn and hard leather sometimes lacquered (mostly in the east).
fn roll_finish(rng: &mut Rng, form: &TorsoForm, quality: Quality) -> Finish {
    let eastern = matches!(form.style, Style::Japanese | Style::Chinese);
    if form.any_finish() {
        Finish::roll(rng, form.style, quality)
    } else if form.mail_finish() {
        let gilt = if quality >= Quality::Rare { 4 } else { 0 };
        rng.pick_weighted(&[
            (Finish::Bright, 70),
            (Finish::Blackened, 18),
            (Finish::Tinned, 8),
            (Finish::Gilt, gilt),
        ])
    } else if form.may_be_lacquered() && one_in(rng, if eastern { 2 } else { 4 }) {
        Finish::Lacquered(Dye::roll_paint(rng))
    } else {
        Finish::Bright
    }
}

/// Japanese lacing is coloured silk; elsewhere it is mostly plain leather or cord.
fn lacing(rng: &mut Rng, style: Style) -> Dye {
    if style == Style::Japanese {
        Dye::roll_paint(rng)
    } else if one_in(rng, 3) {
        Dye::roll(rng)
    } else {
        Dye::Natural
    }
}

/// A sleeveless padded vest, undyed the most often.
fn vest(rng: &mut Rng, style: Style) -> TorsoForm {
    let material = rng.pick_weighted(&VEST_MATERIALS);
    let cover = if one_in(rng, 2) {
        Dye::Natural
    } else {
        Dye::roll(rng)
    };
    TorsoForm {
        body: TorsoBody::Padded {
            quilting: rng.pick_weighted(&[(Quilting::Channels, 70), (Quilting::Diamonds, 30)]),
            sleeves: false,
        },
        construction: Construction::Padded,
        style,
        material,
        back: true,
        over_mail: false,
        lined: false,
        vest: true,
        finish: Finish::Bright,
        cover: Some(cover),
        lacing: None,
        trim: None,
    }
}
