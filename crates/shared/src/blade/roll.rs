//! Rolling a new blade: ordinary, scaled to a size, within limits, or to order.

use super::table::{Row, rows};
use super::{BladeForm, Family, SCALED_PERCENT_LARGE, SCALED_PERCENT_SMALL, roll_row_within};
use crate::creature::Size;
use crate::item::Quality;
use crate::item_form::Limits;
use crate::rng::Rng;

/// A weapon rolled for a request, with the size it must be made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rolled {
    pub form: BladeForm,
    pub size: Size,
}

/// A new ordinary weapon of a family. Picks a row of the table, by how common it is, then
/// a cell it may be in, then each property in turn, within what the row allows.
pub fn roll_blade(rng: &mut Rng, family: Family, quality: Quality) -> BladeForm {
    roll_ordinary(rng, family, quality, &Limits::NONE).expect("nothing limits it")
}

/// A weapon that must be this size: if it isn't the ordinary one, it is scaled, and so
/// has only its generic name.
pub fn roll_blade_at(rng: &mut Rng, family: Family, quality: Quality, size: Size) -> BladeForm {
    roll_blade_at_within(rng, family, quality, size, &Limits::NONE).expect("nothing limits it")
}

/// `roll_blade_at`, keeping to limits without changing its family or size. `None` if
/// nothing can be made within them.
pub fn roll_blade_at_within(
    rng: &mut Rng,
    family: Family,
    quality: Quality,
    size: Size,
    limits: &Limits,
) -> Option<BladeForm> {
    if size == Size::Medium {
        return roll_ordinary(rng, family, quality, limits);
    }
    roll_scaled(rng, family, quality, limits, size)
}

/// A weapon within limits. An ordinary one if any will do, of this family or else the
/// nearest that fits, so a tiny creature asking for a sword usually gets a dagger.
/// Sometimes, or if nothing ordinary fits, one of this family scaled, nearest its
/// wielder's size. `None` if nothing can be made.
pub fn roll_blade_within(
    rng: &mut Rng,
    family: Family,
    quality: Quality,
    limits: &Limits,
) -> Option<Rolled> {
    let wielder = limits.wielder.unwrap_or(Size::Medium);
    let percent = if wielder < Size::Medium {
        SCALED_PERCENT_SMALL
    } else {
        SCALED_PERCENT_LARGE
    };
    let scale_first = wielder != Size::Medium && rng.range_u64(0, 100) < percent;
    if scale_first {
        let rolled = scaled_near(rng, family, quality, limits, wielder);
        if rolled.is_some() {
            return rolled;
        }
    }
    let mut families = vec![family];
    families.extend(family.others(rng));
    for other in families {
        if let Some(form) = roll_ordinary(rng, other, quality, limits) {
            let size = Size::Medium;
            return Some(Rolled { form, size });
        }
    }
    scaled_near(rng, family, quality, limits, wielder)
}

/// A specific weapon at any size, keeping its name: only when asked for, such as a
/// commission from a shop. `None` if there is no such name, or it can't keep to limits.
pub fn commission_blade(
    rng: &mut Rng,
    name: &str,
    quality: Quality,
    size: Size,
    limits: &Limits,
) -> Option<Rolled> {
    let row = rows().find(|row| row.name == name)?;
    let form = roll_row_within(rng, row, quality, limits, size)?;
    Some(Rolled { form, size })
}

/// A wielder's limits: what it can lift without being burdened, and what suits its size.
/// Those smaller than ordinary can't manage anything long, and the largest have no use
/// for anything short.
pub fn limits_for_wielder(size: Size, spare_grams: u32) -> Limits {
    let (longest, shortest) = match size {
        Size::Tiny => (Some(2), None),
        Size::Small => (Some(4), None),
        Size::Medium | Size::Large => (None, None),
        Size::Huge => (None, Some(5)),
        Size::Giant => (None, Some(6)),
    };
    Limits {
        fits: longest.map(|longest| (2, longest)),
        at_least_long: shortest,
        most_grams: Some(spare_grams),
        least_grams: None,
        wielder: Some(size),
        metal: None,
    }
}

/// An ordinary weapon of one family: rows that can't fit are passed over.
pub(super) fn roll_ordinary(
    rng: &mut Rng,
    family: Family,
    quality: Quality,
    limits: &Limits,
) -> Option<BladeForm> {
    let mut rows: Vec<(&Row, u64)> = rows()
        .filter(|row| row.family == family)
        .map(|row| (row, row.weight))
        .collect();
    while !rows.is_empty() {
        let row = rng.pick_weighted(&rows);
        let form = roll_row_within(rng, row, quality, limits, Size::Medium);
        if form.is_some() {
            return form;
        }
        rows.retain(|(other, _)| other.name != row.name);
    }
    None
}

/// A generic weapon of one family at another size.
pub(super) fn roll_scaled(
    rng: &mut Rng,
    family: Family,
    quality: Quality,
    limits: &Limits,
    size: Size,
) -> Option<BladeForm> {
    let mut generic = rows().filter(|row| row.family == family && row.generic);
    let mut form = generic.find_map(|row| roll_row_within(rng, row, quality, limits, size))?;
    form.generic_only = true;
    Some(form)
}

/// Scaled, trying the sizes in order from the wielder's own outwards.
pub(super) fn scaled_near(
    rng: &mut Rng,
    family: Family,
    quality: Quality,
    limits: &Limits,
    wielder: Size,
) -> Option<Rolled> {
    let mut sizes: Vec<Size> = Size::ALL
        .into_iter()
        .filter(|&size| size != Size::Medium)
        .collect();
    sizes.sort_by_key(|&size| (size as i32 - wielder as i32).abs());
    sizes.into_iter().find_map(|size| {
        let form = roll_scaled(rng, family, quality, limits, size)?;
        Some(Rolled { form, size })
    })
}
