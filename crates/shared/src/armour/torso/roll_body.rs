//! Rolling what plate, leather, mail, ring mail and padding have of their own; the
//! constructions of many small plates are in `roll_plates`.

use super::roll::one_in;
use super::roll_plates::roll_plates;
use super::tables::{leather_breasts, plate_breasts, soft_leather};
use super::{Breast, Fluting, Links, Plackart, Quilting, Rings, TorsoBody, Weave};
use crate::armour::{Construction, Style};
use crate::material::Material;
use crate::rng::Rng;

pub(super) fn roll_body(
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
            let soft = soft_leather(material);
            let breast = if soft {
                Breast::Round
            } else {
                rng.pick_weighted(leather_breasts(style))
            };
            let studded = !soft && one_in(rng, 3);
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
        Construction::Padded => TorsoBody::Padded {
            quilting: padded_quilting(rng, style, material),
            sleeves: one_in(rng, 3),
        },
        _ => roll_plates(rng, construction, style, material),
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
