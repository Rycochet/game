//! Chains across towns (owner): a quest may only need one from a smaller
//! town joined to its own by road, so City → Town → Village → Hamlet at
//! most, and nothing is generated recursively. Each town has one feeder
//! quest, fixed by its own seed, taking something to one of its larger
//! neighbours, which only reads the larger town's place and name.

use super::QuestWorld;
use super::world::is_larger;
use crate::poi::Poi;

/// The larger town joined by road that `town`'s feeder quest goes to, if
/// it has one: by a hash of its own seed among them.
pub fn feeder_destination(world: &QuestWorld, town: &Poi) -> Option<Poi> {
    let larger: Vec<Poi> = (linked(world, town).into_iter())
        .filter(|other| is_larger(other, town))
        .collect();
    if larger.is_empty() {
        return None;
    }
    let hash = town.seed.derive("quests").derive("feeder").value();
    Some(larger[(hash % larger.len() as u64) as usize])
}

/// The smaller towns joined to `town` by road whose feeders lead to it: its
/// quests may need theirs done.
pub fn feeds_into(world: &QuestWorld, town: &Poi) -> Vec<Poi> {
    (linked(world, town).into_iter())
        .filter(|other| is_larger(town, other))
        .filter(|other| feeder_destination(world, other).is_some_and(|d| d.chunk == town.chunk))
        .collect()
}

/// The towns joined to `town` by road, by chunk. A link counts whether or
/// not water cuts its road (routing them all would be slow).
fn linked(world: &QuestWorld, town: &Poi) -> Vec<Poi> {
    let at = (town.x, town.y);
    (world.roads.links_in_area(world.pois, at, at).into_iter())
        .filter_map(|(link, _)| {
            let other = if link.a == town.chunk {
                link.b
            } else if link.b == town.chunk {
                link.a
            } else {
                return None;
            };
            world.pois.in_chunk(other)
        })
        .collect()
}
