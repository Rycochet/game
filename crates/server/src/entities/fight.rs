//! Mobs fighting: attacking an enemy in reach, and being hit by a player.

use shared::combat::{Fighter, Swing, Wielded, in_reach};
use shared::creature::CreatureKind;
use shared::dungeon::Level;
use shared::id::Id;
use shared::sheet::Attributes;
use shared::vitals::{Points, Vital};

use super::gear::bury;
use super::{ATTACK_GAP, Doing, Mob, Spot};
use crate::ServerWorld;
use crate::combat::{Pending, health_left, luck, ticks_for};
use crate::storage::SiteId;

/// What a mob did to a player this tick, for the player's client to be told
/// once every level has had its turn.
pub(crate) enum MobBlow {
    Started {
        mob: Id,
        target: Id,
        lands_in: u32,
        ready_in: u32,
    },
    Landed {
        mob: Id,
        target: Id,
        kind: CreatureKind,
        attributes: Attributes,
        wielded: Wielded,
        swing: Swing,
    },
    Fizzled {
        mob: Id,
        target: Id,
    },
}

impl Mob {
    fn fighter(&self) -> Fighter<'_> {
        Fighter {
            kind: self.kind,
            attributes: &self.attributes,
            skills: None,
            wielded: self.wielded,
        }
    }

    /// How much Health it has at this tick.
    pub(crate) fn health(&self, tick: u64) -> Points {
        let limits = self.vitals.limits(&self.attributes, 0, tick);
        self.vitals.at(Vital::Health, limits, tick)
    }

    /// When it is next due: when its blow lands, or when it can act again.
    pub(super) fn next_due(&self, tick: u64) -> u64 {
        match self.fight.pending {
            Some(pending) => pending.lands_at.max(tick + 1),
            None => self.energy.ready_at(tick + 1, self.speed),
        }
    }
}

/// A mob's turn in a fight, if it is in one: its blow landing, or a new one
/// begun at an enemy in reach. What it cost; `None` to go about its business.
pub(super) fn mob_turn(
    mob: &mut Mob,
    level: &Level,
    spots: &[Spot],
    tick: u64,
    blows: &mut Vec<MobBlow>,
) -> Option<u32> {
    let reaches = |mob: &Mob, target: Id, swing: &Swing| {
        spots
            .iter()
            .find(|spot| spot.id == target)
            .is_some_and(|spot| in_reach(level, mob.position, spot.position, swing.weapon.reach()))
    };
    if let Some(pending) = mob.fight.pending.take() {
        mob.fight.attacks += 1;
        let target = pending.target;
        blows.push(if reaches(mob, target, &pending.swing) {
            MobBlow::Landed {
                mob: mob.id,
                target,
                kind: mob.kind,
                attributes: mob.attributes,
                wielded: mob.wielded,
                swing: pending.swing,
            }
        } else {
            MobBlow::Fizzled {
                mob: mob.id,
                target,
            }
        });
        return Some(0);
    }
    // Only one after an enemy: closing in to attack.
    let Doing::Going { after, gap, .. } = mob.doing else {
        return None;
    };
    if gap != ATTACK_GAP {
        return None;
    }
    let swing = mob.fighter().swing(mob.fight.attacks);
    if !reaches(mob, after, &swing) {
        return None;
    }
    let lands_in = ticks_for(swing.wind_up, mob.speed);
    mob.fight.pending = Some(Pending {
        target: after,
        swing,
        lands_at: tick + lands_in,
    });
    blows.push(MobBlow::Started {
        mob: mob.id,
        target: after,
        lands_in: lands_in as u32,
        ready_in: ticks_for(swing.cost, mob.speed) as u32,
    });
    Some(swing.cost)
}

/// What a player's blow did to a mob.
pub(crate) struct MobHit {
    pub harm: Points,
    /// Health left, per mille: none when it died.
    pub left: u16,
    /// What kind of creature it is.
    pub kind: CreatureKind,
}

/// Who struck a mob, and with what.
pub(crate) struct Striker<'a> {
    pub id: Id,
    pub position: (i64, i64),
    pub fighter: Fighter<'a>,
    pub swing: Swing,
}

/// A player's blow lands on a mob on `place` (whose own level is `here_index`):
/// it is harmed, turns on them, and dies if that was the last of its Health.
/// `None` if it is no longer there.
pub(crate) fn hit_mob(
    world: &mut ServerWorld,
    (place, here_index): ((SiteId, u32), u32),
    mob: Id,
    by: Striker,
    tick: u64,
) -> Option<MobHit> {
    let population = world.populations.0.get_mut(&place)?;
    let one = population.mobs.iter_mut().find(|one| one.id == mob)?;
    let mut rng = luck(tick, by.id, mob);
    let harm = by.fighter.harm(&by.swing, &one.fighter(), &mut rng);
    let limits = one.vitals.limits(&one.attributes, 0, tick);
    one.vitals.hurt(harm, limits, tick, 0);
    let left = health_left(&one.vitals, limits, tick);
    let kind = one.kind;
    // Whatever it was doing, it goes for whoever hit it.
    let after_them = matches!(one.doing, Doing::Going { after, .. } if after == by.id);
    if !after_them {
        let at_rest = one.doing == Doing::Idle;
        one.doing = Doing::Going {
            after: by.id,
            last_seen: by.position,
            gap: ATTACK_GAP,
        };
        one.target = Some(by.id);
        if at_rest {
            let due = one.next_due(tick);
            population.due.insert((due, mob));
        }
    }
    if one.health(tick) == Points::ZERO {
        bury(world, place, here_index, mob);
    }
    Some(MobHit { harm, left, kind })
}
