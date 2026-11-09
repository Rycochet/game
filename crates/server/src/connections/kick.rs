//! A player kicking (`Ability::Kick`, anyone with legs): at the target if it
//! is in sight and reach, shoving it a step away (or harming it twice as much
//! if it can't be shoved), or else at the closed door beside them, which
//! enough kicks break.

use bevy::prelude::*;
use shared::combat::{PRACTICE_PER_BLOW, Swing, kick_skill};
use shared::id::Id;
use shared::movement::mob_can_stand;
use shared::protocol::{Landed, ServerMessage};
use shared::skill::{familiarity, taught};

use super::Client;
use crate::ServerWorld;
use crate::combat::{Act, Pending, luck, ticks_for};
use crate::entities::{Striker, hit_mob};
use crate::sites::{door_to_kick, kick_door};

/// Begins a kick, if there is something to kick: whether the client is still
/// there, and what it cost.
pub(super) fn start_kick(
    client: &mut Client,
    world: &mut ServerWorld,
    target: Option<Id>,
) -> (bool, u32) {
    let swing = client.fighter().kick();
    let creature = target.and_then(|id| {
        let at = client.mob_in_sight(world, id)?;
        client.reaches(at, &swing).then_some(id)
    });
    let (act, aimed) = match (creature, door_to_kick(client)) {
        (Some(id), _) => (Act::Kick(swing), Some(id)),
        (None, Some(door)) => (Act::KickDoor(door), None),
        (None, None) => {
            debug!("Not starting Kick: nothing in reach");
            return (true, 0);
        }
    };
    let tick = world.action_clock.ticks();
    let lands_in = ticks_for(swing.wind_up, client.speed);
    client.fight.pending = Some(Pending {
        target: aimed.unwrap_or_else(|| client.entity_id()),
        act,
        lands_at: tick + lands_in,
    });
    if aimed.is_some() {
        let limits = client.limits(tick);
        client.sheet.vitals.in_combat(limits, tick);
        client.send_vitals(tick);
    }
    let stayed = client.link.send(&ServerMessage::AbilityStarted {
        who: None,
        ability: shared::ability::Ability::Kick,
        target: aimed,
        lands_in: lands_in as u32,
        ready_in: ticks_for(swing.cost, client.speed) as u32,
    });
    (stayed, swing.cost)
}

/// A kick at a creature lands: shoved a step straight away from the player if
/// that cell is free, harmed (more if it wasn't shoved).
pub(super) fn land_kick(
    client: &mut Client,
    world: &mut ServerWorld,
    target: Id,
    swing: Swing,
) -> bool {
    let tick = world.action_clock.ticks();
    let at = client.mob_in_sight(world, target);
    let reached = at.filter(|&at| client.reaches(at, &swing));
    let (Some(at), Some(visit)) = (reached, client.visit.as_ref()) else {
        return client.link.send(&ServerMessage::AbilityLanded {
            who: None,
            target: Some(target),
            landed: Landed::Fizzled,
        });
    };
    let place = (visit.population(), visit.index());
    let from = visit.position();
    let to = (
        at.0 + (at.0 - from.0).signum(),
        at.1 + (at.1 - from.1).signum(),
    );
    let kept = (visit.id(), visit.index(), (visit.kept_at(to.0), to.1));
    let player_there = world.standing.iter().any(|&(cell, _)| cell == kept);
    let population = world.populations.0.get(&place.0);
    let free = population.is_some_and(|population| {
        mob_can_stand(population.level(), to.0, to.1)
            && !population.mobs.iter().any(|mob| mob.position == to)
    }) && !player_there
        && from != to;
    let striker = Striker {
        id: client.entity_id(),
        position: from,
        fighter: client.fighter(),
        swing,
        kicked: Some(free),
    };
    let Some(hit) = hit_mob(world, place, target, striker, tick) else {
        return true;
    };
    if free
        && let Some(population) = world.populations.0.get_mut(&place.0)
        && let Some(mob) = population.mobs.iter_mut().find(|mob| mob.id == target)
    {
        mob.position = to;
    }
    client.practise(kick_skill(), hit.kind, tick);
    client
        .link
        .send(&ServerMessage::Sheet(client.sheet.clone()))
        && client.link.send(&ServerMessage::AbilityLanded {
            who: None,
            target: Some(target),
            landed: Landed::Hit {
                harm: hit.harm,
                left: hit.left,
            },
        })
}

/// A kick at a door lands, if it is still beside the player and closed.
pub(super) fn land_door_kick(client: &mut Client, world: &mut ServerWorld, at: (i64, i64)) -> bool {
    let tick = world.action_clock.ticks();
    let me = client.entity_id();
    let force = client.fighter().kick_force(&mut luck(tick, me, me));
    let Some(left) = kick_door(client, world, at, force.0) else {
        return client.link.send(&ServerMessage::AbilityLanded {
            who: None,
            target: None,
            landed: Landed::Fizzled,
        });
    };
    // Practice, as against a creature, but no creature is fought.
    if let Some(skill) = kick_skill() {
        let value = client.sheet.skills.value(skill);
        let amount = taught(PRACTICE_PER_BLOW, familiarity(0), value);
        let amount = client.learning.skills.take(skill, amount, tick);
        client.sheet.skills.practise(skill, amount);
        client.sheet.skills.used(skill, client.activity.played);
        client.unsaved = true;
    }
    client
        .link
        .send(&ServerMessage::Sheet(client.sheet.clone()))
        && client.link.send(&ServerMessage::AbilityLanded {
            who: None,
            target: None,
            landed: Landed::Door { left },
        })
}
