//! A mob's steps: taking them, wandering, acting, and finding a free cell.

use shared::dungeon::{Level, line_is_clear};
use shared::energy::site_step_cost;
use shared::movement::{Step, mob_can_stand, mob_step_on_level};
use shared::rng::Seed;
use std::collections::BTreeSet;

use super::doubling_back::walk_towards;
use super::territory::{LEASH, guard_cell, is_territorial};
use super::timid::too_near;
use super::tracking::start_tracking;
use super::{
    Crossing, Doing, HOME_WITHIN, Maps, Mob, SETTLING_STEPS, Spot, THINKING_COST, cells_apart,
};
use crate::storage::SiteId;

/// The cells someone is standing on: no mob can go on them.
pub(super) type Taken = BTreeSet<(i64, i64)>;

/// Moves a mob by a step, if it can go. What it cost.
pub(super) fn take_step(
    mob: &mut Mob,
    level: &Level,
    step: Option<Step>,
    taken: &Taken,
) -> Option<u32> {
    let to = mob_step_on_level(level, mob.position, step?)?;
    if taken.contains(&to) {
        return None;
    }
    let cost = site_step_cost(mob.position, to);
    mob.position = to;
    Some(cost)
}

/// A direction for wandering, different for each mob and moment.
pub(super) fn wandering_step(mob: &Mob, tick: u64) -> Option<Step> {
    let (high, low) = mob.id.0.as_u64_pair();
    let roll = Seed::new(high ^ low).at(&[tick as i64, mob.position.0, mob.position.1]);
    Step::new((roll % 3) as i64 - 1, (roll / 3 % 3) as i64 - 1)
}

