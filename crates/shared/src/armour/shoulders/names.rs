//! Names for shoulder armour, generic and specific.

use super::{Adornment, CapeCut, Cut, Finish, ShouldersForm, ShouldersKind, ShouldersPiece, Span};
use crate::armour::{Construction, Style};

impl ShouldersForm {
    /// What anything like it is called.
    pub fn generic_name(&self) -> &'static str {
        match self.kind() {
            ShouldersKind::Pauldrons => "Pauldrons",
            ShouldersKind::Spaulders => "Spaulders",
            ShouldersKind::Guards => "Shoulder Guards",
            ShouldersKind::Cape => "Shoulder Cape",
            ShouldersKind::Discs => "Shoulder Discs",
            ShouldersKind::Ailettes => "Ailettes",
            ShouldersKind::Wing => "Wing",
        }
    }

    /// Its own name, when it fits one: Japanese, Roman, Greek, Russian and Bronze Age
    /// pieces by theirs (the owner's choice), others by what makes them stand out.
    pub fn specific_name(&self) -> Option<&'static str> {
        use Construction as C;
        Some(match (self.piece, self.construction) {
            (ShouldersPiece::Guards { size, .. }, _) if self.japanese() => match size {
                Span::Small => "Ko-sode",
                Span::Medium => "Chū-sode",
                Span::Large => "Ō-sode",
            },
            (ShouldersPiece::Pauldrons { .. }, _) if self.adornment == Adornment::Skull => {
                "Skull Pauldrons"
            }
            (ShouldersPiece::Pauldrons { .. }, _)
                if matches!(self.adornment, Adornment::Spikes(_)) =>
            {
                "Spiked Pauldrons"
            }
            (
                ShouldersPiece::Pauldrons {
                    cut: Cut::GreatLeft,
                    ..
                },
                _,
            ) => "Great Pauldrons",
            (ShouldersPiece::Pauldrons { .. }, C::Leather) if self.finish == Finish::Studded => {
                "Studded Leather Pauldrons"
            }
            (ShouldersPiece::Pauldrons { .. }, C::Leather) => "Leather Pauldrons",
            (ShouldersPiece::Pauldrons { .. }, C::Scale) => "Scale Pauldrons",
            (ShouldersPiece::Pauldrons { .. }, C::MailAndPlate) => "Plated Mail Pauldrons",
            (ShouldersPiece::Pauldrons { .. }, C::Plate) if self.style == Style::Gothic => {
                "Gothic Pauldrons"
            }
            (ShouldersPiece::Spaulders { .. }, _) if self.dendra() => "Dendra Shoulder Guards",
            (ShouldersPiece::Spaulders { .. }, _) if self.roman() => "Lorica Shoulder Guards",
            (ShouldersPiece::Spaulders { .. }, C::Leather) => "Leather Spaulders",
            (ShouldersPiece::Guards { .. }, C::Lamellar) => "Lamellar Shoulder Guards",
            (ShouldersPiece::Guards { .. }, C::Scale) => "Scale Shoulder Guards",
            (ShouldersPiece::Guards { .. }, C::PlatesInCloth) => "Brigandine Shoulder Guards",
            (ShouldersPiece::Guards { .. }, C::Leather) if self.finish == Finish::RingMail => {
                "Ring Mail Shoulder Guards"
            }
            (ShouldersPiece::Guards { .. }, C::Leather) => "Leather Shoulder Guards",
            (
                ShouldersPiece::Cape {
                    cut: CapeCut::Yoke, ..
                },
                _,
            ) => "Linothorax Yoke",
            (ShouldersPiece::Cape { .. }, C::Mail) if self.style == Style::Classical => {
                "Mail Doubling"
            }
            (ShouldersPiece::Cape { .. }, C::Scale) if self.style == Style::Classical => {
                "Scale Doubling"
            }
            (ShouldersPiece::Cape { .. }, C::Mail) => "Mail Cape",
            (ShouldersPiece::Cape { .. }, C::Scale) => "Scale Cape",
            (ShouldersPiece::Cape { .. }, _) => "Ring Mail Cape",
            (ShouldersPiece::Discs { .. }, C::MailAndPlate) => "Naplechniki",
            (ShouldersPiece::Discs { .. }, C::Mirror) => "Mirror Shoulder Plates",
            (ShouldersPiece::Wing, _) => "Chukchi Wing",
            _ => return None,
        })
    }

    /// What it is called.
    pub fn name(&self) -> &'static str {
        self.specific_name().unwrap_or(self.generic_name())
    }
}
