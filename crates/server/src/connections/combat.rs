//! A player fighting: beginning an attack, its landing or fizzling, calling it
//! off, being hit by mobs, and being killed.

use bevy::prelude::*;
use shared::ability::Ability;
use shared::combat::{Body, CANCEL_PENALTY, Fighter, PRACTICE_PER_BLOW, Swing, Wielded, in_reach};
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::protocol::{Landed, ServerMessage};
use shared::skill::{familiarity, taught};

use super::Client;
use crate::ServerWorld;
use crate::combat::{Act, Pending, luck, ticks_for};
use crate::entities::{MobBlow, Striker, hit_mob};

impl Client {
    pub(super) fn fighter(&self) -> Fighter<'_> {
        Fighter {
            kind: self.inventory.creature,
            attributes: &self.sheet.attributes,
            skills: Some(&self.sheet.skills),
            wielded: Wielded::of(&self.inventory),
        }
    }

    /// A mob on the player's level where it stands, if it is there and in sight.
    pub(super) fn mob_in_sight(&mut self, world: &ServerWorld, mob: Id) -> Option<Body> {
        let visit = self.visit.as_mut()?;
        let population = world.populations.0.get(&visit.population())?;
        let one = population.mobs.iter().find(|one| one.id == mob)?;
        let (x, y) = one.position;
        let body = Body::on(population.relief(), one.spot, one.size());
        visit.can_see(x, y).then_some(body)
    }

    /// The player where they stand on their level, as far as reaching goes:
    /// `None` off levels.
    pub(super) fn body(&self, world: &ServerWorld) -> Option<Body> {
        let visit = self.visit.as_ref()?;
        let population = world.populations.0.get(&visit.population());
        let relief = population.and_then(|population| population.relief());
        Some(Body::on(relief, visit.spot(), self.size()))
    }

    /// Whether a swing from where the player stands reaches `target`
    /// (`combat::in_reach`).
    pub(super) fn reaches(&self, world: &ServerWorld, target: &Body, swing: &Swing) -> bool {
        let (Some(visit), Some(me)) = (self.visit.as_ref(), self.body(world)) else {
            return false;
        };
        in_reach(visit.cells(), &me, swing.weapon.reach(), target)
    }

    /// Practises these skills fighting a creature of this kind at action tick
    /// `now`, keeping the sheet to be saved. A kind fought often teaches less
    /// (`skill::taught`); each skill takes one practice every 5 seconds, and each
    /// kind counts once in that time as a block fought. Using them keeps them
    /// from rusting, and wears down what they oppose (`Skills::used`).
    pub(super) fn practise(
        &mut self,
        skills: impl IntoIterator<Item = shared::skill::SkillId>,
        foe: CreatureKind,
        now: u64,
    ) {
        let familiar = familiarity(self.sheet.foes.of(foe).fought);
        let played = self.activity.played;
        for skill in skills {
            let value = self.sheet.skills.value(skill);
            let amount = taught(PRACTICE_PER_BLOW, familiar, value);
            let amount = self.learning.skills.take(skill, amount, now);
            self.sheet.skills.practise(skill, amount);
            self.sheet.skills.used(skill, played);
        }
        if self.learning.foes.take(foe, 1, now) > 0 {
            self.sheet.foes.fought(foe);
        }
        self.unsaved = true;
    }
}

/// Begins an ability the client asked for, if it can be done now: Attack, at
/// a mob in sight and in reach, or Heal (`heal.rs`). Whether the client is
/// still there, and what it cost (nothing if it wasn't begun).
pub(super) fn start_ability(
    client: &mut Client,
    world: &mut ServerWorld,
    ability: Ability,
    target: Option<Id>,
) -> (bool, u32) {
    let kind = client.inventory.creature;
    if ability.is_toggle() && ability.can_use(kind) {
        return (client.toggle_sneak(), 0);
    }
    if !ability.can_use(kind) || client.fight.pending.is_some() {
        debug!("Not starting {ability:?}: can't, or already winding up");
        return (true, 0);
    }
    // Using an ability on someone turns the player to them.
    if let Some(at) = target.and_then(|target| client.mob_in_sight(world, target))
        && !client.face_spot(at.at)
    {
        return (false, 0);
    }
    if ability == Ability::Heal {
        return super::heal::start_heal(client, world, target);
    }
    if ability == Ability::Kick {
        return super::kick::start_kick(client, world, target);
    }
    let (Ability::Attack, Some(target)) = (ability, target) else {
        debug!("Not starting {ability:?}: not done yet, or no target");
        return (true, 0);
    };
    let Some(at) = client.mob_in_sight(world, target) else {
        debug!("Not starting {ability:?}: {target:?} isn't in sight");
        return (true, 0);
    };
    let swing = client.fighter().swing(client.fight.attacks);
    if !client.reaches(world, &at, &swing) {
        debug!("Not starting {ability:?}: {target:?} is out of reach");
        return (true, 0);
    }
    let tick = world.action_clock.ticks();
    let lands_in = ticks_for(swing.wind_up, client.speed);
    client.fight.pending = Some(Pending {
        target,
        act: Act::Strike(swing),
        lands_at: tick + lands_in,
    });
    let limits = client.limits(tick);
    client.sheet.vitals.in_combat(limits, tick);
    // So the client knows it is fighting (no changing floors).
    client.send_vitals(tick);
    let stayed = client.link.send(&ServerMessage::AbilityStarted {
        who: None,
        ability,
        target: Some(target),
        lands_in: lands_in as u32,
        ready_in: ticks_for(swing.cost, client.speed) as u32,
    });
    (stayed, swing.cost)
}

