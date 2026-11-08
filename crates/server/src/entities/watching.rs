//! Whether something wary is watching a player: what Sneaking is learned
//! against (owner: a timid or territorial creature, not out to attack them,
//! that can see them).

use shared::creature::{CreatureKind, Temper};
use shared::dungeon::line_is_clear;
use shared::protocol::CharacterId;
use shared::sheet::Feeling;

use super::notice::cells_apart;
use crate::ServerWorld;
use crate::storage::SiteId;

/// Whether any timid or territorial mob on level `place` that isn't
/// unfriendly to the player (of kind `kind`, character `character`) at `at`
/// can see them, by its kind's sight and a clear line.
pub(crate) fn watched_warily(
    world: &ServerWorld,
    place: (SiteId, u32),
    at: (i64, i64),
    kind: CreatureKind,
    character: Option<&CharacterId>,
) -> bool {
    let Some(population) = world.populations.0.get(&place) else {
        return false;
    };
    population.mobs.iter().any(|mob| {
        matches!(mob.kind.temper(), Temper::Timid | Temper::Territorial)
            && mob.feeling(kind, character) != Feeling::Unfriendly
            && cells_apart(mob.position, at) <= mob.kind.senses().sight_range()
            && line_is_clear(&population.level, mob.position, at)
    })
}
