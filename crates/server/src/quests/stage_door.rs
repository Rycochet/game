//! The locked stage door of the quest dungeon level the player is on
//! (`shared::quest::QuestDungeon::stage_door`, owner): until it is unlocked it
//! has the quest's marker (sent with the finds, `QuestNews::Finds`) and a
//! map dot.

use shared::protocol::{BossSign, FindMark};
use shared::quest::Quest;
use shared::spot::Spot;

use super::cache;
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::SiteId;

/// The stage door still locked on the player's level, and its quest.
pub(super) fn locked_stage(
    client: &Client,
    world: &mut ServerWorld,
) -> Option<((i64, i64), Quest)> {
    let visit = client.visit.as_ref()?;
    let SiteId::Quest { quest, .. } = visit.id() else {
        return None;
    };
    let quest = cache::quest(world, quest)?;
    let at = quest
        .dungeon?
        .stage_door(visit.index(), visit.site_level())?;
    visit.is_locked(at).then_some((at, quest))
}

/// Its marker, while in sight.
pub(super) fn stage_mark(client: &mut Client, world: &mut ServerWorld) -> Option<FindMark> {
    let ((x, y), quest) = locked_stage(client, world)?;
    let seen = client
        .visit
        .as_mut()
        .is_some_and(|visit| visit.can_see(x, y));
    seen.then_some(FindMark {
        at: Spot::middle_of((x, y)),
        sign: BossSign::Quest(quest.kind),
    })
}
