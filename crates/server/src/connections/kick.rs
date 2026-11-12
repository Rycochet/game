//! A player kicking (`Ability::Kick`, anyone with legs): at the target if it
//! is in sight and reach, shoving it half a cell away (or harming it twice as much
//! if it can't be shoved), or else at the closed door beside them, which
//! enough kicks break.

use bevy::prelude::*;
use shared::combat::{PRACTICE_PER_BLOW, Swing, kick_skill};
use shared::id::Id;
use shared::protocol::{Landed, ServerMessage};
use shared::skill::{familiarity, taught};

use super::Client;
use crate::ServerWorld;
use crate::combat::{Act, Pending, luck, ticks_for};
use crate::entities::{Striker, hit_mob, shove, shoved_to};
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
        client.reaches(world, &at, &swing).then_some(id)
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

/// A kick at a creature lands: shoved half a cell straight away from the player
/// if there is room, harmed (more if it wasn't shoved).
pub(super) fn land_kick(
    client: &mut Client,
    world: &mut ServerWorld,
    target: Id,
    swing: Swing,
) -> bool {
    let tick = world.action_clock.ticks();
    let at = client.mob_in_sight(world, target);
    let reached = at.filter(|at| client.reaches(world, at, &swing));
    let (Some(at), Some(visit)) = (reached, client.visit.as_ref()) else {
        return client.link.send(&ServerMessage::AbilityLanded {
            who: None,
            target: Some(target),
            landed: Landed::Fizzled,
        });
    };
    let place = (visit.population(), visit.index());
    let from = visit.position();
    // Half a cell away (owner, 2026-11-10), onto nobody.
    let shoved = shoved_to(world, place.0, target, visit.spot()).filter(|to| {
        let (x, y) = to.cell();
        let kept = (visit.id(), visit.index(), (visit.kept_at(x), y));
        to.cell() == at.at.cell() || !world.standing.iter().any(|&(cell, _)| cell == kept)
    });
    let free = shoved.is_some();
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
    if let Some(to) = shoved {
        shove(world, place.0, target, to);
    }
    client.practise(kick_skill(), hit.kind, tick);
    client
        .link
        .send(&ServerMessage::Sheet(client.sheet.as_sent()))
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
        .send(&ServerMessage::Sheet(client.sheet.as_sent()))
        && client.link.send(&ServerMessage::AbilityLanded {
            who: None,
            target: None,
            landed: Landed::Door { left },
        })
}
