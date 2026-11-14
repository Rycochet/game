//! The maps' dots for an Escort step once its escortee has been met (owner:
//! lost, an arrow at the minimap's edge points to them). Before that, the
//! stairs towards their poi's deepest level, as for a boss (`dots`).

use shared::protocol::level_chunk;
use shared::quest::Quest;

use super::escortee::escortee_id;
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::Visit;
use crate::storage::{EscortAt, SiteId};

/// Where an escort's dots are on the player's level, once met: following,
/// the escortee there (in the wilderness, the way home); waiting, them on
/// their level, the remembered stairs towards it elsewhere in their site, or
/// their site's poi in the wilderness. `None` before they are met.
pub(super) fn escort_cells(
    client: &Client,
    visit: &Visit,
    world: &mut ServerWorld,
    quest: &Quest,
) -> Option<Vec<(i64, i64)>> {
    let state = client.character.quests.taken.get(&quest.id)?.escort?;
    let mob = escortee_id(client, quest.id);
    let in_poi = |world: &mut ServerWorld, chunk| {
        let poi = world.pois.in_chunk(chunk);
        poi.and_then(|poi| visit.wild_cell_of(&poi))
            .into_iter()
            .collect()
    };
    Some(match state {
        EscortAt::With if visit.is_wilderness() => in_poi(world, quest.id.town),
        EscortAt::With => match world.escortee_at(mob) {
            Some((place, spot)) if place == visit.population() => vec![spot.cell()],
            _ => Vec::new(),
        },
        EscortAt::Waiting { site, level, spot } => {
            if (site, level) == (visit.id(), visit.index()) {
                vec![spot.cell()]
            } else if site == visit.id() {
                visit.stairs_towards(level, |cell| remembered(client, visit, cell))
            } else if let SiteId::Poi(chunk) = site
                && visit.is_wilderness()
            {
                in_poi(world, chunk)
            } else {
                Vec::new()
            }
        }
    })
}

/// Whether the character remembers a cell of the level they are on.
fn remembered(client: &Client, visit: &Visit, (x, y): (i64, i64)) -> bool {
    let memory = client.character.memory.get(&visit.id());
    let level = memory.and_then(|levels| levels.get(&visit.index()));
    let (chunk, bit) = level_chunk(x, y);
    level.is_some_and(|masks| masks.get(&chunk).is_some_and(|m| m & (1 << bit) != 0))
}
