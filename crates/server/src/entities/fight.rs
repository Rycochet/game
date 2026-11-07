//! Mobs fighting: attacking an enemy in reach, and being hit by a player.

use shared::combat::{Fighter, Swing, Wielded, in_reach};
use shared::creature::CreatureKind;
use shared::dungeon::Level;
use shared::id::Id;
use shared::protocol::CharacterId;
use shared::sheet::Attributes;
use shared::sheet::Feeling;
use shared::vitals::{Points, Vital};

use super::gear::{bury, record_of};
use super::timid::{is_timid, strikes_at};
use super::{ATTACK_GAP, Doing, FRIENDLY_GAP, Mob, SETTLING_STEPS, Spot};
use crate::ServerWorld;
use crate::combat::{Act, Harm, Pending, health_left, luck, ticks_for};
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
        let Act::Strike(swing) = pending.act else {
            return Some(0);
        };
        mob.fight.attacks += 1;
        let target = pending.target;
        blows.push(if reaches(mob, target, &swing) {
            MobBlow::Landed {
                mob: mob.id,
                target,
                kind: mob.kind,
                attributes: mob.attributes,
                wielded: mob.wielded,
                swing,
            }
        } else {
            MobBlow::Fizzled {
                mob: mob.id,
                target,
            }
        });
        return Some(0);
    }
    // One after an enemy, closing in to attack, or a timid one caught near one.
    let timid = strikes_at(mob, spots).map(|spot| spot.id);
    let after = match (timid, mob.doing) {
        (Some(from), _) => from,
        (None, Doing::Going { after, gap, .. }) if gap == ATTACK_GAP => after,
        _ => return None,
    };
    let swing = mob.fighter().swing(mob.fight.attacks);
    if !reaches(mob, after, &swing) {
        return None;
    }
    let lands_in = ticks_for(swing.wind_up, mob.speed);
    mob.fight.pending = Some(Pending {
        target: after,
        act: Act::Strike(swing),
        lands_at: tick + lands_in,
    });
    if timid.is_some() {
        // Once: it steps away before it may strike again.
        mob.doing = Doing::Fleeing {
            from: after,
            struck: true,
        };
    }
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
/// it is harmed, remembers it (`Memory::remember_harm`, saved), does what its
/// standing with them now says (`after_hit`), and dies if that was the last of
/// its Health. `None` if it is no longer there.
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
    one.harmed_by.harmed(by.id, tick);
    if harm != Points::ZERO {
        world.harmed.push(Harm {
            id: mob,
            left,
            told: by.id,
        });
    }
    // A player's entity id is their character's.
    let character = CharacterId(by.id);
    let start = one.kind.feeling_about(by.fighter.kind);
    one.memory.remember_harm(character, harm, start);
    let feeling = one.feeling(by.fighter.kind, Some(&character));
    let at_rest = one.doing == Doing::Idle;
    if let Some(doing) = after_hit(one, &by, feeling) {
        one.doing = doing;
        one.target = doing.target();
        if at_rest && doing != Doing::Idle {
            let due = one.next_due(tick);
            population.due.insert((due, mob));
        }
    }
    let keep = (one.id, one.home_level, one.home);
    let memory = one.memory.clone();
    let dead = one.health(tick) == Points::ZERO;
    record_of(world, place.0, keep).memory = memory;
    if dead {
        bury(world, place, here_index, mob, true);
    }
    Some(MobHit { harm, left, kind })
}

/// What a mob does once hit, by how it now stands with whoever hit it: a timid
/// one keeps away, an unfriendly one goes for them, and one only cooled to
/// neutral stops coming over but doesn't fight back. `None` to carry on.
fn after_hit(mob: &Mob, by: &Striker, feeling: Feeling) -> Option<Doing> {
    if is_timid(mob) {
        let fleeing = matches!(mob.doing, Doing::Fleeing { from, .. } if from == by.id);
        return (!fleeing).then_some(Doing::Fleeing {
            from: by.id,
            struck: false,
        });
    }
    match (feeling, mob.doing) {
        (Feeling::Unfriendly, Doing::Going { after, gap, .. })
            if after == by.id && gap == ATTACK_GAP =>
        {
            None
        }
        (Feeling::Unfriendly, _) => Some(Doing::Going {
            after: by.id,
            last_seen: by.position,
            gap: ATTACK_GAP,
        }),
        (_, Doing::Going { after, gap, .. }) if after == by.id && gap == FRIENDLY_GAP => {
            Some(Doing::Settling {
                steps_left: SETTLING_STEPS,
            })
        }
        _ => None,
    }
}
