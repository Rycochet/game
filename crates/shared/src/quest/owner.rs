//! Which town's quest a poi's boss, item or lost one belongs to: one of the
//! towns that reach it, by a hash of the poi, so no two towns' quests ever
//! claim the same one, and no town needs another's quests to know.

use super::world::{MOST_RADIUS, pois_within, radius, within};
use super::{Hook, QuestWorld};
use crate::poi::{ChunkPos, Poi, PoiMap};

/// The town whose quests may use this hook of `poi`: `None` for a hook
/// anyone's quests may share, or one no town reaches.
pub fn owner(world: &QuestWorld, poi: &Poi, hook: Hook) -> Option<ChunkPos> {
    let towns: Vec<Poi> = (pois_within(world.pois, poi, MOST_RADIUS).into_iter())
        .filter(|p| p.kind.is_town())
        .collect();
    owner_among(world.pois, &towns, poi, hook)
}

/// As `owner`, choosing among `towns` (sorted by chunk), which must include
/// every town that reaches `poi`.
pub(super) fn owner_among(pois: &PoiMap, towns: &[Poi], poi: &Poi, hook: Hook) -> Option<ChunkPos> {
    if !hook.is_exclusive() {
        return None;
    }
    let reaching: Vec<ChunkPos> = (towns.iter())
        .filter(|town| within(pois, town, poi, radius(town.kind, false)))
        .map(|town| town.chunk)
        .collect();
    if reaching.is_empty() {
        return None;
    }
    let which = match hook {
        Hook::Boss => 0,
        Hook::Item => 1,
        _ => 2,
    };
    let hash = poi.seed.derive("quest hook").at(&[which]);
    Some(reaching[(hash % reaching.len() as u64) as usize])
}
