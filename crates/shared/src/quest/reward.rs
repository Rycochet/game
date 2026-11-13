//! What a quest pays: coins in copper by the town's size, the work asked and
//! how far it goes; sometimes more (an item, training or a favour), which
//! lets the story speak of seasons and weather. All figures *to balance*.

use super::plan::Plan;
use super::world::{offset, size};
use crate::poi::{CHUNK_SIZE, Poi, PoiMap};
use crate::rng::Rng;
use crate::story::{Goal, ItemCategory, RewardPart};

/// Copper a step is worth in a hamlet, a village, a town and a city.
const BASE: [u64; 4] = [30, 50, 80, 120];
/// Percent of quests (not dailies) paying more than coins.
const HIGHER_PERCENT: u64 = 20;
/// What a higher reward may be.
const EXTRAS: [(Extra, u64); 3] = [(Extra::Item, 3), (Extra::Training, 2), (Extra::Favour, 1)];
/// Items given as rewards.
const REWARD_ITEMS: [ItemCategory; 5] = [
    ItemCategory::Weapon,
    ItemCategory::Ring,
    ItemCategory::Tool,
    ItemCategory::Remedy,
    ItemCategory::Book,
];

#[derive(Clone, Copy)]
enum Extra {
    Item,
    Training,
    Favour,
}

/// The reward for `plan` from `town`, the `chain`th quest of its slot, and
/// whether it is higher than coins.
pub(super) fn reward(
    pois: &PoiMap,
    town: &Poi,
    plan: &Plan,
    chain: u8,
    daily: bool,
    rng: &mut Rng,
) -> (Vec<RewardPart>, bool) {
    let base = BASE[size(town.kind).unwrap_or(0)];
    let work: u64 = (plan.steps.iter())
        .map(|step| {
            let count = u64::from(step.count.unwrap_or(1));
            match step.goal {
                Goal::Slay if step.count.is_some() => base * count / 4,
                Goal::Gather => base * count / 8,
                Goal::Visit => base / 2,
                _ => base,
            }
        })
        .sum();
    // Further away pays more: half as much again per 2 chunks.
    let far = plan.destination.or(plan.place).map_or(0, |poi| {
        let (dx, dy) = offset(pois, (town.x, town.y), (poi.x, poi.y));
        (dx * dx + dy * dy).isqrt() as u64 / CHUNK_SIZE as u64
    });
    let mut copper = work * (4 + far) / 4 * (2 + u64::from(chain)) / 2;
    if daily {
        copper /= 2;
    }
    let mut parts = vec![RewardPart::Coins(copper.div_ceil(5) * 5)];
    let higher = !daily && rng.range_u64(0, 100) < HIGHER_PERCENT;
    if higher {
        parts.push(match rng.pick_weighted(&EXTRAS) {
            Extra::Item => {
                RewardPart::Item(REWARD_ITEMS[rng.range_u64(0, REWARD_ITEMS.len() as u64) as usize])
            }
            Extra::Training => RewardPart::Training,
            Extra::Favour => RewardPart::Favour,
        });
    }
    (parts, higher)
}
