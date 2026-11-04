//! Every piece of leg armour that may be made: the grid rolls keep to, checked by the
//! tests.

use super::materials::materials;
use super::tables::{
    Wrap, constructions, covers_for, has, has_cloth, knees, may_be_one_leg, may_ornament,
    may_reinforce, wrap,
};
use super::{Knee, LegsForm, Ornament, Reinforce};
use crate::armour::arms::Sides;
use crate::armour::{Construction, Finish, Style};
use crate::material::{Category, Material};

impl LegsForm {
    /// Whether these choices go together.
    pub fn allowed(&self) -> bool {
        let (kind, construction, style, cover) =
            (self.kind, self.construction, self.style, self.cover);
        let made = has(constructions(kind, style), construction)
            && has(covers_for(kind, construction, style), cover)
            && has(materials(construction, style), self.material)
            && has(knees(kind, construction, style, cover), self.knee);
        let closed = match wrap(construction, style, cover) {
            Wrap::Closed => self.closed,
            Wrap::Front => !self.closed,
            Wrap::Either => true,
        };
        let reinforce =
            self.reinforce == Reinforce::None || may_reinforce(construction, style, cover);
        let sides = self.sides == Sides::Both || may_be_one_leg(construction, style, cover);
        let ornament = match self.ornament {
            Ornament::None => true,
            _ if !may_ornament(construction, style) => false,
            Ornament::KneeSpikes => matches!(self.knee, Knee::Cop | Knee::Winged),
            Ornament::ShinFins => cover.shin(),
        };
        let cloth = has_cloth(construction, style) || self.reinforce == Reinforce::QuiltedThighs;
        made && closed
            && reinforce
            && sides
            && ornament
            && (!self.studded || construction == Construction::Leather)
            && self.finish_fits()
            && self.cloth.is_some() == cloth
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
            _ if self.material == Material::HardenedLeather => {
                matches!(self.finish, Finish::Bright | Finish::Lacquered(_))
            }
            _ => self.finish == Finish::Bright,
        }
    }
}
