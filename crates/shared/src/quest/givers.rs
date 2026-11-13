//! Who in a town gives quests: its buildings (the door is the person, for
//! now), each with quest slots by a hash of the town and the building, more
//! in bigger towns; some give dailies too.

use super::world::size;
use crate::creature::CreatureKind;
use crate::poi::Poi;
use crate::rng::Seed;
use crate::site::{BuildingKind, BuildingLook};
use crate::story::Household;

/// A building giving quests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Giver {
    /// Its `RoomId`.
    pub building: u16,
    pub kind: BuildingKind,
    /// How many chains of quests it offers; the giver of slot `n` is its
    /// household's member `n`.
    pub slots: u8,
    /// Whether it offers a daily quest.
    pub daily: bool,
}

/// Percent of houses with a quest.
const HOUSE_PERCENT: u64 = 30;
/// Percent of other buildings with a second slot, by town size.
const SECOND_SLOT_PERCENT: [u64; 4] = [10, 20, 30, 40];
/// Percent of buildings other than houses and inns giving dailies (inns
/// always do).
const DAILY_PERCENT: u64 = 25;

/// The quests' own seed for a town: nothing is drawn from the site's.
pub(super) fn quest_seed(town: &Poi) -> Seed {
    town.seed.derive("quests")
}

/// Every building of `town` that gives quests, in the buildings' order.
pub fn givers(town: &Poi, buildings: &[BuildingLook]) -> Vec<Giver> {
    let seed = quest_seed(town).derive("givers");
    let size = size(town.kind).unwrap_or(0);
    (buildings.iter())
        .filter_map(|building| {
            let roll = |n: i64| seed.at(&[i64::from(building.id), n]) % 100;
            let house = building.kind == BuildingKind::House;
            let slots = if house {
                u8::from(roll(0) < HOUSE_PERCENT)
            } else {
                1 + u8::from(roll(0) < SECOND_SLOT_PERCENT[size])
            };
            let daily = building.kind == BuildingKind::Inn || (!house && roll(1) < DAILY_PERCENT);
            (slots > 0 || daily).then_some(Giver {
                building: building.id,
                kind: building.kind,
                slots,
                daily,
            })
        })
        .collect()
}

/// The household living in a building: its family name and its members.
/// Townsfolk are human until towns have peoples of their own.
pub(super) fn household(town: &Poi, building: u16) -> Household {
    let seed = quest_seed(town).derive("household");
    Household::new(
        CreatureKind::Human,
        Seed::new(seed.at(&[i64::from(building)])),
    )
}
