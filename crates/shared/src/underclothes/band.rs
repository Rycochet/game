//! Breast bands: a band wound or tied round the chest, or a panel on ties.

use serde::{Deserialize, Serialize};

use super::table::has;
use crate::material::Material;
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BandForm {
    /// A long band wound flat round the chest, its end tucked in (fascia).
    Wound,
    /// Wound and twisted (strophium).
    Twisted,
    /// Wide, wound from the chest to the waist (sarashi).
    WideWrap,
    /// A band tied at the back (breast girdle, bandeau).
    Tied,
    /// A panel over the front alone, tied round the neck and back (dudou).
    Panel,
    /// A band with shoulder straps and shaped cups (the Lengberg finds, 15th century).
    Strapped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Band {
    pub form: BandForm,
}

const FORMS: [(BandForm, u64); 6] = [
    (BandForm::Wound, 25),
    (BandForm::Twisted, 15),
    (BandForm::WideWrap, 15),
    (BandForm::Tied, 25),
    (BandForm::Panel, 12),
    (BandForm::Strapped, 8),
];

impl Band {
    pub(super) fn roll(rng: &mut Rng) -> Self {
        Self {
            form: rng.pick_weighted(&FORMS),
        }
    }

    pub(super) fn allowed(&self) -> bool {
        has(&FORMS, self.form)
    }

    pub(super) fn name(&self, material: Material) -> &'static str {
        match self.form {
            BandForm::Wound => "Fascia",
            BandForm::Twisted => "Strophium",
            BandForm::WideWrap if matches!(material, Material::Cotton | Material::Linen) => {
                "Sarashi"
            }
            BandForm::WideWrap => "Breast Wrap",
            BandForm::Tied => "Breast Girdle",
            BandForm::Panel => "Dudou",
            BandForm::Strapped => "Lengberg Band",
        }
    }

    pub(super) fn base_grams(&self) -> u32 {
        match self.form {
            BandForm::Wound => 50,
            BandForm::Twisted => 40,
            BandForm::WideWrap => 110,
            BandForm::Tied => 60,
            BandForm::Panel => 50,
            BandForm::Strapped => 80,
        }
    }
}
