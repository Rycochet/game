//! Running every mob each tick.

use shared::dungeon::line_is_clear;

use super::fight::{MobBlow, mob_turn};
use super::territory::mind_territory;
use super::tracking::track;
use super::{Doing, Maps, Population, Spot, Taken, act, cross, everyone_playing, notice};
use crate::ServerWorld;
use crate::connections::Client;

/// How often old scent marks are thrown away, in ticks: every 10 seconds.
const SWEEP_EVERY: u64 = 200;

/// Lets each level's mobs do what they are doing, as far as they have the energy.
pub(crate) fn run_mobs(clients: &mut [Client], world: &mut ServerWorld) {
    let tick = world.action_clock.ticks();
    let everyone = everyone_playing(clients);
    let mut crossings = Vec::new();
    let mut blows: Vec<MobBlow> = Vec::new();
    for (&place, population) in &mut world.populations.0 {
        let here = || {
            everyone
                .iter()
                .filter(|someone| someone.level == Some(place))
        };
        // Those in god mode still stand somewhere and can see, but aren't noticed.
        let standing: Vec<(i64, i64)> = here().map(|someone| someone.position).collect();
        let spots: Vec<Spot> = here()
            .filter(|someone| !someone.god_mode)
            .map(|someone| Spot {
                id: someone.id,
                character: someone.character,
                kind: someone.look.0,
                position: someone.position,
                scent: someone.scent,
            })
            .collect();
        let Population {
            mobs,
            level,
            due,
            scents,
            last_at,
            ..
        } = population;

        // Players leave scent on the cells they step off.
        for spot in &spots {
            if let Some((was_at, _)) = last_at.insert(spot.id, (spot.position, spot.scent)) {
                scents.walked(spot.id, was_at, spot.position, spot.scent, tick);
            }
        }
        last_at.retain(|id, _| spots.iter().any(|spot| spot.id == *id));
        if tick.is_multiple_of(SWEEP_EVERY) {
            scents.sweep(tick);
        }

        // Anyone at rest, settling or guarding may notice someone.
        for mob in mobs.iter_mut() {
            mind_territory(mob, level, &spots, tick);
            if let Some(doing) = notice(mob, level, &spots) {
                let at_rest = mob.doing == Doing::Idle;
                mob.doing = doing;
                mob.target = doing.target();
                if at_rest {
                    due.insert((mob.energy.ready_at(tick, mob.speed), mob.id));
                }
            }
        }

        let mut taken: Taken = standing.iter().copied().collect();
        taken.extend(mobs.iter().map(|mob| mob.position));
        let mut maps = Maps::default();
        while let Some(&(when, id)) = due.first() {
            if when > tick {
                break;
            }
            due.remove(&(when, id));
            let Some(mob) = mobs.iter_mut().find(|mob| mob.id == id) else {
                continue;
            };
            let mut crossing = None;
            let seen = |cell| {
                standing
                    .iter()
                    .any(|&spot| line_is_clear(level, spot, cell))
            };
            let was_at = mob.position;
            let fighting = mob_turn(mob, level, &spots, tick, &mut blows);
            let cost = if let Some(cost) = fighting {
                cost
            } else if matches!(mob.doing, Doing::Tracking { .. }) {
                track(mob, level, &spots, scents, &taken, tick)
            } else {
                act(
                    mob,
                    level,
                    &spots,
                    &mut maps,
                    tick,
                    seen,
                    &mut crossing,
                    &taken,
                    place,
                )
            };
            // Mobs leave scent too.
            scents.walked(mob.id, was_at, mob.position, mob.scent, tick);
            mob.pacing.turn_ended(was_at, mob.position);
            taken.remove(&was_at);
            taken.insert(mob.position);
            mob.energy.spend(tick, mob.speed, cost);
            mob.target = mob.doing.target();
            if mob.doing.gave_up() {
                mob.harmed_by.clear();
            }
            if let Some(crossing) = crossing {
                crossings.push(crossing);
            } else if mob.doing != Doing::Idle {
                // At least the next tick, so one turn can't run for ever here.
                due.insert((mob.next_due(tick), id));
            }
        }
    }
    for crossing in crossings {
        cross(&mut world.populations, crossing, tick, &everyone);
    }
    crate::connections::take_mob_blows(clients, world, blows, tick);
}

/// How far apart on the world map, the nearer way round the planet.
pub(super) fn world_gap(a: (i64, i64), b: (i64, i64), width: i64) -> i64 {
    let dx = (a.0 - b.0).rem_euclid(width);
    dx.min(width - dx).max((a.1 - b.1).abs())
}
