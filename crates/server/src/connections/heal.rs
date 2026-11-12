//! A player casting Heal (`shared::magic`): on themselves, or on a mob or
//! another player in sight within `HEAL_RANGE`, enemies too (taming is to
//! build on it). The Mana is spent when it lands, so a Heal that fizzles costs
//! only the time.

use bevy::prelude::*;
use shared::ability::Ability;
use shared::combat::Body;
use shared::id::Id;
use shared::magic::{
    HEAL_COST, HEAL_MANA, HEAL_WIND_UP, PRACTICE_PER_CAST, heal_amount, heal_reaches, heal_skill,
};
use shared::protocol::{Landed, ServerMessage};
use shared::vitals::{Points, Vital};

use super::Client;
use crate::combat::{Act, Pending, health_left, ticks_for};
use crate::entities::heal_mob;
use crate::{ServerWorld, ServerWorlds};

impl Client {
    /// Whether there is the Mana for a Heal at this tick.
    fn has_heal_mana(&self, now: u64) -> bool {
        let limits = self.limits(now);
        self.sheet.vitals.at(Vital::Mana, limits, now) >= Points::whole(HEAL_MANA)
    }

    /// Spends a Heal's Mana, practising Healing. Whether there was enough.
    fn pay_for_heal(&mut self, now: u64) -> bool {
        let limits = self.limits(now);
        let mana = Points::whole(HEAL_MANA);
        let played = self.activity.played;
        if !self.sheet.vitals.spend_mana(mana, limits, now, played) {
            return false;
        }
        let practice = self
            .learning
            .skills
            .take(heal_skill(), PRACTICE_PER_CAST, now);
        self.sheet.skills.practise(heal_skill(), practice);
        self.sheet.skills.used(heal_skill(), played);
        self.unsaved = true;
        true
    }

    /// Whether a Heal from where the player stands reaches `target`.
    fn heal_reaches(&self, world: &ServerWorld, target: &Body) -> bool {
        let (Some(visit), Some(me)) = (self.visit.as_ref(), self.body(world)) else {
            return false;
        };
        heal_reaches(visit.cells(), &me, target)
    }

    /// Takes this much healing: how much Health there is now, per mille.
    fn take_healing(&mut self, amount: Points, now: u64) -> u16 {
        let limits = self.limits(now);
        self.sheet
            .vitals
            .restore(Vital::Health, amount, limits, now);
        self.unsaved = true;
        health_left(&self.sheet.vitals, limits, now)
    }

    /// Tells the player their Heal landed (`target` `None`: on themselves),
    /// with their sheet (practice) and vitals (Mana). `false` if they have gone.
    fn healed(&mut self, target: Option<Id>, amount: Points, left: u16, now: u64) -> bool {
        self.link.send(&ServerMessage::Sheet(self.sheet.as_sent()))
            && self.send_vitals(now)
            && self.link.send(&ServerMessage::AbilityLanded {
                who: None,
                target,
                landed: Landed::Healed { amount, left },
            })
    }

    /// Tells the player their Heal fizzled. `false` if they have gone.
    fn heal_fizzled(&mut self, target: Option<Id>) -> bool {
        self.link.send(&ServerMessage::AbilityLanded {
            who: None,
            target,
            landed: Landed::Fizzled,
        })
    }
}

/// Begins a Heal at `target` (`None` or their own id: themselves), if there is
/// the Mana: a mob must be in sight and range now; another player is looked
/// for when it lands. Whether the client is still there, and what it cost.
pub(super) fn start_heal(
    client: &mut Client,
    world: &mut ServerWorld,
    target: Option<Id>,
) -> (bool, u32) {
    let tick = world.action_clock.ticks();
    let me = client.entity_id();
    let target = target.filter(|target| *target != me);
    if !client.has_heal_mana(tick) {
        debug!("Not starting a Heal: not enough Mana");
        return (true, 0);
    }
    if let Some(other) = target {
        let reaches = match client.mob_in_sight(world, other) {
            Some(at) => client.heal_reaches(world, &at),
            None => client.visit.is_some(),
        };
        if !reaches {
            debug!("Not starting a Heal: {other:?} is out of sight or range");
            return (true, 0);
        }
    }
    let lands_in = ticks_for(HEAL_WIND_UP, client.speed);
    client.fight.pending = Some(Pending {
        target: target.unwrap_or(me),
        act: Act::Heal,
        lands_at: tick + lands_in,
    });
    let stayed = client.link.send(&ServerMessage::AbilityStarted {
        who: None,
        ability: Ability::Heal,
        target,
        lands_in: lands_in as u32,
        ready_in: ticks_for(HEAL_COST, client.speed) as u32,
    });
    (stayed, HEAL_COST)
}

