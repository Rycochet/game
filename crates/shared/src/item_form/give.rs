//! Giving a new item its form: `give_form`, and blades rolled to a request.

use crate::blade::{self};
use crate::creature::Size;
use crate::id::Id;
use crate::item::{Item, ItemKind, Made};
use crate::item_form::gem::roll_gem;
use crate::item_form::ring_roll::{roll_ring, roll_ring_size};
use crate::item_form::{ItemForm, Limits};

/// Gives a new item its form, for kinds that have one, going by its quality. `size`
/// is what it must be, such as a mob's own; without one, it is rolled. Only the server
/// makes items, so anything else it needs, such as gems, gets its id from `new_id`.
pub fn give_form(item: &mut Item, size: Option<Size>, new_id: &mut impl FnMut() -> Id) {
    let mut rng = item.id.seed().derive("item form").rng();
    let quality = item.quality.unwrap_or_default();
    let armour = super::armour::armour_form(item.kind, &mut rng, quality);
    let (form, rolled_size) = match item.kind {
        _ if let Some(form) = armour => (form, size.unwrap_or(Size::Medium)),
        ItemKind::Gem => {
            let (gem, size) = roll_gem(&mut rng, quality, true);
            (ItemForm::Gem(gem), size)
        }
        ItemKind::Ring => {
            let size = size.unwrap_or_else(|| roll_ring_size(&mut rng));
            let made_at = item.made_at;
            let ring = roll_ring(&mut rng, quality, size, &mut |gem, gem_size, quality| {
                let mut stone = Item::new(new_id(), ItemKind::Gem);
                stone.made = Made::for_body(ItemKind::Gem, gem_size, None, false);
                stone.quality = Some(quality);
                stone.made_at = made_at;
                stone.form = Some(ItemForm::Gem(gem));
                stone
            });
            (ItemForm::Ring(Box::new(ring)), size)
        }
        kind => {
            let Some(family) = kind.blade_family() else {
                return;
            };
            let size = size.unwrap_or(Size::Medium);
            let blade = blade::roll_blade_at(&mut rng, family, quality, size);
            (ItemForm::Blade(Box::new(blade)), size)
        }
    };
    let size = size.unwrap_or(rolled_size);
    item.made = Made::for_body(item.kind, size, item.made.fingers, item.made.magical);
    item.form = Some(form);
}

/// Gives a new blade its form within limits, which may make it another family's kind.
/// `false` if nothing can be made within them, so the item shouldn't exist.
pub fn give_blade_within(item: &mut Item, limits: &Limits) -> bool {
    let Some(family) = item.kind.blade_family() else {
        return false;
    };
    let mut rng = item.id.seed().derive("item form").rng();
    let quality = item.quality.unwrap_or_default();
    let rolled = blade::roll_blade_within(&mut rng, family, quality, limits);
    rolled.map(|rolled| set_blade(item, rolled)).is_some()
}

/// Makes a new item into a specific weapon at any size, keeping its name: only for when
/// that is asked for, as a commission from a shop or an ability might. `false` if there
/// is no such weapon, or it can't keep to the limits.
pub fn give_commission(item: &mut Item, name: &str, size: Size, limits: &Limits) -> bool {
    let mut rng = item.id.seed().derive("item form").rng();
    let quality = item.quality.unwrap_or_default();
    let rolled = blade::commission_blade(&mut rng, name, quality, size, limits);
    rolled.map(|rolled| set_blade(item, rolled)).is_some()
}

pub(super) fn set_blade(item: &mut Item, rolled: blade::Rolled) {
    item.kind = rolled.form.family.kind();
    item.made = Made::for_body(item.kind, rolled.size, item.made.fingers, item.made.magical);
    item.form = Some(ItemForm::Blade(Box::new(rolled.form)));
}
