//! Rolling a piece of torso armour, each choice from what the ones before allow, so
//! nothing is thrown away.

use super::tables::{
    VEST_MATERIALS, constructions, leather_breasts, materials, may_be_over_mail, may_lack_back,
    plate_breasts,
};
use super::{
    Breast, Fluting, Links, Plackart, Quilting, Rings, TorsoBody, TorsoForm, TorsoKind, Weave,
};
use crate::armour::{Construction, Dye, Finish, Style};
use crate::item::Quality;
use crate::material::Material;
use crate::rng::Rng;

/// One in so many.
fn one_in(rng: &mut Rng, chance: u64) -> bool {
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
    let body = roll_body(rng, construction, style, material);
    let breast = match body {
        TorsoBody::Plate { breast, .. } | TorsoBody::Leather { breast, .. } => Some(breast),
        _ => None,
    };
    let rigid = matches!(body, TorsoBody::Plate { .. })
        || (construction == Construction::Leather && material != Material::BuffLeather);
    let back = !breast.is_some_and(may_lack_back) || !one_in(rng, 4);
    let over_mail = rigid && may_be_over_mail(style) && one_in(rng, 4);
    let lined = rigid && one_in(rng, 3);
    let finish = match construction {
        Construction::Plate => Finish::roll(rng, style, quality),
        Construction::Mail if material != Material::Bronze => mail_finish(rng, quality),
        Construction::Leather if rigid && one_in(rng, 4) => Finish::Lacquered(Dye::roll_paint(rng)),
        _ => Finish::Bright,
    };
    let japanese_mail = matches!(
        body,
        TorsoBody::Mail {
            weave: Weave::Japanese,
            ..
        }
    );
    let cover = (construction == Construction::Padded && material != Material::Coir
        || japanese_mail)
        .then(|| Dye::roll(rng));
    let trim = (matches!(construction, Construction::Plate | Construction::Mail)
        && material != Material::Bronze
        && one_in(rng, 5))
    .then_some(Material::Brass);
    TorsoForm {
        body,
        construction,
        style,
        material,
        back,
        over_mail,
        lined,
        vest: false,
        finish,
        cover,
        lacing: None,
        trim,
    }
}

/// Mail is left bright, blackened, tinned, or gilt when fine.
fn mail_finish(rng: &mut Rng, quality: Quality) -> Finish {
    let gilt = if quality >= Quality::Rare { 4 } else { 0 };
    rng.pick_weighted(&[
        (Finish::Bright, 70),
        (Finish::Blackened, 18),
        (Finish::Tinned, 8),
        (Finish::Gilt, gilt),
    ])
}

fn roll_body(
    rng: &mut Rng,
    construction: Construction,
    style: Style,
    material: Material,
) -> TorsoBody {
    match construction {
        Construction::Plate => {
            let breast = rng.pick_weighted(plate_breasts(style));
            plate(rng, breast, style)
        }
        Construction::Leather => {
            let breast = if material == Material::BuffLeather {
                Breast::Round
            } else {
                rng.pick_weighted(leather_breasts(style))
            };
            let studded = material != Material::BuffLeather && one_in(rng, 3);
            TorsoBody::Leather { breast, studded }
        }
        Construction::Mail => TorsoBody::Mail {
            links: rng.pick_weighted(&[
                (Links::Riveted, 50),
                (Links::RivetedSolid, 25),
                (Links::Butted, 15),
                (Links::Welded, 10),
            ]),
            weave: if style == Style::Japanese {
                Weave::Japanese
            } else {
                rng.pick_weighted(&[(Weave::FourInOne, 85), (Weave::SixInOne, 15)])
            },
            rings: rng.pick_weighted(&[
                (Rings::Fine, 25),
                (Rings::Medium, 50),
                (Rings::Coarse, 25),
            ]),
            sleeves: !one_in(rng, 3),
        },
        Construction::RingMail => TorsoBody::RingMail {
            sleeves: one_in(rng, 2),
        },
        _ => TorsoBody::Padded {
            quilting: padded_quilting(rng, style, material),
            sleeves: one_in(rng, 3),
        },
    }
}

/// Plate's parts, from the breast's shape and the style.
fn plate(rng: &mut Rng, breast: Breast, style: Style) -> TorsoBody {
    let plackart = match breast {
        Breast::Gothic => Plackart::Pointed,
        Breast::Globose => rng.pick_weighted(&[(Plackart::Half, 70), (Plackart::None, 30)]),
        Breast::Round if style.european() => {
            rng.pick_weighted(&[(Plackart::None, 75), (Plackart::Half, 25)])
        }
        _ => Plackart::None,
    };
    let ridge = match breast {
        Breast::Gothic | Breast::Peascod | Breast::Munition => true,
        Breast::Round | Breast::Box => one_in(rng, 2),
        _ => false,
    };
    let fluting = match (breast, style) {
        (Breast::Gothic, _) => Fluting::Ridges,
        (_, Style::Maximilian) => Fluting::All,
        _ => Fluting::None,
    };
    let lance_rest = style.european() && breast != Breast::Discs && one_in(rng, 3);
    TorsoBody::Plate {
        breast,
        plackart,
        ridge,
        fluting,
        lance_rest,
    }
}

/// Laminated linen is Greek and layered cotton Korean-like (Chinese style); coir is
/// knotted in squares.
fn padded_quilting(rng: &mut Rng, style: Style, material: Material) -> Quilting {
    match (material, style) {
        (Material::Coir, _) => Quilting::Squares,
        (Material::Linen, Style::Classical) | (Material::Cotton, Style::Chinese) => {
            rng.pick_weighted(&[(Quilting::Laminated, 70), (Quilting::Channels, 30)])
        }
        _ => rng.pick_weighted(&[
            (Quilting::Channels, 45),
            (Quilting::Diamonds, 30),
            (Quilting::Squares, 25),
        ]),
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
