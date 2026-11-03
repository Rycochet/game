//! Which choices go together on a piece for the hands.

use super::{
    HandsForm, HandsPiece, constructions, cuffs_for, fingers_for, knuckles_for, materials_for,
    may_have_claws, may_have_trim, may_lock, needs_cloth,
};
use crate::armour::{Construction, in_style};

impl HandsForm {
    /// Every hand piece with these choices: the grid a roll keeps to.
    pub fn allowed(&self, piece: HandsPiece) -> bool {
        let in_list =
            |list: &[(Construction, u64)]| list.iter().any(|&(c, _)| c == self.construction);
        in_list(&in_style(constructions(piece), self.style))
            && fingers_for(self.construction, piece, self.style)
                .iter()
                .any(|&(f, _)| f == self.fingers)
            && cuffs_for(self.construction, self.fingers, self.style)
                .iter()
                .any(|&(c, _)| c == self.cuff)
            && knuckles_for(self.construction)
                .iter()
                .any(|&(k, _)| k == self.knuckles)
            && (!self.claws || may_have_claws(self.construction, self.fingers))
            && (!self.locking || may_lock(self.construction, self.fingers))
            && materials_for(self.construction)
                .iter()
                .any(|&(m, _)| m == self.material)
            && self.cloth.is_some() == needs_cloth(self.construction, self.fingers)
            && (self.trim.is_none() || may_have_trim(self.construction, self.material))
    }
}
