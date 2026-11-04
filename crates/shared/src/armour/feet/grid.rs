//! Every pair of foot pieces that may be made: the grid rolls keep to, checked by the
//! tests.

use super::materials::materials;
use super::tables::{constructions, has, has_cloth, lames, may_be_high, may_detach, pieces, toes};
use super::{DETACHABLE_POINT_MM, FeetForm, FeetPiece, POINT_MM, Toe};
use crate::armour::{Construction, Finish, Style};
use crate::material::{Category, Material};

impl FeetForm {
    /// Whether these choices go together.
    pub fn allowed(&self) -> bool {
        let (piece, construction, style, toe) =
            (self.piece, self.construction, self.style, self.toe);
        let sabatons = piece == FeetPiece::Sabatons;
        let made = has(constructions(style), construction)
            && has(pieces(construction, style), piece)
            && has(&toes(piece, construction, style), toe)
            && has(materials(construction, style), self.material);
        let point = match toe {
            Toe::Pointed if self.detachable => within(self.point, DETACHABLE_POINT_MM),
            Toe::Pointed if style == Style::Elven => within(self.point, (30, 60)),
            Toe::Pointed => within(self.point, POINT_MM),
            _ => self.point == 0,
        };
        let (narrow, wide) = toe.widths();
        let lames = if sabatons {
            let (least, most) = lames(construction, style, toe);
            (least..=most).contains(&self.lames)
        } else {
            self.lames == 0
        };
        made && point
            && (narrow..=wide).contains(&self.width)
            && lames
            && (!self.detachable || may_detach(construction, style, toe))
            && (!self.heel || sabatons)
            && (!self.high || may_be_high(piece, construction, toe))
            && self.fluted
                == (sabatons && construction == Construction::Plate && style == Style::Maximilian)
            && (piece == FeetPiece::FootGuards || !self.toes)
            && (style != Style::Classical || piece != FeetPiece::FootGuards || self.toes)
            && (piece == FeetPiece::MailShoes || !self.soled)
            && (construction != Construction::RingMail || self.soled)
            && (!self.studded || construction == Construction::Leather)
            && self.finish_fits()
            && self.cloth.is_some() == has_cloth(style)
            && self.trim.is_none_or(|trim| {
                trim == Material::Brass
                    && sabatons
                    && construction == Construction::Plate
                    && self.metal()
            })
    }

    /// Of a metal that takes finishes: bronze is left as it is.
    fn metal(&self) -> bool {
        self.material.category() == Category::Metal && self.material != Material::Bronze
    }

    fn finish_fits(&self) -> bool {
        match self.construction {
            Construction::Mail | Construction::RingMail if self.metal() => matches!(
                self.finish,
                Finish::Bright | Finish::Blackened | Finish::Tinned | Finish::Gilt
            ),
            _ if self.metal() => true,
            _ if self.material == Material::HardenedLeather => {
                matches!(self.finish, Finish::Bright | Finish::Lacquered(_))
            }
            _ => self.finish == Finish::Bright,
        }
    }
}

fn within(value: u16, (least, most): (u16, u16)) -> bool {
    (least..=most).contains(&value)
}
