//! Every piece of torso armour that may be made: the grid rolls keep to, checked by
//! the tests.

use super::tables::{
    VEST_MATERIALS, constructions, has, leather_breasts, materials, may_be_over_mail,
    may_lack_back, plate_breasts,
};
use super::{Breast, Fluting, Plackart, Quilting, TorsoBody, TorsoForm, Weave};
use crate::armour::{Construction, Finish, Style};
use crate::material::Material;

impl TorsoForm {
    /// Whether these choices go together.
    pub fn allowed(&self) -> bool {
        let style = self.style;
        let made_of = if self.vest {
            has(&VEST_MATERIALS, self.material)
        } else {
            has(&constructions(style), self.construction)
                && has(materials(self.construction, style), self.material)
        };
        made_of
            && self.body_fits()
            && (self.back || self.breast().is_some_and(may_lack_back))
            && (!self.over_mail || (self.rigid() && may_be_over_mail(style) && !self.vest))
            && (!self.lined || self.rigid())
            && self.finish_fits()
            && self.cover.is_some() == self.has_cover()
            && self.lacing.is_none()
            && self.trim.is_none_or(|trim| {
                trim == Material::Brass
                    && matches!(self.construction, Construction::Plate | Construction::Mail)
                    && self.material != Material::Bronze
            })
    }

    /// Cloth to colour: padding (not coir) and Japanese mail's backing.
    fn has_cover(&self) -> bool {
        match self.body {
            TorsoBody::Padded { .. } => self.material != Material::Coir,
            TorsoBody::Mail { weave, .. } => weave == Weave::Japanese,
            _ => false,
        }
    }

    fn finish_fits(&self) -> bool {
        match self.construction {
            Construction::Plate => true,
            Construction::Mail if self.material != Material::Bronze => matches!(
                self.finish,
                Finish::Bright | Finish::Blackened | Finish::Tinned | Finish::Gilt
            ),
            Construction::Leather if self.rigid() => {
                matches!(self.finish, Finish::Bright | Finish::Lacquered(_))
            }
            _ => self.finish == Finish::Bright,
        }
    }

    fn body_fits(&self) -> bool {
        let style = self.style;
        match self.body {
            TorsoBody::Plate {
                breast,
                plackart,
                ridge,
                fluting,
                lance_rest,
            } => {
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
                self.construction == Construction::Plate
                    && has(plate_breasts(style), breast)
                    && plackart_fits
                    && ridge_fits
                    && fluting_fits
                    && (!lance_rest || (style.european() && breast != Breast::Discs))
            }
            TorsoBody::Leather { breast, studded } => {
                self.construction == Construction::Leather
                    && if self.material == Material::BuffLeather {
                        breast == Breast::Round && !studded
                    } else {
                        has(leather_breasts(style), breast)
                    }
            }
            TorsoBody::Mail { weave, .. } => {
                self.construction == Construction::Mail
                    && (weave == Weave::Japanese) == (style == Style::Japanese)
            }
            TorsoBody::RingMail { .. } => self.construction == Construction::RingMail,
            TorsoBody::Padded { quilting, sleeves } => {
                let laminated = matches!(
                    (self.material, style),
                    (Material::Linen, Style::Classical) | (Material::Cotton, Style::Chinese)
                );
                let quilting_fits = match quilting {
                    _ if self.vest => {
                        !sleeves && matches!(quilting, Quilting::Channels | Quilting::Diamonds)
                    }
                    _ if self.material == Material::Coir => quilting == Quilting::Squares,
                    Quilting::Laminated => laminated,
                    Quilting::Channels => true,
                    _ => !laminated,
                };
                self.construction == Construction::Padded && quilting_fits
            }
        }
    }
}
