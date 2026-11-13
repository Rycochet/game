//! Standing in a town (`shared::quest::Standing`), from points: quests handed
//! in, and helpful acts round the town. The figures are Claude's, to balance
//! (Quest system PR 10 adds more acts: trading, gifts to beggars).

use shared::creature::{CreatureKind, Temper};
use shared::poi::{CHUNK_SIZE, ChunkPos, Poi};
use shared::quest::{Standing, radius};

use crate::ServerWorld;
use crate::storage::QuestBook;

/// The points each tier above Stranger starts at: a normal quest makes a
/// stranger Known, a second Trusted (a chain's third quest needs that).
const TIERS: [(Standing, u32); 4] = [
    (Standing::Known, 15),
    (Standing::Trusted, 30),
    (Standing::Honoured, 75),
    (Standing::Champion, 150),
];

/// Points for handing in a normal quest, and a daily.
pub(crate) const FOR_QUEST: u32 = 15;
pub(crate) const FOR_DAILY: u32 = 5;

/// Points for killing a hostile creature within a town's reach: kills alone
/// make a character no more than Known there.
const FOR_KILL: u32 = 1;
const KILLS_UP_TO: u32 = 15;

/// The tier `points` reach.
pub(crate) fn tier(points: u32) -> Standing {
    (TIERS.iter().rev())
        .find(|(_, least)| points >= *least)
        .map_or(Standing::Stranger, |(standing, _)| *standing)
}

/// A character's standing in a town.
pub(crate) fn standing(book: &QuestBook, town: ChunkPos) -> Standing {
    tier(book.standing.get(&town).copied().unwrap_or(0))
}

/// Adds points to a character's standing in a town.
pub(crate) fn add(book: &mut QuestBook, town: ChunkPos, points: u32) {
    let had = book.standing.entry(town).or_default();
    *had = had.saturating_add(points);
}

/// A creature of `kind` was killed at world grid point `at`: if hostile, each
/// town whose quests reach there thinks a little better of the killer.
/// Whether anything changed.
pub(crate) fn killed_near(
    book: &mut QuestBook,
    world: &ServerWorld,
    kind: CreatureKind,
    at: (i64, i64),
) -> bool {
    if kind.temper() != Temper::Hostile {
        return false;
    }
    let mut changed = false;
    for town in towns_reaching(world, at) {
        let had = book.standing.entry(town).or_default();
        if *had < KILLS_UP_TO {
            *had = (*had + FOR_KILL).min(KILLS_UP_TO);
            changed = true;
        }
    }
    changed
}

/// The towns whose quests reach `at` (`quest::radius`).
fn towns_reaching(world: &ServerWorld, (x, y): (i64, i64)) -> Vec<ChunkPos> {
    let reach = radius(shared::poi::PoiKind::City, false) * CHUNK_SIZE;
    let near = world
        .pois
        .in_area((x - reach, y - reach), (x + reach, y + reach));
    (near.into_iter())
        .filter(|poi: &Poi| poi.kind.is_town())
        .filter(|poi| {
            let reach = radius(poi.kind, false) * CHUNK_SIZE;
            let dx = (poi.x - x).rem_euclid(world.width.max(1));
            let dx = dx.min(world.width - dx);
            dx * dx + (poi.y - y).pow(2) <= reach * reach
        })
        .map(|poi| poi.chunk)
        .collect()
}

#[cfg(test)]
mod tests;
