//! Gems: `GemForm`, `Cut`, and rolling a gem's material, cut and size.

use serde::{Deserialize, Serialize};

use crate::creature::Size;
use crate::item::Quality;
use crate::material::Material;
use crate::rng::Rng;

/// How a gem has been cut, or not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cut {
    Cushion,
    Emerald,
    Oval,
    Pear,
    Princess,
    Round,
    Trillion,
    Uncut,
}

impl Cut {
    pub const ALL: [Self; 8] = [
        Self::Cushion,
        Self::Emerald,
        Self::Oval,
        Self::Pear,
        Self::Princess,
        Self::Round,
        Self::Trillion,
        Self::Uncut,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Cushion => "Cushion",
            Self::Emerald => "Emerald",
            Self::Oval => "Oval",
            Self::Pear => "Pear",
            Self::Princess => "Princess",
            Self::Round => "Round",
            Self::Trillion => "Trillion",
            Self::Uncut => "Uncut",
        }
    }
}

/// One gem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GemForm {
    pub material: Material,
    pub cut: Cut,
    /// The lumps of an uncut stone. 0 for a cut one, whose looks don't depend on it.
    pub shape_seed: u16,
}

/// Quality, then material (better favours finer), then cut (by material), then size
/// (finer stones come smaller).
pub(super) fn roll_gem(rng: &mut Rng, quality: Quality, may_be_uncut: bool) -> (GemForm, Size) {
    let material = rng.pick_weighted(&Material::GEMS.map(|gem| (gem, fine_weight(gem, quality))));
    let cut = roll_cut(rng, material, may_be_uncut);
    let size = rng.pick_weighted(&Size::ALL.map(|size| (size, size_weight(size, material))));
    let shape_seed = if cut == Cut::Uncut {
        rng.range_u64(1, 1 << 16) as u16
    } else {
        0
    };
    let gem = GemForm {
        material,
        cut,
        shape_seed,
    };
    (gem, size)
}

pub(super) fn roll_cut(rng: &mut Rng, material: Material, may_be_uncut: bool) -> Cut {
    let weight = |cut| match (cut, may_be_uncut) {
        (Cut::Uncut, false) => 0,
        _ => cut_weight(cut, material),
    };
    rng.pick_weighted(&Cut::ALL.map(|cut| (cut, weight(cut))))
}

/// Finer materials are rarer, less so the better the quality.
pub(super) fn fine_weight(material: Material, quality: Quality) -> u64 {
    let tier = material.tier();
    let base = match tier {
        1 => 40,
        2 => 25,
        3 => 12,
        _ => 4,
    };
    base * (1 + quality as u64 * tier * tier)
}

/// How often each cut is used, and the ones a stone is known for more often.
pub(super) fn cut_weight(cut: Cut, material: Material) -> u64 {
    let base = match cut {
        Cut::Round => 30,
        Cut::Oval => 20,
        Cut::Cushion => 15,
        Cut::Uncut => 12,
        Cut::Princess | Cut::Pear => 10,
        Cut::Emerald | Cut::Trillion => 5,
    };
    let known_for = matches!(
        (material, cut),
        (Material::Emerald, Cut::Emerald) | (Material::Diamond, Cut::Round | Cut::Princess)
    );
    if known_for { base * 6 } else { base }
}

/// Mostly around medium. Whole-number division leaves the finest with no giants.
pub(super) fn size_weight(size: Size, material: Material) -> u64 {
    let tier = material.tier();
    match size {
        Size::Tiny => 10 + 5 * tier,
        Size::Small => 30,
        Size::Medium => 40,
        Size::Large => 16 / tier,
        Size::Huge => 6 / tier,
        Size::Giant => 2 / tier,
    }
}
