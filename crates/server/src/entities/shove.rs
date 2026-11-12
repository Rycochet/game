//! Shoving a mob half a cell (a kick, `connections::kick`).

use shared::clearance::{Crowding, Other, check_segment, crowded_at};
use shared::id::Id;
use shared::movement::mob_can_stand;
use shared::spot::{HALF_SPAN, Spot};

use crate::ServerWorld;
use crate::storage::SiteId;

/// Where mob `target` on `place` would be shoved by someone at `from`: half a
/// cell straight away from them, if its circle can go there (`check_segment`)
/// without landing on another mob or on a World Gate (players are the
/// caller's to check).
pub(crate) fn shoved_to(
    world: &ServerWorld,
    place: (SiteId, u32),
    target: Id,
    from: Spot,
) -> Option<Spot> {
    let population = world.populations.0.get(&place)?;
    let mob = population.mobs.iter().find(|mob| mob.id == target)?;
    let apart = from.distance(mob.spot) as i64;
    if apart == 0 {
        return None;
    }
    let (dx, dy) = (mob.spot.x - from.x, mob.spot.y - from.y);
    let to = mob
        .spot
        .offset(dx * HALF_SPAN / apart, dy * HALF_SPAN / apart);
    let level = population.level();
    let (x, y) = to.cell();
    if !mob_can_stand(level, x, y) || check_segment(level, mob.size(), mob.spot, to).is_none() {
        return None;
    }
    let mobs = population.mobs.iter().filter(|other| other.id != target);
    let others: Vec<Other> = mobs
        .map(|other| Other {
            at: other.spot,
            size: other.size(),
            crowding: Crowding::Squeeze,
        })
        .collect();
    (!crowded_at(mob.size(), to, &others)).then_some(to)
}

/// Puts mob `target` on `place` at `to` (`shoved_to`).
pub(crate) fn shove(world: &mut ServerWorld, place: (SiteId, u32), target: Id, to: Spot) {
    let population = world.populations.0.get_mut(&place);
    let mob = population.and_then(|p| p.mobs.iter_mut().find(|mob| mob.id == target));
    if let Some(mob) = mob {
        mob.place(to);
    }
}
