//! Wimples and barbes: linen draped round the throat and chin, pinned at the head.

use serde::{Deserialize, Serialize};

use super::table::has;
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WimpleForm {
    /// Round the throat and chin, pinned at the crown or temples.
    Wimple,
    /// A pleated chin cloth (late medieval).
    Barbe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Wimple {
    pub form: WimpleForm,
}

const FORMS: [(WimpleForm, u64); 2] = [(WimpleForm::Wimple, 80), (WimpleForm::Barbe, 20)];

impl Wimple {
    pub(super) fn roll(rng: &mut Rng) -> Self {
        Self {
            form: rng.pick_weighted(&FORMS),
        }
    }

    pub(super) fn allowed(&self) -> bool {
        has(&FORMS, self.form)
    }

    pub(super) fn name(&self) -> &'static str {
        match self.form {
            WimpleForm::Wimple => "Wimple",
            WimpleForm::Barbe => "Barbe",
        }
    }

    pub(super) fn base_grams(&self) -> u32 {
        match self.form {
            WimpleForm::Wimple => 60,
            WimpleForm::Barbe => 70,
        }
    }
}
