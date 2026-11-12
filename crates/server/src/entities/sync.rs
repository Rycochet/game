//! Telling each client what it can see of mobs and other players.

use shared::hall;
use shared::mob::EntityLook;
use shared::protocol::{ServerMessage, WORLD_MAP_SIGHT};
use shared::sheet::Feeling;
use shared::spot::{CELL_SPAN, Spot};
use std::collections::BTreeSet;
use std::time::Instant;

use super::tell::{Seen, tell};
use super::{KEPT_FOR, Someone, everyone_playing, world_gap};
use crate::ServerWorld;
use crate::combat::Harm;
use crate::connections::{Client, Phase};
use crate::hall::HALL_SIGHT;

/// Everyone playing who is in the Hall of Worlds, which all worlds share.
pub(crate) fn in_hall(clients: &mut [Client]) -> Vec<Someone> {
    let mut everyone = everyone_playing(clients);
    everyone.retain(|someone| someone.in_hall);
    everyone
}

/// Tells everyone playing in a world (`clients`) who has come into sight, moved
/// or gone, and lets go of levels nobody has been on for a while, once they have
/// settled. Those in the Hall see everyone there (`in_hall`), from any world.
pub(crate) fn sync(clients: &mut [Client], world: &mut ServerWorld, in_hall: &[Someone]) {
    let playing = |client: &&mut Client| client.phase == Phase::Playing;
    let mut everyone = everyone_playing(clients);
    everyone.retain(|someone| !someone.in_hall);

    let now = Instant::now();
    let in_use: BTreeSet<_> = everyone
        .iter()
        .filter_map(|someone| someone.level)
        .collect();
    world.populations.0.retain(|level, population| {
        // The wilderness's live areas keep their own (`wild`).
        if level.0 == crate::storage::SiteId::Wilderness {
            return true;
        }
        if in_use.contains(level) || !population.is_settled() {
            population.empty_since = None;
            return true;
        }
        now.duration_since(*population.empty_since.get_or_insert(now)) < KEPT_FOR
    });

    let width = world.width;
    for client in clients.iter_mut().filter(playing) {
        let here = client.visit.as_ref().map(|visit| visit.population());
        let me = client.character.world_position;
        let my_hall = client.character.hall.filter(|_| client.visit.is_none());
        let my_kind = client.inventory.creature;
        let detect = client.detect_monsters;
        // Only what is in the player's cone is told (`shared::facing`).
        let cone = client.cone();
        let ahead = |spot: Spot| cone.is_none_or(|cone| cone.sees(spot));
        let mut seen: Vec<Seen> = Vec::new();
        for someone in everyone.iter().chain(in_hall) {
            let in_sight = someone.id != client.entity_id()
                && someone.level == here
                && someone.in_hall == my_hall.is_some()
                && match (&mut client.visit, my_hall) {
                    (Some(visit), _) => {
                        visit.can_see(someone.position.0, someone.position.1) && ahead(someone.spot)
                    }
                    (None, Some(at)) => {
                        hall_gap(at, someone.position) <= HALL_SIGHT && ahead(someone.spot)
                    }
                    (None, None) => world_gap(me, someone.position, width) <= WORLD_MAP_SIGHT,
                };
            if in_sight {
                // In the map's measure: units on levels and in the Hall.
                let (x, y) = match my_hall {
                    // Counted from where this client's `x` starts, the near
                    // way round.
                    Some(at) => {
                        let mine = at.cell().0;
                        let shown = mine + toward(mine, someone.position.0) - client.hall_origin;
                        let at = someone
                            .spot
                            .offset((shown - someone.position.0) * CELL_SPAN, 0);
                        (at.x, at.y)
                    }
                    None if here.is_some() => (someone.spot.x, someone.spot.y),
                    None => someone.position,
                };
                let look = EntityLook {
                    id: someone.id,
                    kind: someone.look.0,
                    name: Some(someone.name.clone()).filter(|name| !name.is_empty()),
                    appearance: someone.look.1.clone(),
                    feeling: Feeling::None,
                    x,
                    y,
                    worn: someone.worn.clone(),
                    mark: someone.mark.clone(),
                    heading: someone.heading,
                };
                seen.push((look, someone.speed));
            }
        }
        if let (Some(level), Some(visit)) = (here, &mut client.visit)
            && let Some(population) = world.populations.0.get(&level)
        {
            for mob in &population.mobs {
                if detect || (visit.can_see(mob.position.0, mob.position.1) && ahead(mob.spot)) {
                    let (x, y) = (mob.spot.x, mob.spot.y);
                    let look = EntityLook {
                        id: mob.id,
                        kind: mob.kind,
                        name: None,
                        appearance: mob.appearance.clone(),
                        feeling: mob.feeling(my_kind, client.character_id.as_ref()),
                        x,
                        y,
                        worn: mob.shown.clone(),
                        mark: None,
                        heading: mob.heading,
                    };
                    seen.push((look, mob.speed()));
                }
            }
        }
        let on_level = here.is_some() || my_hall.is_some();
        let newly_seen = tell(client, seen, &world.died, on_level);
        client.practise_seeing(newly_seen, world.action_clock.ticks());
        tell_harmed(client, &world.harmed);
    }
    // Everyone who saw them die, or harmed, has been told.
    world.died.clear();
    world.harmed.clear();
    super::harmed_by::credit_kills(clients, world);
}

/// Tells a client of everyone it sees who was harmed, unless it was told
/// already (it struck the blow, or took it).
fn tell_harmed(client: &mut Client, harmed: &[Harm]) {
    let me = client.entity_id();
    for harm in harmed {
        if harm.told != me && client.seen_entities.contains_key(&harm.id) {
            client.link.send(&ServerMessage::Harmed {
                id: harm.id,
                left: harm.left,
            });
        }
    }
}

/// How far east `to` is from `from` in the Hall, the near way round (west is
/// negative).
fn toward(from: i64, to: i64) -> i64 {
    let across = hall::wrap_x(to - from);
    if across > hall::HALL_LENGTH / 2 {
        across - hall::HALL_LENGTH
    } else {
        across
    }
}

/// How far apart two places in the Hall are, the near way round.
fn hall_gap(a: Spot, b: (i64, i64)) -> i64 {
    let a = a.cell();
    let across = hall::wrap_x(a.0 - b.0);
    across
        .min(hall::HALL_LENGTH - across)
        .max((a.1 - b.1).abs())
}
