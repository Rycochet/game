//! Whether a construction's own parts go together, by the same lists rolls use.

use super::roll_plates::{
    LAMES, backings, cloth_sizes, lacings, lamellar_cuts, lamellar_sizes, laminar_riveting,
    may_have_leather_cover, mirrors, plate_rows, rivets, scale_shapes, scale_sizes,
};
use super::tables::{has, leather_breasts, plate_breasts, soft_leather};
use super::{Breast, Fluting, Plackart, Quilting, TorsoBody, TorsoForm, Weave};
use crate::armour::{Construction, Style};
use crate::material::{Category, Material};

impl TorsoForm {
    pub(super) fn body_fits(&self) -> bool {
        use Construction as C;
        let style = self.style;
        let (wanted, fits) = match self.body {
            TorsoBody::Plate {
                breast,
                plackart,
                ridge,
                fluting,
                lance_rest,
            } => (
                C::Plate,
                plate_fits(style, breast, plackart, ridge, fluting, lance_rest),
            ),
            TorsoBody::Leather { breast, studded } => {
                let fits = if soft_leather(self.material) {
                    breast == Breast::Round && !studded
                } else {
                    has(leather_breasts(style), breast)
                };
                (C::Leather, fits)
            }
            TorsoBody::Mail { weave, .. } => (
                C::Mail,
                (weave == Weave::Japanese) == (style == Style::Japanese),
            ),
            TorsoBody::RingMail { .. } => (C::RingMail, true),
            TorsoBody::Padded { quilting, sleeves } => {
                (C::Padded, self.quilting_fits(quilting, sleeves))
            }
            TorsoBody::Scale {
                shape,
                size,
                backing,
                ..
            } => (
                C::Scale,
                has(scale_shapes(style, self.material), shape)
                    && has(scale_sizes(style), size)
                    && has(backings(style, self.material), backing),
            ),
            TorsoBody::Lamellar { cut, size, lacing } => (
                C::Lamellar,
                has(lamellar_cuts(style), cut)
                    && has(lamellar_sizes(style), size)
                    && has(lacings(style), lacing),
            ),
            TorsoBody::Laminar { lames, riveted } => (
                C::Laminar,
                LAMES.contains(&lames) && has(laminar_riveting(style), riveted),
            ),
            TorsoBody::PlatesInCloth {
                size,
                rivets: on,
                leather,
            } => (
                C::PlatesInCloth,
                has(cloth_sizes(style), size)
                    && has(rivets(style, size), on)
                    && (!leather || may_have_leather_cover(style, on)),
            ),
            TorsoBody::MailAndPlate { rows, .. } => (C::MailAndPlate, has(plate_rows(style), rows)),
            TorsoBody::Mirror { mirrors: kind } => (C::Mirror, has(mirrors(style), kind)),
            TorsoBody::Splint { rods } => (
                C::Splint,
                !rods || self.material.category() != Category::Metal,
            ),
            TorsoBody::Woven => (C::Woven, true),
        };
        self.construction == wanted && fits
    }

    fn quilting_fits(&self, quilting: Quilting, sleeves: bool) -> bool {
        let laminated = matches!(
            (self.material, self.style),
            (Material::Linen, Style::Classical) | (Material::Cotton, Style::Chinese)
        );
        match quilting {
            _ if self.vest => {
                !sleeves && matches!(quilting, Quilting::Channels | Quilting::Diamonds)
            }
            _ if self.material == Material::Coir => quilting == Quilting::Squares,
            Quilting::Laminated => laminated,
            Quilting::Channels => true,
            _ => !laminated,
        }
    }
}

/// Plate's parts, as the breast and style allow them.
fn plate_fits(
    style: Style,
    breast: Breast,
    plackart: Plackart,
    ridge: bool,
    fluting: Fluting,
    lance_rest: bool,
) -> bool {
    let plackart_fits = match breast {
        Breast::Gothic => plackart == Plackart::Pointed,
        Breast::Globose => plackart != Plackart::Pointed,
        Breast::Round if style.european() => plackart != Plackart::Pointed,
        _ => plackart == Plackart::None,
    };
    let ridge_fits = match breast {
        Breast::Gothic | Breast::Peascod | Breast::Munition => ridge,
        Breast::Round | Breast::Box => true,
        _ => !ridge,
    };
    let fluting_fits = fluting
        == match (breast, style) {
            (Breast::Gothic, _) => Fluting::Ridges,
            (_, Style::Maximilian) => Fluting::All,
            _ => Fluting::None,
        };
    has(plate_breasts(style), breast)
        && plackart_fits
        && ridge_fits
        && fluting_fits
        && (!lance_rest || (style.european() && breast != Breast::Discs))
}
