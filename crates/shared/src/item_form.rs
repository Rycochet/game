//! What one item is physically like, rolled once when the server makes it and saved
//! on the item from then on. Changing the rolls never changes items already made.
//! See `docs/context/shared/item.md`.
//!
//! Each kind rolls its choices in order, each seeing those before it. A choice can
//! lean on an earlier one by changing weights, or limit it with a weight of 0.

use serde::{Deserialize, Serialize};

use crate::creature::Size;
use crate::item::{ItemKind, Quality};
use crate::material::Material;
use crate::rng::{Rng, Seed};

/// What an item is like, for kinds that have this.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemForm {
    Gem(GemForm),
}

impl ItemForm {
    /// What something like this is called, without its size.
    pub fn name(&self) -> String {
        match self {
            Self::Gem(gem) => format!("{} {}", gem.cut.name(), gem.material.name()),
        }
    }
}

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

/// A new item's form, and the size it comes out if it wasn't told.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rolled {
    pub size: Size,
    pub form: ItemForm,
}

/// The form of a new item, for kinds that have one. `size` is what it must be, such
/// as a mob's own; without one, kinds with a form roll it.
pub fn roll_form(
    kind: ItemKind,
    seed: Seed,
    quality: Option<Quality>,
    size: Option<Size>,
) -> Option<Rolled> {
    let mut rng = seed.derive("item form").rng();
    match kind {
        ItemKind::Gem => {
            let quality = quality.unwrap_or_default();
            let (gem, rolled_size) = roll_gem(&mut rng, quality);
            Some(Rolled {
                size: size.unwrap_or(rolled_size),
                form: ItemForm::Gem(gem),
            })
        }
        _ => None,
    }
}

/// Quality, then material (better favours finer), then cut (by material), then size
/// (finer stones come smaller).
fn roll_gem(rng: &mut Rng, quality: Quality) -> (GemForm, Size) {
    let material = rng.pick_weighted(&Material::GEMS.map(|gem| (gem, gem_weight(gem, quality))));
    let cut = rng.pick_weighted(&Cut::ALL.map(|cut| (cut, cut_weight(cut, material))));
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

/// Finer gems are rarer, less so the better the quality.
fn gem_weight(gem: Material, quality: Quality) -> u64 {
    let tier = gem.tier();
    let base = match tier {
        1 => 40,
        2 => 25,
        3 => 12,
        _ => 4,
    };
    base * (1 + quality as u64 * tier * tier)
}

/// How often each cut is used, and the ones a stone is known for more often.
fn cut_weight(cut: Cut, material: Material) -> u64 {
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
fn size_weight(size: Size, material: Material) -> u64 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Category;

    fn gem(seed: u64, quality: Quality) -> (GemForm, Size) {
        let rolled = roll_form(ItemKind::Gem, Seed::new(seed), Some(quality), None).unwrap();
        let ItemForm::Gem(gem) = rolled.form;
        (gem, rolled.size)
    }

    fn share_of(quality: Quality, wanted: impl Fn(&GemForm, Size) -> bool) -> u32 {
        let found = (0..4000).filter(|&seed| {
            let (gem, size) = gem(seed, quality);
            wanted(&gem, size)
        });
        found.count() as u32
    }

    #[test]
    fn the_same_seed_rolls_the_same_gem() {
        // Golden: changing the rolls is fine, but should be noticed.
        let (rolled, size) = gem(1, Quality::Common);
        assert_eq!(gem(1, Quality::Common), (rolled, size));
        assert_eq!(
            (rolled.material, rolled.cut, size),
            (Material::Emerald, Cut::Oval, Size::Medium)
        );
    }

    #[test]
    fn only_kinds_with_a_form_roll_one() {
        assert_eq!(roll_form(ItemKind::Apple, Seed::new(1), None, None), None);
    }

    #[test]
    fn a_size_given_is_kept_and_the_gem_stays_the_same() {
        let told = roll_form(
            ItemKind::Gem,
            Seed::new(1),
            Some(Quality::Common),
            Some(Size::Giant),
        );
        let told = told.unwrap();
        assert_eq!(told.size, Size::Giant);
        let ItemForm::Gem(form) = told.form;
        assert_eq!(form, gem(1, Quality::Common).0);
    }

    #[test]
    fn every_gem_rolled_is_a_gem_and_only_uncut_ones_have_a_shape_seed() {
        for seed in 0..4000 {
            let (gem, _) = gem(seed, Quality::Rare);
            assert_eq!(gem.material.category(), Category::Gem);
            assert_eq!(gem.cut == Cut::Uncut, gem.shape_seed != 0, "{gem:?}");
        }
    }

    #[test]
    fn better_gems_are_more_often_fine() {
        let diamonds = |quality| share_of(quality, |gem, _| gem.material == Material::Diamond);
        assert!(diamonds(Quality::Legendary) > diamonds(Quality::Poor) * 4);
    }

    #[test]
    fn stones_favour_the_cuts_they_are_known_for() {
        let emerald_cut = |wanted: Material| {
            let of_it = (0..40_000).map(|seed| gem(seed, Quality::Rare).0);
            let of_it: Vec<_> = of_it.filter(|gem| gem.material == wanted).collect();
            let cut = of_it.iter().filter(|gem| gem.cut == Cut::Emerald).count();
            cut as f64 / of_it.len() as f64
        };
        assert!(emerald_cut(Material::Emerald) > emerald_cut(Material::Ruby) * 3.0);
    }

    #[test]
    fn the_finest_stones_never_come_giant() {
        let fine_giants = |gem: &GemForm, size| gem.material.tier() >= 3 && size == Size::Giant;
        assert_eq!(share_of(Quality::Legendary, fine_giants), 0);
        let any_giants = |_: &GemForm, size| size == Size::Giant;
        assert!(share_of(Quality::Common, any_giants) > 0);
    }

    #[test]
    fn a_gem_is_named_by_its_cut_and_stone() {
        let form = ItemForm::Gem(GemForm {
            material: Material::Ruby,
            cut: Cut::Uncut,
            shape_seed: 3,
        });
        assert_eq!(form.name(), "Uncut Ruby");
    }
}
