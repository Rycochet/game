//! Someone escorted on the world map: only their quester sees them (owner),
//! a step behind, on the point the quester last came from (Claude).

use shared::mob::EntityLook;
use shared::sheet::Feeling;

use super::escortee::{escort_step, escortee_id, spawn, who};
use super::{cache, done_flags};
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::EscortAt;

/// Where the player was on the world map: the point they stand on, and the
/// one they came to it from (`Seen::trail`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Trail {
    at: Option<(i64, i64)>,
    behind: Option<(i64, i64)>,
}

impl Trail {
    /// Where someone a step behind is, now the player stands at `at`.
    fn follow(&mut self, at: (i64, i64)) -> (i64, i64) {
        if self.at != Some(at) {
            self.behind = self.at.or(Some(at));
            self.at = Some(at);
        }
        self.behind.unwrap_or(at)
    }
}

/// What the player sees of someone they escort on the world map, if they
/// are on it with them: their look a step behind, and how fast they go (the
/// player's pace, as the pair go together).
pub(crate) fn world_look(
    client: &mut Client,
    world: &mut ServerWorld,
) -> Option<(EntityLook, u32)> {
    if client.visit.is_some() || client.character.hall.is_some() {
        client.quest_seen.trail = Trail::default();
        return None;
    }
    let book = &client.character.quests;
    let id = (book.taken.iter())
        .find_map(|(id, p)| (p.escort == Some(EscortAt::With)).then_some(*id))?;
    let quest = cache::quest(world, id)?;
    let progress = client.character.quests.taken.get(&id)?;
    let done = done_flags(&quest, &progress.counts);
    let (step, _) = escort_step(&quest)?;
    if !quest.open_steps(&done).contains(&step) {
        return None;
    }
    let who = who(&quest)?;
    let (x, y) = client.quest_seen.trail.follow(client.world_position());
    let spawn = spawn(&who, (x, y));
    let worn = crate::entities::escortee_worn(world, &spawn);
    let look = EntityLook {
        id: escortee_id(client, id),
        kind: who.kind,
        gender: who.gender,
        name: Some(who.name),
        appearance: spawn.appearance,
        feeling: Feeling::Friendly,
        x,
        y,
        worn,
        mark: None,
        heading: shared::facing::Heading(0),
        boss: None,
    };
    Some((look, client.speed))
}
