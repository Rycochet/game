//! Daily quests: the one a giver offers on a UTC day, `hash(town, giver,
//! day)`, the same for everyone; short, near the town, on shared hooks only
//! so nobody's daily takes anything from anyone.

use super::givers::quest_seed;
use super::make::{Making, make};
use super::template::plan;
use super::town::town_candidates;
use super::{Quest, QuestId, QuestKind, QuestWorld, Slot, givers};
use crate::poi::Poi;
use crate::rng::Seed;
use crate::site::BuildingLook;

pub const SECONDS_PER_DAY: i64 = 24 * 60 * 60;

/// The UTC day (days since 1 January 1970) at this many seconds since then.
pub fn utc_day(unix_seconds: i64) -> i64 {
    unix_seconds.div_euclid(SECONDS_PER_DAY)
}

/// The buildings of `town` that give dailies.
pub fn daily_givers(town: &Poi, buildings: &[BuildingLook]) -> Vec<u16> {
    (givers(town, buildings).into_iter())
        .filter(|giver| giver.daily)
        .map(|giver| giver.building)
        .collect()
}

/// The daily quest `building` of `town` offers on `day`, if it gives them.
pub fn daily_quest(
    world: &QuestWorld,
    town: &Poi,
    buildings: &[BuildingLook],
    building: u16,
    day: i64,
) -> Option<Quest> {
    if !daily_givers(town, buildings).contains(&building) {
        return None;
    }
    let candidates = town_candidates(world, town, true);
    let seed = Seed::new((quest_seed(town).derive("daily")).at(&[i64::from(building), day]));
    let mut rng = seed.derive("plan").rng();
    let weights: Vec<(usize, u64)> = (candidates.iter().enumerate())
        .map(|(i, c)| (i, c.weight))
        .collect();
    let plan = if rng.range_u64(0, 100) < super::dungeon::DAILY_PERCENT {
        super::dungeon::plan_cellar(town, true, &mut rng)
    } else {
        let main = candidates[rng.pick_weighted(&weights)];
        plan(town, &main, &candidates, &mut rng)
    };
    let making = Making {
        id: QuestId {
            town: town.chunk,
            slot: Slot::Daily { building, day },
        },
        kind: QuestKind::Daily,
        building,
        member: 0,
        requirements: Vec::new(),
        chain: 0,
        earlier: None,
        dark: false,
        continues: false,
        text_seed: seed.derive("text"),
    };
    Some(make(world, town, &plan, making, &mut rng))
}
