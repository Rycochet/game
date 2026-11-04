//! Names for pieces for the hands, generic and specific.

use super::{Cuff, Fingers, HandsForm, Knuckles};
use crate::armour::{Construction, Style};

impl HandsForm {
    /// What anything like it is called.
    pub fn generic_name(&self) -> &'static str {
        match (self.construction, self.fingers) {
            (Construction::Padded, _) => "Padded Gloves",
            (_, Fingers::Fingered) => "Gauntlets",
            (_, Fingers::Mitten) => "Mitten Gauntlets",
            (_, Fingers::Open) => "Demi-gauntlets",
        }
    }

    /// Its own name, when it fits one.
    pub fn specific_name(&self) -> Option<&'static str> {
        use Construction as C;
        let plate = self.construction == C::Plate;
        let metal = self.material.category() == crate::material::Category::Metal;
        Some(match (self.construction, self.fingers) {
            _ if self.claws => "Clawed Gauntlets",
            (C::Leather, _) if self.knuckles == Knuckles::Spiked => "Studded Leather Gauntlets",
            _ if self.knuckles == Knuckles::Spiked && metal => "Spiked Gauntlets",
            (C::Plate, Fingers::Mitten) if self.locking => "Locking Gauntlets",
            (C::Plate, Fingers::Mitten)
                if self.style == Style::Kastenbrust && self.cuff == Cuff::Long =>
            {
                "Kastenbrust Gauntlets"
            }
            _ if plate && self.style == Style::Gothic && self.fingers != Fingers::Open => {
                "Gothic Gauntlets"
            }
            (C::Plate, Fingers::Fingered) if self.cuff == Cuff::Hourglass => "Hourglass Gauntlets",
            (C::PlatesInCloth, _) => "Brigandine Gauntlets",
            (C::Mail, Fingers::Mitten) => "Mail Mittens",
            (C::Mail, _) => "Mail Gloves",
            (C::RingMail, Fingers::Mitten) => "Ring Mail Mittens",
            (C::RingMail, _) => "Ring Mail Gloves",
            (C::Scale, Fingers::Mitten) => "Scale Mittens",
            (C::Scale, _) => "Scale Gloves",
            (C::MailAndPlate, Fingers::Open) => "Tekkō",
            (C::MailAndPlate, _) => "Mail-and-plate Gauntlets",
            (C::Leather, _) => "Leather Gauntlets",
            _ => return None,
        })
    }

    /// What it is called.
    pub fn name(&self) -> &'static str {
        self.specific_name().unwrap_or(self.generic_name())
    }
}