/// Every tick: lands the player's ability once its wind-up is over, on its
/// target if it is still in sight and reach, or fizzles. `false` if the client
/// has gone.
pub(super) fn land_ability(client: &mut Client, world: &mut ServerWorld) -> bool {
    let tick = world.action_clock.ticks();
    let Some(pending) = client.fight.pending.filter(|p| p.lands_at <= tick) else {
        return true;
    };
    client.fight.pending = None;
    let swing = match pending.act {
        Act::Strike(swing) => swing,
        Act::Heal => return super::heal::land_heal(client, world, pending.target),
        Act::Kick(swing) => return super::kick::land_kick(client, world, pending.target, swing),
        Act::KickDoor(at) => return super::kick::land_door_kick(client, world, at),
    };
    client.fight.attacks += 1;
    let target = pending.target;
    let at = client.mob_in_sight(world, target);
    let reached = at.filter(|at| client.reaches(world, at, &swing));
    let (Some(_), Some(visit)) = (reached, client.visit.as_ref()) else {
        return client.link.send(&ServerMessage::AbilityLanded {
            who: None,
            target: Some(target),
            landed: Landed::Fizzled,
        });
    };
    let place = (visit.population(), visit.index());
    let striker = Striker {
        id: client.entity_id(),
        position: visit.position(),
        fighter: client.fighter(),
        swing,
        kicked: None,
    };
    let practised = striker.fighter.practised_by(&swing);
    let Some(hit) = hit_mob(world, place, target, striker, tick) else {
        return true;
    };
    client.practise(practised, hit.kind, tick);
    debug!(
        "A player's blow did {:?} to {target:?}, {} per mille left",
        hit.harm, hit.left
    );
    let landed = Landed::Hit {
        harm: hit.harm,
        left: hit.left,
    };
    client
        .link
        .send(&ServerMessage::Sheet(client.sheet.clone()))
        && client.link.send(&ServerMessage::AbilityLanded {
            who: None,
            target: Some(target),
            landed,
        })
}

/// Calls off the player's ability if it hasn't landed: they can act again
/// after only `CANCEL_PENALTY`. `false` if the client has gone.
pub(super) fn cancel_ability(client: &mut Client, world: &ServerWorld) -> bool {
    let tick = world.action_clock.ticks();
    let Some(pending) = client.fight.pending.filter(|p| p.lands_at > tick) else {
        return true;
    };
    client.fight.pending = None;
    client.energy.cancel(tick, client.speed, CANCEL_PENALTY);
    client.link.send(&ServerMessage::AbilityLanded {
        who: None,
        target: Some(pending.target),
        landed: Landed::Cancelled,
    })
}

/// Tells players what mobs did to them this tick, harming them, and putting
/// out anyone killed.
pub(crate) fn take_mob_blows(
    clients: &mut [Client],
    world: &mut ServerWorld,
    blows: Vec<MobBlow>,
    tick: u64,
) {
    for blow in blows {
        let target = match &blow {
            MobBlow::Started { target, .. }
            | MobBlow::Landed { target, .. }
            | MobBlow::Fizzled { target, .. } => *target,
        };
        let Some(client) = clients.iter_mut().find(|c| c.entity_id() == target) else {
            continue;
        };
        let message = match blow {
            MobBlow::Started {
                mob,
                lands_in,
                ready_in,
                ..
            } => ServerMessage::AbilityStarted {
                who: Some(mob),
                ability: Ability::Attack,
                target: None,
                lands_in,
                ready_in,
            },
            MobBlow::Fizzled { mob, .. } => ServerMessage::AbilityLanded {
                who: Some(mob),
                target: None,
                landed: Landed::Fizzled,
            },
            MobBlow::Landed {
                mob,
                kind,
                attributes,
                wielded,
                swing,
                ..
            } => {
                let attacker = Fighter {
                    kind,
                    attributes: &attributes,
                    skills: None,
                    wielded,
                };
                let harm = attacker.harm(&swing, &client.fighter(), &mut luck(tick, mob, target));
                // Less on Easy, more on Hard and Ironman.
                let harm = client.playing_on().harm_taken(harm);
                let left = struck::struck(client, world, harm, kind, tick);
                debug!("{mob:?} did {harm:?} to a player, {left} per mille left");
                ServerMessage::AbilityLanded {
                    who: Some(mob),
                    target: None,
                    landed: Landed::Hit { harm, left },
                }
            }
        };
        client.link.send(&message);
    }
}

mod struck;

#[cfg(test)]
mod death_tests;
#[cfg(test)]
mod grudge_tests;
#[cfg(test)]
mod heal_tests;
#[cfg(test)]
mod kick_tests;
#[cfg(test)]
mod tests;
