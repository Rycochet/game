//! Every piece of waist armour that may be made: the grid rolls keep to, checked by the
//! tests.

use super::WaistForm;
use super::counts::{culets, hangs, lames, openings};
use super::extras::{has_cloth, may_add_mail, may_be_front, may_guard_groin};
use super::materials::materials;
use super::tables::{constructions, has, lengths, pieces};
use crate::armour::{Construction, Finish, Style};
use crate::material::{Category, Material};

impl WaistForm {
    /// Whether these choices go together.
    pub fn allowed(&self) -> bool {
        let (kind, construction, style, piece) =
            (self.kind, self.construction, self.style, self.piece);
        let made = has(constructions(kind, style), construction)
            && has(pieces(kind, construction, style), piece)
            && has(lengths(kind, piece, construction, style), self.length)
            && has(
                lames(kind, piece, construction, style, self.length),
                self.lames,
            )
            && has(culets(piece, construction), self.culet)
            && has(hangs(kind, piece, style), self.hangs)
            && has(openings(piece, construction), self.opening)
            && has(materials(construction, style), self.material);
        made && (!self.front || may_be_front(piece, construction, style))
            && (!self.mail_beneath || may_add_mail(piece, construction, style))
            && (!self.groin || may_guard_groin(piece, construction, style))
            && (!self.studded
                || (construction == Construction::Leather && style != Style::Japanese))
            && self.finish_fits()
            && self.cloth.is_some() == has_cloth(piece, construction, style)
            && self.trim.is_none_or(|trim| {
                trim == Material::Brass
                    && matches!(construction, Construction::Plate | Construction::Mail)
                    && self.metal()
                    && style != Style::Japanese
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
            _ if self.material == Material::HardenedLeather => match self.finish {
                Finish::Lacquered(_) => true,
                Finish::Bright => self.style != Style::Japanese,
                _ => false,
            },
            _ => self.finish == Finish::Bright,
        }
    }
}
