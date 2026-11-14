//! Mobs being healed by a player's Heal.

use shared::id::Id;
use shared::vitals::{Points, Vital};

use crate::ServerWorld;
use crate::combat::health_left;
use crate::storage::SiteId;

/// A mob on this site level (`Visit::population`) is given this much Health,
/// never past its most. How much it has now, per mille; `None` if it is no
/// longer there. It doesn't turn on the healer, whoever it is fighting: taming
/// is to build on this.
pub(crate) fn heal_mob(
    world: &mut ServerWorld,
    place: (SiteId, u32),
    mob: Id,
    amount: Points,
    tick: u64,
) -> Option<u16> {
    let population = world.populations.0.get_mut(&place)?;
    let one = population.mobs.iter_mut().find(|one| one.id == mob)?;
    let limits = one.vitals.limits(&one.attributes, 0);
    one.vitals.restore(Vital::Health, amount, limits, tick);
    Some(health_left(&one.vitals, limits, tick))
}
