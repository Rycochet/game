//! Loincloths: a strip, triangle or tied cloth between the legs, on the Waist alone.

use serde::{Deserialize, Serialize};

use super::table::has;
use crate::material::Material;
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoinForm {
    /// A long strip round the waist and between the legs, tucked through itself.
    Strip,
    /// A strip hung from a cord, its front flap hanging (etchu).
    Apron,
    /// A thin strip with strings at its corners (kaupinam, langota).
    Narrow,
    /// A triangle with strings or loops (cache-sexe, perizoma).
    Triangle,
    /// Strings with a hanging part pulled up behind the knot (subligaculum).
    Tied,
    /// Cut and sewn shorts, cloth or leather.
    Briefs,
}

impl LoinForm {
    /// Cut to the body rather than wrapped: these may be leather.
    pub fn cut(self) -> bool {
        matches!(self, Self::Triangle | Self::Tied | Self::Briefs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Loincloth {
    pub form: LoinForm,
    /// Twisted to a rope at the back: strips only.
    pub twisted: bool,
    /// Sewn rather than tied: narrow ones only (langota).
    pub sewn: bool,
}

const FORMS: [(LoinForm, u64); 6] = [
    (LoinForm::Strip, 25),
    (LoinForm::Apron, 15),
    (LoinForm::Narrow, 15),
    (LoinForm::Triangle, 15),
    (LoinForm::Tied, 15),
    (LoinForm::Briefs, 15),
];

impl Loincloth {
    pub(super) fn roll(rng: &mut Rng) -> Self {
        let form = rng.pick_weighted(&FORMS);
        let twisted = form == LoinForm::Strip && rng.range_u64(0, 3) == 0;
        let sewn = form == LoinForm::Narrow && rng.range_u64(0, 2) == 0;
        Self {
            form,
            twisted,
            sewn,
        }
    }

    pub(super) fn allowed(&self) -> bool {
        has(&FORMS, self.form)
            && (!self.twisted || self.form == LoinForm::Strip)
            && (!self.sewn || self.form == LoinForm::Narrow)
    }

    pub(super) fn name(&self, material: Material) -> &'static str {
        let leather = material == Material::Leather;
        match self.form {
            LoinForm::Strip if self.twisted => "Fundoshi",
            LoinForm::Strip => "Loincloth",
            LoinForm::Apron => "Etchu",
            LoinForm::Narrow if self.sewn => "Langota",
            LoinForm::Narrow => "Kaupinam",
            LoinForm::Triangle if material == Material::Linen => "Cache-sexe",
            LoinForm::Triangle => "Perizoma",
            LoinForm::Tied => "Subligaculum",
            LoinForm::Briefs if leather => "Subligaculum",
            LoinForm::Briefs => "Tubban",
        }
    }

    pub(super) fn base_grams(&self) -> u32 {
        match self.form {
            LoinForm::Strip => 140,
            LoinForm::Apron => 110,
            LoinForm::Narrow => 30,
            LoinForm::Triangle => 50,
            LoinForm::Tied => 70,
            LoinForm::Briefs => 90,
        }
    }
}
