//! How a material changes what armour made of it gives.

use crate::item::Item;
use crate::item_form::ItemForm;
use crate::material::Material;

/// Thousandths each armour bonus is multiplied by for a material: 1000 is as
/// usual.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaterialFactors {
    pub toughness: i64,
    pub agility: i64,
    pub mana: i64,
}

impl MaterialFactors {
    pub const USUAL: Self = Self {
        toughness: 1000,
        agility: 1000,
        mana: 1000,
    };
}

/// A material's factors. All as usual for now (owner, 2026-11-14): later
/// Mithril might cost less Agility, Adamantine give more Toughness.
pub fn material_factors(_material: Material) -> MaterialFactors {
    MaterialFactors::USUAL
}

/// What an armour piece is made of, as its form says (a set's is its torso's).
pub(super) fn armour_material(item: &Item) -> Option<Material> {
    Some(match item.form.as_ref()? {
        ItemForm::Hands(form) => form.material,
        ItemForm::Shoulders(form) => form.material,
        ItemForm::Torso(form) => form.material,
        ItemForm::Head(form) => form.material,
        ItemForm::Arms(form) => form.material,
        ItemForm::Legs(form) => form.material,
        ItemForm::Waist(form) => form.material,
        ItemForm::Feet(form) => form.material,
        ItemForm::Neck(form) => form.material,
        ItemForm::Set(form) => form.material(),
        _ => return None,
    })
}
