//! Quests that wait for one of the world's moons to be in the sky: a hidden
//! need, told only by the giver's hint (owner).

use super::givers::quest_seed;
use super::{Need, QuestId, Requirement, Slot};
use crate::poi::Poi;
use crate::rng::Seed;

/// Percent of givers' quests outside hamlets that wait for a moon. *To
/// balance.*
pub(super) const MOON_PERCENT: u64 = 6;

/// The moon `id` waits for, if any, by its own seed (so no other draw
/// changes): never on a world without moons, nor in hamlets.
pub(super) fn moon_need(town: &Poi, id: QuestId, moons: u8) -> Option<Requirement> {
    let Slot::Giver {
        building,
        slot,
        chain,
    } = id.slot
    else {
        return None;
    };
    if moons == 0 || super::world::size(town.kind).unwrap_or(0) == 0 {
        return None;
    }
    let at = [i64::from(building), i64::from(slot), i64::from(chain)];
    let mut rng = Seed::new(quest_seed(town).derive("moon need").at(&at)).rng();
    (rng.range_u64(0, 100) < MOON_PERCENT).then(|| {
        let moon = rng.range_u64(0, u64::from(moons)) as u8;
        Requirement::hidden(Need::Moon { moon })
    })
}