/// One mob's turn: what it does and what it costs. Afterwards it may be at rest.
/// `seen` says whether any player can see a cell. `crossing` is filled in if it
/// is going to another level.
#[allow(clippy::too_many_arguments)]
pub(super) fn act(
    mob: &mut Mob,
    level: &Level,
    spots: &[Spot],
    maps: &mut Maps,
    tick: u64,
    seen: impl Fn((i64, i64)) -> bool,
    crossing: &mut Option<Crossing>,
    taken: &Taken,
    (site, level_index): (SiteId, u32),
) -> u32 {
    let free = |cell: (i64, i64)| !taken.contains(&cell);
    let settle = Doing::Settling {
        steps_left: SETTLING_STEPS,
    };
    match mob.doing {
        Doing::Idle => 0,
        // `run_mobs` sends these to `tracking::track`, which knows the scents.
        Doing::Tracking { .. } => THINKING_COST,
        Doing::Going {
            after,
            last_seen,
            gap,
        } => {
            let target = spots
                .iter()
                .find(|spot| spot.id == after && line_is_clear(level, mob.position, spot.position));
            let goal = target.map_or(last_seen, |spot| spot.position);
            mob.doing = Doing::Going {
                after,
                last_seen: goal,
                gap,
            };
            if is_territorial(mob) && cells_apart(mob.home, goal) > LEASH {
                // Too far to bother chasing: back to keeping them out.
                mob.doing = match target {
                    Some(_) => Doing::Guarding {
                        against: after,
                        last_seen: goal,
                    },
                    None => settle,
                };
                return THINKING_COST;
            }
            let near = if target.is_some() { gap } else { 0 };
            if cells_apart(mob.position, goal) <= near {
                // Lost them, here where they were last seen: after their scent,
                // if it has a nose.
                if target.is_none() {
                    mob.doing = start_tracking(mob, after, gap).unwrap_or(settle);
                }
                return THINKING_COST;
            }
            walk_towards(mob, level, maps, goal, taken).unwrap_or_else(|| {
                // No way there.
                mob.doing = settle;
                THINKING_COST
            })
        }
        Doing::Following {
            stairs,
            to_level,
            arrives,
            ..
        } => {
            if mob.position == stairs {
                *crossing = Some(Crossing {
                    mob: mob.id,
                    site,
                    from_level: level_index,
                    to_level,
                    from_stairs: stairs,
                    arrives,
                    going_back: false,
                });
                return shared::energy::CHANGE_MAP_COST;
            }
            walk_towards(mob, level, maps, stairs, taken).unwrap_or_else(|| {
                mob.doing = settle;
                THINKING_COST
            })
        }
        Doing::Fleeing { from, .. } => {
            let threat = spots
                .iter()
                .find(|spot| spot.id == from && line_is_clear(level, mob.position, spot.position));
            // Out of sight, or far enough: no further.
            let Some(threat) = threat.filter(|threat| too_near(mob, threat)) else {
                mob.doing = settle;
                return THINKING_COST;
            };
            // Cornered or not, it may strike again after this.
            mob.doing = Doing::Fleeing {
                from,
                struck: false,
            };
            let step = maps
                .to(level, threat.position)
                .away(level, mob.position, free);
            take_step(mob, level, step, taken).unwrap_or(THINKING_COST)
        }
        Doing::Guarding { against, .. } => {
            let sees_within = mob.kind.senses().sight_range();
            let target = spots.iter().find(|spot| {
                spot.id == against
                    && cells_apart(mob.position, spot.position) <= sees_within
                    && line_is_clear(level, mob.position, spot.position)
            });
            let Some(target) = target else {
                mob.doing = settle;
                return THINKING_COST;
            };
            mob.doing = Doing::Guarding {
                against,
                last_seen: target.position,
            };
            let spot = guard_cell(level, mob, target.position, taken);
            mob.pacing.aim_at(spot);
            if mob.position == spot {
                // Standing its ground.
                return THINKING_COST;
            }
            walk_towards(mob, level, maps, spot, taken).unwrap_or(THINKING_COST)
        }
        Doing::Settling { steps_left } => {
            mob.doing = match steps_left.checked_sub(1) {
                Some(steps_left) => Doing::Settling { steps_left },
                None => Doing::GoingHome { since: tick },
            };
            take_step(mob, level, wandering_step(mob, tick), taken).unwrap_or(THINKING_COST)
        }
        Doing::GoingHome { since } => {
            // Home, or the stairs leading back towards it.
            let hop = mob.way_back.last().copied();
            let goal = hop.map_or(mob.home, |hop| hop.stairs);
            let late = tick.saturating_sub(since) > HOME_WITHIN;
            let walked = walk_towards(mob, level, maps, goal, taken);
            // Stuck or lost, it gets there when nobody can see.
            if (late || walked.is_none()) && !seen(mob.position) && !seen(goal) && free(goal) {
                mob.position = goal;
            }
            if mob.position == goal {
                match hop {
                    Some(hop) => {
                        *crossing = Some(Crossing {
                            mob: mob.id,
                            site,
                            from_level: level_index,
                            to_level: hop.to_level,
                            from_stairs: hop.stairs,
                            arrives: hop.arrives,
                            going_back: true,
                        });
                    }
                    None => mob.doing = Doing::Idle,
                }
            }
            walked.unwrap_or(THINKING_COST)
        }
    }
}

/// The nearest walkable cell to this one that isn't taken, not far off.
pub(super) fn nearest_free(
    level: &Level,
    cell: (i64, i64),
    taken: impl Fn((i64, i64)) -> bool,
) -> Option<(i64, i64)> {
    const FURTHEST: i64 = 8;
    (1..=FURTHEST).find_map(|reach| {
        (-reach..=reach).find_map(|dy| {
            (-reach..=reach).find_map(|dx| {
                let spot = (cell.0 + dx, cell.1 + dy);
                let on_edge = dx.abs().max(dy.abs()) == reach;
                (on_edge && mob_can_stand(level, spot.0, spot.1) && !taken(spot)).then_some(spot)
            })
        })
    })
}
