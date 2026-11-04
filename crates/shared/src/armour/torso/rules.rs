//! What a piece may have beyond its construction's own parts, used by both rolling and
//! the grid so the two agree: mail beneath, a lining, cloth and lacing to colour, brass
//! edges, and which finishes suit it.

use super::tables::may_be_over_mail;
use super::{Rivets, TorsoBody, TorsoForm, Weave};
use crate::armour::{Construction, Finish};
use crate::material::{Category, Material};

impl TorsoForm {
    /// Rigid pieces and plates in cloth may be worn over their own mail shirt.
    pub(super) fn may_have_mail(&self) -> bool {
        let fits = self.rigid() || self.construction == Construction::PlatesInCloth;
        fits && !self.vest && may_be_over_mail(self.style)
    }

    /// Rigid pieces and plates in cloth may be lined.
    pub(super) fn may_be_lined(&self) -> bool {
        self.rigid() || self.construction == Construction::PlatesInCloth
    }

    /// A jack's plates are stitched between layers of padding.
    pub(super) fn always_lined(&self) -> bool {
        matches!(
            self.body,
            TorsoBody::PlatesInCloth {
                rivets: Rivets::Hidden,
                ..
            }
        )
    }

    /// Cloth to colour: padding (not coir), Japanese mail's backing and a cloth cover.
    pub(super) fn has_cover(&self) -> bool {
        match self.body {
            TorsoBody::Padded { .. } => self.material != Material::Coir,
            TorsoBody::Mail { weave, .. } => weave == Weave::Japanese,
            TorsoBody::PlatesInCloth { leather, .. } => !leather,
            _ => false,
        }
    }

    /// Lacing to colour: lamellae, laced lames and slats.
    pub(super) fn has_lacing(&self) -> bool {
        matches!(
            self.body,
            TorsoBody::Lamellar { .. }
                | TorsoBody::Laminar { riveted: false, .. }
                | TorsoBody::Splint { .. }
        )
    }

    /// Brass edges on metal plates, mail, lames, plated mail and mirrors, not bronze.
    pub(super) fn may_have_trim(&self) -> bool {
        use Construction as C;
        let edged = matches!(
            self.construction,
            C::Plate | C::Mail | C::Laminar | C::MailAndPlate | C::Mirror
        );
        edged && self.material.category() == Category::Metal && self.material != Material::Bronze
    }

    /// Small plates of leather, horn or bone, and rigid leather: lacquered or not.
    pub(super) fn may_be_lacquered(&self) -> bool {
        use Construction as C;
        let small_plates = matches!(self.construction, C::Scale | C::Lamellar | C::Laminar);
        let not_metal = self.material.category() != Category::Metal;
        (small_plates && not_metal) || (self.construction == C::Leather && self.rigid())
    }

    /// Metal that shows and takes any finish: plate, scales, lamellae and lames.
    pub(super) fn any_finish(&self) -> bool {
        use Construction as C;
        let shows = matches!(
            self.construction,
            C::Plate | C::Scale | C::Lamellar | C::Laminar
        );
        shows && self.material.category() == Category::Metal
    }

    /// Mail, and the mail under plated mail and mirrors: bright, blackened, tinned or
    /// gilt (not bronze, which stays bright).
    pub(super) fn mail_finish(&self) -> bool {
        use Construction as C;
        matches!(self.construction, C::Mail | C::MailAndPlate | C::Mirror)
            && self.material != Material::Bronze
    }

    pub(super) fn finish_fits(&self) -> bool {
        if self.any_finish() {
            true
        } else if self.mail_finish() {
            matches!(
                self.finish,
                Finish::Bright | Finish::Blackened | Finish::Tinned | Finish::Gilt
            )
        } else if self.may_be_lacquered() {
            matches!(self.finish, Finish::Bright | Finish::Lacquered(_))
        } else {
            self.finish == Finish::Bright
        }
    }
}
