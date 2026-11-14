//! Larger quest dungeons, by a hidden entrance near a town (owner): only the
//! quester sees it, on the world map in a chunk with no poi of its own,
//! marked like any quest poi. Towns' and cities' normal quests only; deeper
//! and wider than a cellar, with any creature of the wild places.

use super::dungeon::{DungeonWay, HIDDEN_KIND, QuestDungeon};
use super::hook::slain_with_weights;
use super::plan::Plan;
use super::world::size;
use super::{Hook, QuestWorld, Step, Target};
use crate::poi::{CHUNK_SIZE, EDGE_MARGIN, Poi, PoiKind};
use crate::rng::{Rng, Seed};
use crate::story::Goal;

/// Percent of a town's or city's normal quests that go down a hidden way,
/// after the cellars' share (`dungeon::NORMAL_PERCENT`). *To balance.*
pub(super) const HIDDEN_PERCENT: u64 = 8;
/// How many chunks from the town an entrance may be, and places tried.
const REACH: i64 = 3;
const TRIES: u32 = 24;

/// The plan for a hidden dungeon of `town`'s, if an empty chunk with dry land
/// is near enough: clear its creatures and reach its bottom, alongside.
/// `large`: the artifact line's last quest (owner: a large quest dungeon), 4
/// or 5 levels, wider and fuller (Claude's figures, *to balance*).
pub(super) fn plan_hidden(
    world: &QuestWorld,
    town: &Poi,
    large: bool,
    rng: &mut Rng,
) -> Option<Plan> {
    let bigger = size(town.kind).unwrap_or(0) as u64;
    let (x, y) = entrance(world, town, rng)?;
    let pois = world.pois;
    let levels = match large {
        true => rng.range_u64(4, 6),
        false => rng.range_u64(2, bigger.max(1) + 2),
    } as u32;
    let size = match large {
        true => (56 + 4 * bigger as i64, 68 + 6 * bigger as i64),
        false => (40 + 4 * bigger as i64, 52 + 6 * bigger as i64),
    };
    let creatures = rng.pick_weighted(&slain_with_weights(PoiKind::Ruins));
    let count = match large {
        true => rng.range_u64(12 + 3 * bigger, 18 + 4 * bigger),
        false => rng.range_u64(5 + 2 * bigger, 9 + 3 * bigger),
    } as u32;
    let per_level = (count.div_ceil(levels) + 1) as u8;
    let poi = Poi {
        kind: HIDDEN_KIND,
        chunk: pois.chunk_at(x, y),
        x,
        y,
        seed: Seed::new(rng.next_u64()),
        desert: false,
    };
    let dungeon = QuestDungeon {
        way: DungeonWay::Hidden,
        poi,
        levels,
        size,
        creatures,
        per_level,
    };
    let step = |goal, hook, count| Step {
        goal,
        target: Target::Dungeon {
            chunk: poi.chunk,
            hook,
            way: DungeonWay::Hidden,
        },
        count,
        after: Vec::new(),
        alternatives: Vec::new(),
    };
    Some(Plan {
        steps: vec![
            step(Goal::Slay, Hook::Kills(creatures), Some(count)),
            step(Goal::Explore, Hook::Explore, None),
        ],
        place: Some(poi),
        creatures: Some((creatures, count)),
        dungeon: Some(dungeon),
        ..Plan::default()
    })
}

/// A world point within `REACH` chunks of the town, in a chunk with no poi,
/// on open land and clear of its chunk's edges.
fn entrance(world: &QuestWorld, town: &Poi, rng: &mut Rng) -> Option<(i64, i64)> {
    let pois = world.pois;
    let span = (CHUNK_SIZE - 2 * EDGE_MARGIN) as u64;
    (0..TRIES).find_map(|_| {
        let dx = rng.range_u64(0, 2 * REACH as u64 + 1) as i64 - REACH;
        let dy = rng.range_u64(0, 2 * REACH as u64 + 1) as i64 - REACH;
        let (ox, oy) = (rng.range_u64(0, span) as i64, rng.range_u64(0, span) as i64);
        let west = (town.chunk.x + dx) * CHUNK_SIZE;
        let north = (town.chunk.y + dy) * CHUNK_SIZE;
        let x = (west + EDGE_MARGIN + ox).rem_euclid(pois.width());
        let y = north + EDGE_MARGIN + oy;
        let chunk = pois.chunk_at(x, y);
        let empty = chunk != town.chunk && pois.in_chunk(chunk).is_none();
        (empty && pois.is_open_land(x, y)).then_some((x, y))
    })
}
