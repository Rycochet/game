//! Giving a new item its form: `give_form`, and blades rolled to a request.

use crate::blade::{self};
use crate::creature::Size;
use crate::id::Id;
use crate::item::{Item, Made};
use crate::item_form::{ItemForm, Limits, RollRequest, give_form_to};
use crate::rng::Seed;

/// Gives a new item its form, for kinds that have one, going by its quality. `size`
/// is what it must be, such as a mob's own; without one, it is rolled. Only the server
/// makes items, so anything else it needs, such as gems, gets its id from `new_id`.
pub fn give_form(item: &mut Item, size: Option<Size>, new_id: &mut impl FnMut() -> Id) {
    give_form_from(item, size, item.id.seed(), new_id);
}

/// `give_form`, rolled from `seed` rather than the item's id, so the same seed makes
/// the same form whatever the id: Give Item's preview and what the server makes.
pub fn give_form_from(
    item: &mut Item,
    size: Option<Size>,
    seed: Seed,
    new_id: &mut impl FnMut() -> Id,
) {
    give_form_to(item, size, seed, &RollRequest::NONE, new_id)
        .expect("asking for nothing is never refused");
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
