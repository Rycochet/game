//! Steps moving on: kills of a step's kind at its poi, and arriving at a
//! step's poi (visiting it, reaching its deepest level, or a town its parcel
//! is delivered to). Only open steps count (`Quest::open_steps`). Bosses, quest items,
//! escorts and gathering count once those exist (later Quest system PRs).

use shared::creature::CreatureKind;
use shared::poi::ChunkPos;
use shared::quest::{Hook, QuestId, Step, Target};

use super::{cache, done_flags, log, parcels, tell_markers};
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
        /// In a quest's own dungeon: that quest's (`site` is then `None`).
        dungeon: Option<QuestId>,
    },
    /// The character has come to a level of a poi.
    Arrived(QuestPlace),
}

impl Event {
    /// Whether it counts for a step of quest `id`.
    fn counts_for(self, id: QuestId, step: &Step) -> bool {
        match (self, step.target) {
            (Self::Killed { kind, site, .. }, Target::Poi { chunk, hook }) => {
                hook == Hook::Kills(kind) && site == Some(chunk)
            }
            (Self::Killed { kind, dungeon, .. }, Target::Dungeon { hook, .. }) => {
                hook == Hook::Kills(kind) && dungeon == Some(id)
            }
            (Self::Arrived(place), _) if place.dungeon.is_some() => {
                let deepest = place.deepest && place.dungeon == Some(id);
                matches!(
                    step.target,
                    Target::Dungeon {
                        hook: Hook::Explore,
                        ..
                    }
                ) && deepest
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
    let mut delivered = false;
    for i in quest.open_steps(&done_flags(&quest, &progress.counts)) {
        let step = &quest.steps[i];
        if event.counts_for(id, step) {
            // A delivery needs its parcel, which is handed over on arriving.
            if matches!(step.target, Target::Town { .. })
                && !parcels::deliver(&mut client.inventory, id)
            {
                continue;
            }
            delivered |= matches!(step.target, Target::Town { .. });
            let most = step.count.unwrap_or(1);
            progress.counts[i] = (progress.counts[i] + 1).min(most);
            moved = true;
        }
    }
    progress.ready = quest.ready_to_turn_in(&done_flags(&quest, &progress.counts));
    if delivered {
        // A client gone is noticed when its messages are next read.
        client.update_speed(world.action_clock.ticks());
        client.send_inventory();
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
    progress.ready = quest.ready_to_turn_in(&done_flags(&quest, &progress.counts));
    client.unsaved = true;
    true
}
