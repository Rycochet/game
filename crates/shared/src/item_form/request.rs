//! "Must be" options for rolling a new item: `RollRequest`, which of them each kind
//! takes (`fields`), and `Refused`, for a request that can't be met.

use serde::{Deserialize, Serialize};

use crate::armour::{Construction, Dye, Finish, Style};
use crate::footwear::FootwearPiece;
use crate::item::ItemKind;
use crate::item_form::{Cut, Limits};
use crate::material::Material;

/// What a new item must be like. Anything left `None` is rolled as usual, so
/// `RollRequest::NONE` rolls exactly what `give_form` does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollRequest {
    /// Armour: its style (a set's pieces share it).
    pub style: Option<Style>,
    /// Armour: its construction (a set's is its torso's).
    pub construction: Option<Construction>,
    /// Armour's or footwear's material (footwear's upper), a ring's metal or a blade's
    /// blade metal.
    pub material: Option<Material>,
    /// Armour's finish, for pieces that have one.
    pub finish: Option<Finish>,
    /// Armour's cloth, cover and lacing, or footwear's colour.
    pub dye: Option<Dye>,
    /// A gem's material, or every stone's in a ring.
    pub gem_material: Option<Material>,
    /// A gem's cut, or every stone's in a ring (never uncut there).
    pub gem_cut: Option<Cut>,
    /// How many settings a ring has.
    pub settings: Option<u8>,
    /// What a blade must keep to; it may then be another family's kind.
    pub limits: Option<Limits>,
}

/// One of the options in a `RollRequest`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Field {
    Style,
    Construction,
    Material,
    Finish,
    Dye,
    GemMaterial,
    GemCut,
    Settings,
    Limits,
}

/// A request that can't be met, naming the option that couldn't be: one the kind
/// doesn't take, a value it can't have, or nothing rolled that could keep to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Refused(pub Field);

impl RollRequest {
    pub const NONE: Self = Self {
        style: None,
        construction: None,
        material: None,
        finish: None,
        dye: None,
        gem_material: None,
        gem_cut: None,
        settings: None,
        limits: None,
    };

    /// The options it asks for.
    pub fn asked(&self) -> Vec<Field> {
        let asked = [
            (Field::Style, self.style.is_some()),
            (Field::Construction, self.construction.is_some()),
            (Field::Material, self.material.is_some()),
            (Field::Finish, self.finish.is_some()),
            (Field::Dye, self.dye.is_some()),
            (Field::GemMaterial, self.gem_material.is_some()),
            (Field::GemCut, self.gem_cut.is_some()),
            (Field::Settings, self.settings.is_some()),
            (Field::Limits, self.limits.is_some()),
        ];
        asked
            .into_iter()
            .filter_map(|(field, asked)| asked.then_some(field))
            .collect()
    }

    /// The same, with one option rolled as usual again.
    pub fn without(mut self, field: Field) -> Self {
        match field {
            Field::Style => self.style = None,
            Field::Construction => self.construction = None,
            Field::Material => self.material = None,
            Field::Finish => self.finish = None,
            Field::Dye => self.dye = None,
            Field::GemMaterial => self.gem_material = None,
            Field::GemCut => self.gem_cut = None,
            Field::Settings => self.settings = None,
            Field::Limits => self.limits = None,
        }
        self
    }

    /// The first option asked for that this kind doesn't take.
    pub(super) fn not_for(&self, kind: ItemKind) -> Option<Field> {
        let takes = fields(kind);
        self.asked()
            .into_iter()
            .find(|field| !takes.contains(field))
    }
}

/// The options a kind of item takes; none for kinds without a form.
pub fn fields(kind: ItemKind) -> &'static [Field] {
    use Field::*;
    if FootwearPiece::of(kind).is_some() {
        return &[Material, Dye];
    }
    if kind.blade_family().is_some() {
        return &[Material, Limits];
    }
    match kind {
        ItemKind::Gem => &[GemMaterial, GemCut],
        ItemKind::Ring => &[Material, Settings, GemMaterial, GemCut],
        // Their cloth is a material, and shoulders' finish is leather's own.
        ItemKind::Gauntlets
        | ItemKind::DemiGauntlets
        | ItemKind::PaddedGloves
        | ItemKind::ShoulderArmour => &[Style, Construction, Material],
        _ if super::armour::is_armour(kind) => &[Style, Construction, Material, Finish, Dye],
        _ => &[],
    }
}

/// Leaves only `wanted` its weight, if it can come up; otherwise changes nothing and
/// says so. Picking from the result draws the same numbers as from the whole list, so
/// nothing rolled after it changes.
pub(crate) fn keep_only<T: Copy + PartialEq>(options: &mut [(T, u64)], wanted: Option<T>) -> bool {
    let Some(wanted) = wanted else {
        return true;
    };
    if !options
        .iter()
        .any(|&(option, weight)| option == wanted && weight > 0)
    {
        return false;
    }
    for (option, weight) in options.iter_mut() {
        if *option != wanted {
            *weight = 0;
        }
    }
    true
}
