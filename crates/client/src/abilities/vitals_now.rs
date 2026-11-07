//! What the player's Health and Mana allow now, as the client refills them:
//! whether an ability's Mana is there, and whether an automatic Heal is due.
//! The server decides; this only keeps from asking for what it would refuse.

use shared::ability::Ability;
use shared::magic::needs_healing;
use shared::vitals::{PER_POINT, Points};

use crate::common::OwnVitals;

/// Whether there is the Mana for it at `now` (`Time`'s seconds). Nothing told
/// yet: only what costs none.
pub(super) fn affords(ability: Ability, vitals: &OwnVitals, now: f64) -> bool {
    let cost = ability.mana_cost();
    cost == 0 || vitals.0.is_some_and(|told| told.now(now).1 >= cost as f32)
}

/// Whether the player's Health is low enough for an automatic Heal.
pub(super) fn wants_healing(vitals: &OwnVitals, now: f64) -> bool {
    vitals.0.is_some_and(|told| {
        let health = Points((told.now(now).0 * PER_POINT as f32) as u32);
        needs_healing(health, told.health.most)
    })
}
