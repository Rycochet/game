//! `Weapon`: what combat needs to know of what a creature attacks with.

use crate::blade::Family;
use crate::item::{Item, Quality};
use crate::item_form::ItemForm;

/// What an attack is made with, as far as the rules care: bare hands (or claws,
/// hooves, beaks: whatever a kind fights with) or a blade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Weapon {
    /// The family of blade, or `None` for no weapon.
    pub family: Option<Family>,
    /// What it weighs, in grams.
    pub grams: u32,
    /// From the end of the handle to the tip, in millimetres, at its size.
    pub length_mm: u32,
    pub quality: Quality,
    /// How fine its metal is (`Material::tier`, 1 to 4).
    pub metal_tier: u64,
}

impl Weapon {
    /// Fighting with no weapon.
    pub const UNARMED: Self = Self {
        family: None,
        grams: 0,
        length_mm: 0,
        quality: Quality::Common,
        metal_tier: 2,
    };

    /// The weapon an item is, if it is one: so far only blades, which know their
    /// own weight and length.
    pub fn of(item: &Item) -> Option<Self> {
        let Some(ItemForm::Blade(blade)) = &item.form else {
            return None;
        };
        let length = u32::from(blade.blade) + u32::from(blade.handle);
        Some(Self {
            family: Some(blade.family),
            grams: item.total_weight(),
            length_mm: length * item.made.size.quarters() / 4,
            quality: item.quality.unwrap_or_default(),
            metal_tier: blade.blade_metal.tier(),
        })
    }

    pub fn is_unarmed(&self) -> bool {
        self.family.is_none()
    }

    /// How heavy it feels to swing: its weight, counted for more the longer it
    /// is, as weight far from the hands is slower to move (polearms are slow by
    /// leverage more than by grams). A metre long counts as it weighs. First guess.
    pub fn handling_grams(&self) -> u32 {
        let per_mille = (667 + self.length_mm / 3).max(500);
        (u64::from(self.grams) * u64::from(per_mille) / 1000) as u32
    }

    /// How many cells away it reaches: long hafted weapons two, everything else
    /// the cells next to the wielder (owner, 2026-11-07).
    pub fn reach(&self) -> u32 {
        match self.family {
            Some(Family::Polearm | Family::Spear) => 2,
            _ => 1,
        }
    }

    /// Whether it cuts or stabs, doing more than a fist.
    pub fn is_edged(&self) -> bool {
        self.family.is_some()
    }
}
