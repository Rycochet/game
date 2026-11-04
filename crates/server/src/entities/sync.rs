//! Telling each client what it can see of mobs and other players.

use shared::id::Id;
use shared::mob::EntityLook;
use shared::protocol::{ServerMessage, WORLD_MAP_SIGHT};
use shared::sheet::Feeling;
use std::collections::BTreeSet;
use std::time::Instant;

use super::{KEPT_FOR, everyone_playing, world_gap};
use crate::ServerWorld;
use crate::connections::{Client, Phase};

/// Tells everyone playing who has come into sight, moved or gone, and lets go of
/// levels nobody has been on for a while, once they have settled.
pub(crate) fn sync(clients: &mut [Client], world: &mut ServerWorld) {
    let playing = |client: &&mut Client| client.phase == Phase::Playing;
    let everyone = everyone_playing(clients);

    let now = Instant::now();
    let in_use: BTreeSet<_> = everyone
        .iter()
        .filter_map(|someone| someone.level)
        .collect();
    world.populations.0.retain(|level, population| {
        if in_use.contains(level) || !population.is_settled() {
            population.empty_since = None;
            return true;
        }
        now.duration_since(*population.empty_since.get_or_insert(now)) < KEPT_FOR
    });

    let width = world.width;
    for client in clients.iter_mut().filter(playing) {
        let here = client
            .visit
            .as_ref()
            .map(|visit| (visit.id(), visit.index()));
        let me = client.character.world_position;
        let my_kind = client.inventory.creature;
        let detect = client.detect_monsters;
        let mut seen: Vec<EntityLook> = Vec::new();
        for someone in &everyone {
            let in_sight = someone.id != client.entity_id()
                && someone.level == here
                && match &mut client.visit {
                    Some(visit) => visit.can_see(someone.position.0, someone.position.1),
                    None => world_gap(me, someone.position, width) <= WORLD_MAP_SIGHT,
                };
            if in_sight {
                seen.push(EntityLook {
                    id: someone.id,
                    kind: someone.look.0,
                    appearance: someone.look.1.clone(),
                    feeling: Feeling::None,
                    x: someone.position.0,
                    y: someone.position.1,
                    worn: someone.worn.clone(),
                });
            }
        }
        if let (Some(level), Some(visit)) = (here, &mut client.visit)
            && let Some(population) = world.populations.0.get(&level)
        {
            for mob in &population.mobs {
                if detect || visit.can_see(mob.position.0, mob.position.1) {
                    seen.push(EntityLook {
                        id: mob.id,
                        kind: mob.kind,
                        appearance: mob.appearance.clone(),
                        feeling: mob.feeling(my_kind, client.character_id.as_ref()),
                        x: mob.position.0,
                        y: mob.position.1,
                        worn: mob.shown.clone(),
                    });
                }
            }
        }
        tell(client, seen);
    }
}

/// Tells a client how what it can see differs from what it knew. A client that
/// has gone is noticed when its messages are next read.
pub(super) fn tell(client: &mut Client, seen: Vec<EntityLook>) {
    let now_seen: BTreeSet<Id> = seen.iter().map(|look| look.id).collect();
    let feelings_changed: BTreeSet<Id> = seen
        .iter()
        .filter(|look| {
            let feeling = client.seen_feelings.get(&look.id);
            let worn = client.seen_worn.get(&look.id);
            feeling.is_some_and(|known| *known != look.feeling)
                || worn.is_some_and(|known| *known != look.worn)
        })
        .map(|look| look.id)
        .collect();
    client.seen_feelings = seen.iter().map(|look| (look.id, look.feeling)).collect();
    client.seen_worn = seen
        .iter()
        .map(|look| (look.id, look.worn.clone()))
        .collect();
    client.seen_entities.retain(|id, _| {
        let stays = now_seen.contains(id);
        if !stays {
            client.link.send(&ServerMessage::EntityGone { id: *id });
        }
        stays
    });
    for look in seen {
        let at = (look.x, look.y);
        let known = client.seen_entities.insert(look.id, at);
        // Told afresh, the token takes the new colour and the new picture.
        let known = known.filter(|_| !feelings_changed.contains(&look.id));
        match known {
            None => {
                client.link.send(&ServerMessage::EntitySeen(look));
            }
            Some(known) if known != at => {
                client.link.send(&ServerMessage::EntityMoved {
                    id: look.id,
                    x: at.0,
                    y: at.1,
                });
            }
            Some(_) => {}
        }
    }
}
