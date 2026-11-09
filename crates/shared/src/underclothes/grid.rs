//! Every piece of underclothing that may be made: the grid rolls keep to, checked by
//! the tests.

use super::UnderForm;
use super::cloth::{cloths, contrasts, decorations, dyes, materials};
use super::roll::two_coloured;
use super::table::has;

impl UnderForm {
    /// Whether these choices go together. Gold thread is allowed at any quality, though
    /// only rolled at Rare and better.
    pub fn allowed(&self) -> bool {
        let garment = &self.garment;
        let material = self.material;
        let coloured = match self.contrast {
            Some(second) => two_coloured(garment) && has(&contrasts(self.dye), second),
            None => !two_coloured(garment),
        };
        garment.allowed()
            && has(&materials(garment), material)
            && has(&cloths(garment, material), self.cloth)
            && has(&decorations(garment, material), self.decoration)
            && has(&dyes(garment, material), self.dye)
            && coloured
    }
}
