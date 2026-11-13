//! Steps moving on: kills of a step's kind at its poi, and arriving at a
//! step's poi (visiting it, reaching its deepest level, or a town delivered
//! to). Only open steps count (`Quest::open_steps`). Bosses, quest items,
//! escorts and gathering count once those exist (later Quest system PRs).

use shared::creature::CreatureKind;
use shared::poi::ChunkPos;
use shared::quest::{Hook, QuestId, Step, Target};

use super::{cache, done_flags, log, tell_markers};
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::QuestPlace;

/// Something that may move a step on.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Event {
    /// The character shared in killing a creature of this kind, in the site of
    /// the poi in this chunk (`None` in the wilderness).
    Killed {
        kind: CreatureKind,
        site: Option<ChunkPos>,
    },
    /// The character has come to a level of a poi.
    Arrived(QuestPlace),
}

impl Event {
    /// Whether it counts for a step.
    fn counts_for(self, step: &Step) -> bool {
        match (self, step.target) {
            (Self::Killed { kind, site }, Target::Poi { chunk, hook }) => {
                hook == Hook::Kills(kind) && site == Some(chunk)
            }
            (Self::Arrived(place), Target::Poi { chunk, hook }) => {
                place.poi.chunk == chunk
                    && (hook == Hook::Visit || (hook == Hook::Explore && place.deepest))
            }
            (Self::Arrived(place), Target::Town { chunk }) => place.poi.chunk == chunk,
            _ => false,
        }
    }
}

/// Moves on every open step of the character's quests that `event` counts
/// for, telling the client. `false` if it has gone.
pub(crate) fn advance(
    client: &mut Client,
    world: &mut ServerWorld,
    event: Event,
    now: i64,
) -> bool {
    let ids: Vec<QuestId> = client.character.quests.taken.keys().copied().collect();
    let mut sent = true;
    for id in ids {
        if moved_on(client, world, id, event) {
            client.unsaved = true;
            sent &=
                log::tell_changed(client, world, id) && tell_markers(client, world, id.town, now);
        }
    }
    sent
}

/// Moves one quest's steps on; whether any did.
pub(crate) fn moved_on(
    client: &mut Client,
    world: &mut ServerWorld,
    id: QuestId,
    event: Event,
) -> bool {
    let Some(quest) = cache::quest(world, id) else {
        return false;
    };
    let Some(progress) = client.character.quests.taken.get_mut(&id) else {
        return false;
    };
    progress.counts.resize(quest.steps.len(), 0);
    let mut moved = false;
    for i in quest.open_steps(&done_flags(&quest, &progress.counts)) {
        let step = &quest.steps[i];
        if event.counts_for(step) {
            let most = step.count.unwrap_or(1);
            progress.counts[i] = (progress.counts[i] + 1).min(most);
            moved = true;
        }
    }
    moved
}

/// Sets one step of a quest taken as done (the debug API). Whether it was taken.
#[cfg(any(test, feature = "debug-api"))]
pub(crate) fn complete_step(
    client: &mut Client,
    world: &mut ServerWorld,
    id: QuestId,
    step: usize,
) -> bool {
    let Some(quest) = cache::quest(world, id) else {
        return false;
    };
    let Some(progress) = client.character.quests.taken.get_mut(&id) else {
        return false;
    };
    progress.counts.resize(quest.steps.len(), 0);
    if let (Some(count), Some(step)) = (progress.counts.get_mut(step), quest.steps.get(step)) {
        *count = step.count.unwrap_or(1);
    }
    client.unsaved = true;
    true
}
