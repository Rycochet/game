//! Every head piece that may be made: the grid rolls keep to, checked by the tests.

use super::materials::{COIFS, has, materials};
use super::roll::{bowl_fits, constructions, ornament_fits, rows_for};
use super::table::{ROWS, Row};
use super::{Back, Bowl, Brim, Face, HeadForm, HeadKind, Ornament, Skull, Top};
use crate::armour::{Construction, Finish, Style, in_style};
use crate::material::{Category, Material};

impl HeadForm {
    /// Whether these choices go together.
    pub fn allowed(&self) -> bool {
        has(materials(self.construction, self.style), self.material)
            && match self.kind {
                HeadKind::Cap | HeadKind::Coif => self.soft_fits(),
                HeadKind::Helmet | HeadKind::BevoredHelm => {
                    rows_for(self.kind, self.style)
                        .iter()
                        .any(|&(at, _)| self.fits_row(&ROWS[at], Some(self.style)))
                        && self.extras_fit()
                }
            }
    }

    /// Whether its shape is one a row allows, in this style or (with `None`) any.
    pub(super) fn fits_row(&self, row: &Row, style: Option<Style>) -> bool {
        let made = match style {
            Some(style) => constructions(row, style).contains(&self.construction),
            None => row.constructions.contains(&self.construction),
        };
        made && row.bevor == (self.kind == HeadKind::BevoredHelm)
            && row.bowls.contains(&self.bowl)
            && bowl_fits(self.construction, self.bowl)
            && row.skulls.contains(&self.skull)
            && row.faces.contains(&self.face)
            && row.cheeks.fits(self.cheeks)
            && row.backs.contains(&self.back)
            && row.brims.contains(&self.brim)
            && row.tops.contains(&self.top)
    }

    /// Ornament, studs, finish and colours, beyond its row.
    fn extras_fit(&self) -> bool {
        let ornament_made = match self.style {
            Style::Japanese => matches!(self.ornament, Ornament::None | Ornament::FrontCrest),
            Style::Elven => matches!(
                self.ornament,
                Ornament::None | Ornament::Wings | Ornament::Crown
            ),
            Style::Dwarven => matches!(
                self.ornament,
                Ornament::None | Ornament::Horns | Ornament::Crown
            ),
            style if style.european() => self.ornament != Ornament::FrontCrest,
            _ => self.ornament == Ornament::None,
        };
        let crest = matches!(self.top, Top::Crest | Top::Plume);
        ornament_made
            && ornament_fits(self.ornament, self.skull, self.top)
            && (!self.studded || (self.construction == Construction::Leather && !self.tusks()))
            && self.finish_fits()
            && self.cover.is_some() == crest
            && self.lacing.is_some() == self.laced()
    }

    fn finish_fits(&self) -> bool {
        let eastern = matches!(self.style, Style::Japanese | Style::Chinese);
        match self.material {
            Material::Bronze | Material::Copper => self.finish == Finish::Bright,
            material if material.category() == Category::Metal => {
                if eastern {
                    matches!(
                        self.finish,
                        Finish::Lacquered(_) | Finish::Russet | Finish::Gilt
                    )
                } else {
                    !matches!(self.finish, Finish::Lacquered(_))
                }
            }
            Material::HardenedLeather | Material::Leather => match self.finish {
                Finish::Lacquered(_) => true,
                finish => finish == Finish::Bright && self.style != Style::Japanese,
            },
            _ => self.finish == Finish::Bright,
        }
    }

    /// A cap or coif: no bowl, open, nothing added.
    fn soft_fits(&self) -> bool {
        let made = match self.kind {
            HeadKind::Cap => self.construction == Construction::Padded,
            _ => has(&in_style(&COIFS, self.style), self.construction),
        };
        let padded = self.construction == Construction::Padded;
        let finish = match self.construction {
            Construction::Mail if self.material != Material::Bronze => matches!(
                self.finish,
                Finish::Bright | Finish::Blackened | Finish::Tinned
            ),
            _ => self.finish == Finish::Bright,
        };
        made && finish
            && self.bowl == Bowl::Soft
            && self.skull == Skull::Round
            && self.face == Face::Open
            && !self.cheeks
            && self.back == Back::None
            && self.brim == Brim::None
            && self.top == Top::None
            && self.ornament == Ornament::None
            && !self.studded
            && self.cover.is_some() == padded
            && self.lacing.is_none()
    }
}
