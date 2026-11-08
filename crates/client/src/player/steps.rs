//! Taking the steps asked for, as energy allows.

use bevy::prelude::*;
use shared::energy::{ACTION_TICKS_PER_SECOND, site_step_cost};
use shared::movement::{step_on_level, step_on_world_map, world_step_time};

use super::{
    MOST_STEPS_WAITING, Player, PlayerEnergy, Stepped, TakeStep, Travel, action_tick, enemy_cells,
};
use crate::common::{
    CancelledEnergy, CurrentDungeon, CurrentWorld, GameClock, GameView, GodMode, MapPosition,
    Other, PlayerGait, PlayerSpeed, SpentEnergy,
};

/// Takes the steps asked for this frame, in order: in a dungeon, only onto
/// walkable cells the client knows of, or over liquid to one; on the world map,
/// not into open water (the server's rules, `shared::movement`). Steps are
/// ignored until a jump has landed.
#[allow(clippy::too_many_arguments)]
pub(super) fn take_steps(
    mut requests: MessageReader<TakeStep>,
    mut spent: MessageReader<SpentEnergy>,
    mut cancelled: MessageReader<CancelledEnergy>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    world: Option<Res<CurrentWorld>>,
    clock: Res<GameClock>,
    time: Res<Time>,
    speed: Res<PlayerSpeed>,
    gait: Res<PlayerGait>,
    mut energy: ResMut<PlayerEnergy>,
    mut stepped: MessageWriter<Stepped>,
    god: Res<GodMode>,
    others: Query<(&Other, &MapPosition), Without<Player>>,
    player: Single<(&mut MapPosition, &mut Travel), With<Player>>,
) {
    let (mut position, mut travel) = player.into_inner();
    let tick = action_tick(&time);
    // The time up to now went at the old speed.
    let old_speed = energy.speed.max(1);
    if old_speed != speed.0 {
        energy.energy.settle(tick, old_speed);
        energy.speed = speed.0;
    }
    for &SpentEnergy(cost) in spent.read() {
        energy.energy.spend(tick, speed.0, cost);
        energy.stopped_for += cost as f64 / (speed.0.max(1) as f64 * ACTION_TICKS_PER_SECOND);
    }
    for &CancelledEnergy(penalty) in cancelled.read() {
        energy.energy.cancel(tick, speed.0, penalty);
        energy.stopped_for = 0.0;
    }
    for &TakeStep(step) in requests.read() {
        if energy.waiting.len() < MOST_STEPS_WAITING {
            energy.waiting.push_back(step);
        }
    }
    while energy.energy.ready(tick, speed.0) {
        let Some(step) = energy.waiting.pop_front() else {
            break;
        };
        if travel.jump.is_some() {
            continue;
        }
        let from = (position.x, position.y);
        let to = match view.get() {
            GameView::Dungeon => step_on_level(&dungeon.level, from, step),
            _ => world.as_ref().and_then(|world| {
                step_on_world_map(&world.ground, world.conditions(&clock), from, step)
            }),
        };
        let Some((x, y)) = to else {
            continue;
        };
        // An enemy won't move aside; anyone else swaps places, as the server says.
        if enemy_cells(&others, *god).contains(&(x, y)) {
            continue;
        }
        let cost = match view.get() {
            GameView::Dungeon => site_step_cost(from, (x, y)),
            _ => world.as_ref().map_or(0, |world| {
                let conditions = world.conditions(&clock);
                let ends = (
                    world.ground.elevation(from.0, from.1, conditions),
                    world.ground.elevation(x, y, conditions),
                );
                world_step_time(
                    from,
                    (x, y),
                    ends,
                    world.roads.steps().kind_of(from, (x, y)),
                    gait.0,
                )
            }),
        };
        energy.energy.spend(tick, speed.0, cost);
        *position = MapPosition { x, y };
        stepped.write(Stepped::between(step, from, (x, y)));
        if (x, y) != step.from(from) {
            // A jump replaces any journey, as a step does, and is shown in
            // `travel` instead of jumping there at once.
            travel.route.clear();
            travel.last = *position;
            travel.jump = Some(*position);
        }
    }
}