/// A Heal's wind-up is over: on themselves or a mob now, on another player
/// once every client has had its turn (`heal_players`). `false` if the client
/// has gone.
pub(super) fn land_heal(client: &mut Client, world: &mut ServerWorld, target: Id) -> bool {
    let tick = world.action_clock.ticks();
    // By the skill before this cast's practice.
    let amount = heal_amount(Some(&client.sheet.skills));
    if target == client.entity_id() {
        if !client.pay_for_heal(tick) {
            return client.heal_fizzled(None);
        }
        let left = client.take_healing(amount, tick);
        return client.healed(None, amount, left, tick);
    }
    let Some(at) = client.mob_in_sight(world, target) else {
        client.fight.heal_out = Some(target);
        return true;
    };
    let Some(place) = client.visit.as_ref().map(|visit| visit.population()) else {
        return client.heal_fizzled(Some(target));
    };
    if !client.heal_reaches(world, &at) || !client.pay_for_heal(tick) {
        return client.heal_fizzled(Some(target));
    }
    match heal_mob(world, place, target, amount, tick) {
        Some(left) => client.healed(Some(target), amount, left, tick),
        None => client.heal_fizzled(Some(target)),
    }
}

/// Once every client has had its turn: Heals landing on other players, if
/// they are still on the healer's level, in sight and in range.
pub(super) fn heal_players(clients: &mut [Client], worlds: &mut ServerWorlds) {
    for i in 0..clients.len() {
        let Some(target) = clients[i].fight.heal_out.take() else {
            continue;
        };
        let world = clients[i].world;
        let tick = worlds.of(world).action_clock.ticks();
        let found = clients
            .iter()
            .position(|other| !other.gone && other.world == world && other.entity_id() == target);
        match found.filter(|&j| j != i) {
            Some(j) => {
                let (caster, other) = pair(clients, i, j);
                heal_player(caster, other, worlds.of(world), tick);
            }
            None => {
                clients[i].heal_fizzled(Some(target));
            }
        }
    }
}

/// Two different clients at once.
pub(super) fn pair(clients: &mut [Client], i: usize, j: usize) -> (&mut Client, &mut Client) {
    if i < j {
        let (low, high) = clients.split_at_mut(j);
        (&mut low[i], &mut high[0])
    } else {
        let (low, high) = clients.split_at_mut(i);
        (&mut high[0], &mut low[j])
    }
}

/// A Heal from `caster` landing on `other`, or fizzling.
fn heal_player(caster: &mut Client, other: &mut Client, world: &ServerWorld, tick: u64) {
    let target = other.entity_id();
    let same_level = other
        .visit
        .as_ref()
        .zip(caster.visit.as_ref())
        .is_some_and(|(theirs, mine)| theirs.population() == mine.population());
    let at = other.body(world).filter(|_| same_level);
    let seen = at.filter(|body| {
        let (x, y) = body.at.cell();
        caster
            .visit
            .as_mut()
            .is_some_and(|visit| visit.can_see(x, y))
            && caster.heal_reaches(world, body)
    });
    let amount = heal_amount(Some(&caster.sheet.skills));
    if seen.is_none() || !caster.pay_for_heal(tick) {
        caster.heal_fizzled(Some(target));
        return;
    }
    let left = other.take_healing(amount, tick);
    other
        .link
        .send(&ServerMessage::Sheet(other.sheet.as_sent()));
    other.send_vitals(tick);
    caster.healed(Some(target), amount, left, tick);
}
