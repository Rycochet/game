//! Every piece of arm armour that may be made: the grid rolls keep to, checked by the
//! tests.

use super::tables::{
    always_closed, constructions, has, has_closure, has_couter, has_cover, materials,
    may_be_one_arm, reaches,
};
use super::{ArmsForm, Couter, Sides, Spikes};
use crate::armour::{Construction, Finish};
use crate::material::{Category, Material};

impl ArmsForm {
    /// Whether these choices go together.
    pub fn allowed(&self) -> bool {
        let (construction, style, reach) = (self.construction, self.style, self.reach);
        let made = has(constructions(style), construction)
            && has(reaches(construction, style), reach)
            && has(materials(construction, style, reach), self.material);
        let sides = self.sides == Sides::Both || may_be_one_arm(construction, style, reach);
        let couter = match self.couter {
            Couter::None => !has_couter(construction, style, reach),
            Couter::Cup => has_couter(construction, style, reach),
            Couter::Winged => {
                has_couter(construction, style, reach) && construction == Construction::Plate
            }
        };
        let closed = if !has_closure(construction, reach) {
            !self.closed
        } else {
            self.closed || !always_closed(style)
        };
        let spikes = match self.spikes {
            Spikes::None => true,
            _ if !matches!(construction, Construction::Plate | Construction::Leather) => false,
            Spikes::Elbow => self.couter != Couter::None,
            Spikes::Ridge => reach.forearm(),
        };
        made && sides
            && couter
            && closed
            && spikes
            && (!self.studded || construction == Construction::Leather)
            && self.finish_fits()
            && self.cover.is_some() == has_cover(construction, style)
            && self.trim.is_none_or(|trim| {
                trim == Material::Brass
                    && matches!(construction, Construction::Plate | Construction::Mail)
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
