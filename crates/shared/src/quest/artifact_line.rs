//! A city's artifact line (owner): for characters Honoured there or more, a
//! long chain of whole quests across the city's road neighbours, ending in a
//! large quest dungeon of its own and an artifact or the planet's set item,
//! by the city's seed. Each quest uses only shared hooks round its neighbour,
//! so nothing is taken from that town's own quests.

use super::area::Candidate;
use super::feeder::linked;
use super::givers::quest_seed;
use super::hidden::plan_hidden;
use super::make::{Making, make};
use super::plan::Plan;
use super::template::plan;
use super::town::town_candidates;
use super::{Need, Quest, QuestId, QuestKind, QuestWorld, Requirement, Slot, Standing, givers};
use crate::poi::{Poi, PoiKind};
use crate::rng::{Rng, Seed};
use crate::site::{BuildingKind, BuildingLook};
use crate::story::RewardPart;

/// How many quests a line has, the last its dungeon (Claude's figures, *to
/// balance*): 5 to 7.
const SHORTEST: u64 = 5;
const LONGEST: u64 = 7;
/// Percent of cities whose line ends in the planet's set item rather than an
/// artifact (Claude's figure).
const SET_ITEM_PERCENT: u64 = 50;

/// `city`'s artifact line, first to last; empty for anything but a city, or
/// a city with no road neighbours or no hidden way down near it.
pub(super) fn artifact_line(
    world: &QuestWorld,
    city: &Poi,
    buildings: &[BuildingLook],
) -> Vec<Quest> {
    if city.kind != PoiKind::City {
        return Vec::new();
    }
    let seed = quest_seed(city).derive("artifact line");
    let mut rng = seed.derive("plan").rng();
    let neighbours = linked(world, city);
    if neighbours.is_empty() {
        return Vec::new();
    }
    // Round the neighbours in turn, from one by the seed.
    let first = rng.range_u64(0, neighbours.len() as u64) as usize;
    let Some(building) = giver(city, buildings, &mut rng) else {
        return Vec::new();
    };
    let length = rng.range_u64(SHORTEST, LONGEST + 1) as usize;
    let mut plans: Vec<Plan> = Vec::new();
    for step in 0..length - 1 {
        let neighbour = neighbours[(first + step) % neighbours.len()];
        if let Some(plan) = plan_near(world, &neighbour, &mut rng) {
            plans.push(plan);
        }
    }
    let Some(last) = plan_hidden(world, city, true, &mut rng) else {
        return Vec::new();
    };
    plans.push(last);
    let prize = match rng.range_u64(0, 100) < SET_ITEM_PERCENT {
        true => RewardPart::SetItem,
        false => RewardPart::Artifact,
    };
    let member = givers(city, buildings)
        .iter()
        .find(|g| g.building == building)
        .map_or(0, |g| u32::from(g.slots));
    let mut quests: Vec<Quest> = Vec::new();
    for (step, plan) in plans.iter().enumerate() {
        let step = step as u8;
        let id = QuestId {
            town: city.chunk,
            slot: Slot::Artifact { step },
        };
        let mut requirements = vec![Requirement::shown(Need::Standing {
            town: city.chunk,
            least: Standing::Honoured,
        })];
        requirements.extend(
            id.earlier()
                .map(|before| Requirement::shown(Need::Done(before))),
        );
        let last = usize::from(step) + 1 == plans.len();
        let making = Making {
            id,
            kind: QuestKind::Artifact,
            building,
            member,
            requirements,
            chain: step,
            earlier: quests.last().map(|q| &q.story),
            dark: false,
            continues: !last,
            text_seed: Seed::new(seed.derive("text").at(&[i64::from(step)])),
            prize: last.then_some(prize),
        };
        quests.push(make(world, city, plan, making, &mut rng));
    }
    quests
}

/// The line's giver: one of the city's givers that isn't a house, else any
/// giver, else any building.
fn giver(city: &Poi, buildings: &[BuildingLook], rng: &mut Rng) -> Option<u16> {
    let all = givers(city, buildings);
    let mut choices: Vec<u16> = (all.iter())
        .filter(|g| g.slots > 0 && g.kind != BuildingKind::House)
        .map(|g| g.building)
        .collect();
    if choices.is_empty() {
        choices = all.iter().map(|g| g.building).collect();
    }
    if choices.is_empty() {
        choices = buildings.iter().map(|b| b.id).collect();
    }
    let at = rng.range_u64(0, choices.len().max(1) as u64) as usize;
    choices.get(at).copied()
}

/// A whole quest round `neighbour`, on shared hooks only (as dailies are), so
/// the neighbour's own quests keep their bosses, items and lost ones.
fn plan_near(world: &QuestWorld, neighbour: &Poi, rng: &mut Rng) -> Option<Plan> {
    let candidates: Vec<Candidate> = town_candidates(world, neighbour, true);
    let weights: Vec<(usize, u64)> = (candidates.iter().enumerate())
        .map(|(i, c)| (i, c.weight))
        .collect();
    if weights.is_empty() {
        return None;
    }
    let main = candidates[rng.pick_weighted(&weights)];
    Some(plan(neighbour, &main, &candidates, rng))
}
