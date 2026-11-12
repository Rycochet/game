//! Taking the steps asked for, as energy allows.

use bevy::prelude::*;
use shared::energy::ACTION_TICKS_PER_SECOND;
use shared::movement::{step_on_world_map_by, world_step_time};
use shared::protocol::DoorAsk;

use super::doors::AskDoor;
use super::level_costs::LevelCosts;
use super::level_step::level_step;
use super::route_help::{Others, crowd};
use super::stride::Stride;
use super::{
    MOST_STEPS_WAITING, Moved, Player, PlayerEnergy, Stepped, TakeStep, Travel, action_tick,
    enemy_cells,
};
use crate::common::{
    CancelledEnergy, CurrentDungeon, CurrentWorld, GameClock, GameView, GodMode, MapPosition,
    NearRivers, PlayerGait, PlayerSize, PlayerSpeed, Sneaking, SpentEnergy, SpotPosition,
    WildCellHeights,
};
use shared::clearance::crowd_per_mille;
use shared::spot::Spot;

/// Takes the steps asked for this frame, in order: on a level or in the Hall
/// as a move to the next cell's middle (`level_step`: from a middle by the
/// cell rule, onto walkable cells the client knows of or over liquid to
/// one), shown going there; on the world map, not into open water (the
/// server's rules, `shared::movement`). Steps are ignored until a jump has
/// landed.
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
    (gait, sneaking): (Res<PlayerGait>, Res<Sneaking>),
    (heights, rivers): (Res<WildCellHeights>, Res<NearRivers>),
    mut energy: ResMut<PlayerEnergy>,
    (mut stepped, mut moved, mut asks): (
        MessageWriter<Stepped>,
        MessageWriter<Moved>,
        MessageWriter<AskDoor>,
    ),
    (god, size): (Res<GodMode>, Res<PlayerSize>),
    others: Others,
    player: Single<(&mut MapPosition, &mut SpotPosition, &mut Travel), With<Player>>,
) {
    let (mut position, mut spot, mut travel) = player.into_inner();
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
        if travel.jump.is_some() || travel.landing.is_some() {
            continue;
        }
        let from = (position.x, position.y);
        if *view.get() == GameView::Dungeon {
            let crowd = crowd(&others, *god);
            let costs = LevelCosts {
                level: &dungeon.level,
                heights: &heights,
                gait: gait.0,
                sneaking: sneaking.0,
                speed: speed.0,
                size: size.0,
                others: &crowd,
            };
            // Put somewhere else (the server, arriving): from there.
            if spot.0 != travel.last_spot {
                travel.stride = Stride::at(spot.0);
                travel.last_spot = spot.0;
                travel.spots.clear();
            }
            // Whatever was walked and not yet sent goes first.
            if let Some((from, to)) = travel.stride.finish() {
                energy
                    .energy
                    .spend(tick, speed.0, costs.move_time(from, to));
                moved.write(Moved { from, to });
            }
            let start = travel.stride.at;
            let landing = level_step(&dungeon.level, size.0, gait.0, start, step);
            let free = |to: &Spot| crowd_per_mille(size.0, start, *to, &crowd).is_some();
            let Some(to) = landing.filter(free) else {
                // Walking into a closed door tries to open it.
                let (dx, dy) = step.from(from);
                if dungeon.level.is_shut(dx, dy) && travel.door_asked != Some((dx, dy)) {
                    travel.door_asked = Some((dx, dy));
                    asks.write(AskDoor(DoorAsk::Open { x: dx, y: dy }));
                }
                continue;
            };
            energy
                .energy
                .spend(tick, speed.0, costs.move_time(start, to));
            let (x, y) = to.cell();
            *position = MapPosition { x, y };
            spot.0 = to;
            moved.write(Moved { from: start, to });
            // Shown going there, as a jump is (`travel_on_level`).
            travel.route.clear();
            travel.spots.clear();
            travel.last = *position;
            travel.last_spot = to;
            travel.stride = Stride::at(to);
            travel.landing = Some(to);
            continue;
        }
        let to = world.as_ref().and_then(|world| {
            step_on_world_map_by(&world.ground, world.conditions(&clock), from, step, gait.0)
        });
        let Some((x, y)) = to else {
            continue;
        };
        // An enemy won't move aside; anyone else swaps places, as the server says.
        if enemy_cells(&others, *god).contains(&(x, y)) {
            continue;
        }
        let cost = world.as_ref().map_or(0, |world| {
            let conditions = world.conditions(&clock);
            let ends = (
                world.ground.elevation(from.0, from.1, conditions),
                world.ground.elevation(x, y, conditions),
            );
            let road = world.roads.steps().kind_of(from, (x, y));
            let on_river = rivers.points.contains((x, y));
            let footing = world.ground.footing((x, y), conditions, road, on_river);
            world_step_time(from, (x, y), ends, footing, gait.0)
        });
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
