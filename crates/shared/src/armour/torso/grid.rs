//! Every piece of torso armour that may be made: the grid rolls keep to, checked by
//! the tests. Each construction's own parts are checked in `grid_body`.

use super::TorsoForm;
use super::materials::materials;
use super::tables::{VEST_MATERIALS, constructions, has, may_lack_back};
use crate::material::Material;

impl TorsoForm {
    /// Whether these choices go together.
    pub fn allowed(&self) -> bool {
        let made_of = if self.vest {
            has(&VEST_MATERIALS, self.material)
        } else {
            has(&constructions(self.style), self.construction)
                && has(materials(self.construction, self.style), self.material)
        };
        made_of
            && self.body_fits()
            && (self.back || self.breast().is_some_and(may_lack_back))
            && (!self.over_mail || self.may_have_mail())
            && (!self.lined || self.may_be_lined())
            && (self.lined || !self.always_lined())
            && self.finish_fits()
            && self.cover.is_some() == self.has_cover()
            && self.lacing.is_some() == self.has_lacing()
            && self
                .trim
                .is_none_or(|trim| trim == Material::Brass && self.may_have_trim())
    }
}
