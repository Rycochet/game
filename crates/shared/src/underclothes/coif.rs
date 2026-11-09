//! Linen coifs, undercaps, hairnets and nightcaps, on the head. The mail Coif is armour.

use serde::{Deserialize, Serialize};

use super::table::has;
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoifForm {
    /// Two or three panels with a seam over the crown, over the ears, tied under the chin.
    Coif,
    /// A close cap over the hair, without ties.
    Undercap,
    /// A net over the hair on a band: a hairnet, or a caul in gold thread.
    Net,
    /// Conical with a turned-up brim, often embroidered (16th century on).
    BrimmedNightcap,
    /// Long and pointed, its end worn as a scarf.
    PointedNightcap,
    /// A cloth tied under the chin.
    TiedNightcap,
}

impl CoifForm {
    pub fn nightcap(self) -> bool {
        matches!(
            self,
            Self::BrimmedNightcap | Self::PointedNightcap | Self::TiedNightcap
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Coif {
    pub form: CoifForm,
}

const FORMS: [(CoifForm, u64); 6] = [
    (CoifForm::Coif, 40),
    (CoifForm::Undercap, 15),
    (CoifForm::Net, 15),
    (CoifForm::BrimmedNightcap, 8),
    (CoifForm::PointedNightcap, 10),
    (CoifForm::TiedNightcap, 12),
];

impl Coif {
    pub(super) fn roll(rng: &mut Rng) -> Self {
        Self {
            form: rng.pick_weighted(&FORMS),
        }
    }

    pub(super) fn allowed(&self) -> bool {
        has(&FORMS, self.form)
    }

    /// Its name; a coif's always says its material ("Linen Coif", owner, 2026-11-09).
    pub(super) fn name(&self, gold: bool) -> &'static str {
        match self.form {
            CoifForm::Coif => "Coif",
            CoifForm::Undercap => "Undercap",
            CoifForm::Net if gold => "Caul",
            CoifForm::Net => "Hairnet",
            _ => "Nightcap",
        }
    }

    pub(super) fn base_grams(&self) -> u32 {
        match self.form {
            CoifForm::Coif => 40,
            CoifForm::Undercap => 30,
            CoifForm::Net => 15,
            CoifForm::BrimmedNightcap => 70,
            CoifForm::PointedNightcap => 60,
            CoifForm::TiedNightcap => 40,
        }
    }
}
