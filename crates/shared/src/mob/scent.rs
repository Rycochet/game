//! Scent: how strong a creature's trail is, and how long a nose can still follow
//! it. Everything that walks leaves one; the server keeps the marks.

use crate::creature::Acuity;
use crate::item::Inventory;

/// Action ticks in a second (`energy::ACTION_TICKS_PER_SECOND`, as a whole number).
const TICKS_PER_SECOND: u64 = 20;

/// The weakest trail, as a percentage: a creature carrying nothing.
pub const BASE_SCENT: u32 = 100;
/// The strongest trail: twice as long-lasting as the weakest.
pub const MOST_SCENT: u32 = 200;
/// How much stronger each kilogram worn makes a trail, in percent. Plate (about
/// 25 kg) doubles it; light clothes barely change it. Weight stands in for metal
/// for now, as heavy armour is metal armour.
const PER_KG_WORN: u32 = 4;
/// The same for what is carried rather than worn.
const PER_KG_CARRIED: u32 = 1;

/// How strong a trail someone wearing and carrying this leaves, as a percentage.
pub fn scent_strength(inventory: &Inventory) -> u32 {
    let worn: u32 = inventory.worn.iter().map(|item| item.total_weight()).sum();
    let carried = inventory.total_weight().saturating_sub(worn);
    let extra = worn / 1000 * PER_KG_WORN + carried / 1000 * PER_KG_CARRIED;
    (BASE_SCENT + extra).min(MOST_SCENT)
}

/// How long, in action ticks, a nose this good can follow the weakest trail.
pub const fn smells_for(smell: Acuity) -> u64 {
    match smell {
        Acuity::None => 0,
        Acuity::Poor => 30 * TICKS_PER_SECOND,
        Acuity::Normal => 2 * 60 * TICKS_PER_SECOND,
        Acuity::Keen => 5 * 60 * TICKS_PER_SECOND,
    }
}

/// After this many action ticks, no nose can smell any mark: the keenest nose on
/// the strongest trail. Older marks can be thrown away.
pub const SCENT_KEPT_FOR: u64 = smells_for(Acuity::Keen) * MOST_SCENT as u64 / 100;

/// Whether a nose this good can still smell a mark this strong, laid at `laid`.
pub fn can_smell(smell: Acuity, strength: u32, laid: u64, now: u64) -> bool {
    let lasts = smells_for(smell) * strength as u64 / 100;
    now.saturating_sub(laid) < lasts
}
