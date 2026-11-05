//! Following someone by their scent, after losing sight of them: a mob reads their
//! mark where it stands and goes the way they went, or picks up the freshest one
//! beside it. Each mark it follows is newer than the last, so it never goes
//! round in circles.

use shared::creature::Acuity;
use shared::dungeon::{Level, line_is_clear};
use shared::movement::Step;

use super::scent::{Mark, Scents, Went};
use super::territory::{LEASH, is_territorial};
use super::{Doing, Mob, SETTLING_STEPS, Spot, THINKING_COST, Taken, cells_apart, take_step};

/// A mob that has lost someone starts tracking them, if it can smell at all.
pub(super) fn start_tracking(mob: &Mob, after: shared::id::Id, gap: i64) -> Option<Doing> {
    (mob.kind.senses().smell != Acuity::None).then_some(Doing::Tracking {
        after,
        gap,
        since: 0,
    })
}

/// One turn of tracking: what it cost.
pub(super) fn track(
    mob: &mut Mob,
    level: &Level,
    spots: &[Spot],
    scents: &Scents,
    taken: &Taken,
    now: u64,
) -> u32 {
    let Doing::Tracking { after, gap, since } = mob.doing else {
        return 0;
    };
    let settle = Doing::Settling {
        steps_left: SETTLING_STEPS,
    };
    // Seen again: after them as before.
    let sees_within = mob.kind.senses().sight_range();
    if let Some(target) = spots.iter().find(|spot| {
        spot.id == after
            && cells_apart(mob.position, spot.position) <= sees_within
            && line_is_clear(level, mob.position, spot.position)
    }) {
        mob.doing = Doing::Going {
            after,
            last_seen: target.position,
            gap,
        };
        return THINKING_COST;
    }
    if is_territorial(mob) && cells_apart(mob.home, mob.position) > LEASH {
        mob.doing = settle;
        return THINKING_COST;
    }
    let smell = mob.kind.senses().smell;
    let fresh = |cell| {
        scents
            .smelt(cell, after, smell, now)
            .filter(|mark: &Mark| mark.laid >= since)
    };
    if let Some(mark) = fresh(mob.position) {
        mob.doing = Doing::Tracking {
            after,
            gap,
            since: mark.laid,
        };
        return match mark.went {
            Went::Towards(dx, dy) => {
                let step = Step::new(dx.signum() as i64, dy.signum() as i64);
                take_step(mob, level, step, taken).unwrap_or(THINKING_COST)
            }
            Went::Stairs { to_level, arrives } if mob.kind.def().uses_stairs => {
                mob.doing = Doing::Following {
                    after,
                    stairs: mob.position,
                    to_level,
                    arrives,
                    gap,
                };
                THINKING_COST
            }
            Went::Stairs { .. } => {
                mob.doing = settle;
                THINKING_COST
            }
        };
    }
    // Nothing here: the freshest mark beside it newer than the last it followed
    // (not the one it came from), if any.
    let beside = (-1..=1)
        .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
        .filter(|&(dx, dy)| (dx, dy) != (0, 0))
        .filter_map(|(dx, dy)| {
            let cell = (mob.position.0 + dx, mob.position.1 + dy);
            fresh(cell)
                .filter(|mark| mark.laid > since)
                .map(|mark| (mark.laid, dx, dy))
        })
        .max();
    match beside {
        Some((_, dx, dy)) => {
            take_step(mob, level, Step::new(dx, dy), taken).unwrap_or(THINKING_COST)
        }
        None => {
            mob.doing = settle;
            THINKING_COST
        }
    }
}
