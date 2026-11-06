//! Mobs in the wilderness. Each player there is in a live area: the 5 × 5
//! blocks round the block it was centred on, with one population for all its
//! mobs (`(SiteId::Wilderness, area)`), shared by everyone in its middle 3 × 3.
//! A block belongs to one area at a time. A block an area lets go of keeps its
//! mobs where they were, resting, for `WILD_BLOCK_KEPT` (owner), and an area
//! taking it in again within that time has them back as they were; after it,
//! they are let go, to start again at home. Only what changes is saved: who
//! died, and what they remember or were given (`LevelSave` of the
//! wilderness's level 0).

mod blocks;
mod resets;
mod trails;

#[cfg(test)]
mod tests;

use shared::wilderness::BlockPos;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use super::Mob;
use crate::ServerWorld;
use crate::connections::{Client, Phase};
use crate::storage::SiteId;

pub(crate) use resets::touched;

/// How long a block nobody's area holds keeps its mobs (owner: so turning
/// back makes nothing vanish).
pub(crate) const WILD_BLOCK_KEPT: Duration = Duration::from_secs(5 * 60);

/// Blocks each side of an area's middle one: it holds 5 × 5.
const AREA_BLOCKS: i64 = 2;

/// Blocks each side of the middle in which a player stays in an area: its
/// middle 3 × 3, so they never see past its edge.
const INNER_BLOCKS: i64 = 1;

/// The wilderness's live areas, and the blocks they have let go of lately.
#[derive(Default)]
pub(crate) struct WildAreas {
    /// The next area's number (0 is "none yet").
    next: u32,
    areas: BTreeMap<u32, BlockPos>,
    /// Which area holds each block.
    owner: BTreeMap<BlockPos, u32>,
    /// Blocks let go of, with the mobs that were in them and since when.
    kept: BTreeMap<BlockPos, (Vec<Mob>, Instant)>,
    /// How many of each home block's mobs are about (in an area or kept), so
    /// a block's mobs are made only when none of them is.
    living: BTreeMap<BlockPos, u32>,
}

/// Puts each player in the wilderness in a live area, moves areas with those
/// who walk out of their middle, and lets go of what nobody needs. Every tick,
/// before mobs act.
pub(crate) fn keep_wild_areas(clients: &mut [Client], world: &mut ServerWorld) {
    let playing = |client: &&mut Client| client.phase == Phase::Playing && !client.is_gone();
    let mut players: Vec<&mut Client> = clients
        .iter_mut()
        .filter(playing)
        .filter(|client| client.visit.as_ref().is_some_and(|v| v.is_wilderness()))
        .collect();
    let mut members: BTreeMap<u32, u32> = BTreeMap::new();
    for client in &players {
        let area = client.visit.as_ref().map_or(0, |visit| visit.wild_area());
        *members.entry(area).or_default() += 1;
    }
    for client in &mut players {
        let Some(visit) = client.visit.as_mut() else {
            continue;
        };
        let at = visit.position();
        let block = world.wilderness.wild.block_of(at.0, at.1);
        let current = visit.wild_area();
        let areas = &world.wilderness.areas.areas;
        let inside = |centre: &BlockPos| near(world, *centre, block, INNER_BLOCKS);
        let chosen = if areas.get(&current).is_some_and(inside) {
            current
        } else if let Some((&other, _)) = areas.iter().find(|(_, centre)| inside(centre)) {
            other
        } else if areas.contains_key(&current) && members.get(&current) == Some(&1) {
            blocks::centre(world, current, block, at);
            current
        } else {
            let areas = &mut world.wilderness.areas;
            areas.next = areas.next.max(1);
            let new = areas.next;
            areas.next += 1;
            blocks::centre(world, new, block, at);
            new
        };
        if chosen != current {
            *members.entry(current).or_default() -= 1;
            *members.entry(chosen).or_default() += 1;
            visit.set_wild_area(chosen);
        }
    }
    let empty: Vec<u32> = world
        .wilderness
        .areas
        .areas
        .keys()
        .copied()
        .filter(|area| members.get(area).is_none_or(|&count| count == 0))
        .collect();
    for area in empty {
        blocks::close(world, area);
    }
    blocks::forget_old(world, Instant::now());
}

/// Whether two blocks are within `reach` of each other each way, the near
/// way round the planet.
fn near(world: &ServerWorld, a: BlockPos, b: BlockPos, reach: i64) -> bool {
    let columns = world.wilderness.wild.block_columns();
    let dx = (a.x - b.x).rem_euclid(columns);
    dx.min(columns - dx) <= reach && (a.y - b.y).abs() <= reach
}

/// The population of an area.
fn place(area: u32) -> (SiteId, u32) {
    (SiteId::Wilderness, area)
}
