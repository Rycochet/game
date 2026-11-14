//! Travelling along a found route to a clicked point on the world map (on
//! levels and in the Hall: `level_travel`).

use bevy::math::DVec2;
use bevy::prelude::*;
use shared::energy::NORMAL_SPEED;
use shared::movement::{Step, WorldGround, step_cost, step_on_world_map_by, world_step_time};
use shared::pathfinding::find_world_path;

use super::route_help::{nearest_cell, towards};
use super::{Arrived, Player, PlayerEnergy, Stepped, TRAVEL_SPEED, Travel, TravelTo, action_tick};
use crate::common::{
    CurrentWorld, EscortGait, GameClock, GameView, MapPosition, NearRivers, PlayerGait,
    PlayerSpeed, SmoothPosition,
};

/// Moves along the route found when the journey was asked for, always ending
/// exactly on its last point, announcing each step: at constant speed, except
/// that slopes are slower, by what they cost, and roads and Running quicker.
/// Any other change to `MapPosition` (a step, the server) cancels travel and
/// jumps there. A step is taken on leaving a point. World map only.
#[allow(clippy::too_many_arguments)]
pub(super) fn travel(
    time: Res<Time>,
    view: Res<State<GameView>>,
    world: Option<Res<CurrentWorld>>,
    clock: Res<GameClock>,
    mut requests: MessageReader<TravelTo>,
    mut arrived: MessageWriter<Arrived>,
    mut stepped: MessageWriter<Stepped>,
    speed_now: Res<PlayerSpeed>,
    (gait, escort): (Res<PlayerGait>, Res<EscortGait>),
    rivers: Res<NearRivers>,
    mut energy: ResMut<PlayerEnergy>,
    player: Single<(&mut MapPosition, &mut SmoothPosition, &mut Travel), With<Player>>,
) {
    let (mut tile, mut smooth, mut travel) = player.into_inner();
    // With someone escorted, at the slower's pace.
    let world_gait = gait.on_world_map(&escort);
    // Levels and the Hall are `travel_on_level`'s.
    let Some(world) = world.filter(|_| *view.get() == GameView::WorldMap) else {
        requests.clear();
        return;
    };
    let rivers = &rivers.points;
    // What a world map step stands on: a road, water or its terrain.
    let footing = |ground: &WorldGround, conditions, road, to: (i64, i64)| {
        ground.footing(to, conditions, road, rivers.contains(to))
    };
    let conditions = world.conditions(&clock);
    let (ground, roads) = (&world.ground, world.roads.steps());

    if *tile != travel.last {
        travel.route.clear();
        travel.jump = None;
        travel.last = *tile;
        *smooth = SmoothPosition {
            x: tile.x as f64,
            y: tile.y as f64,
        };
    }

    if let Some(request) = requests.read().last() {
        let from = (tile.x, tile.y);
        let to = (request.x, request.y);
        let path = find_world_path(ground, conditions, roads, rivers, from, to, world_gait);
        travel.goal = to;
        travel.route_arrives = path.reached;
        travel.route = path.cells.into();
        if travel.route.is_empty() {
            // Nowhere to go: back to the middle of this cell.
            travel.route.push_back(from);
        }
    }

    let from = DVec2::new(smooth.x, smooth.y);
    let speed = TRAVEL_SPEED;
    let tick = action_tick(&time);
    let stopped = energy.stopped_for.min(time.delta_secs_f64());
    energy.stopped_for -= stopped;
    let moving = time.delta_secs_f64() - stopped;
    let distance = speed * moving * speed_now.0 as f64 / NORMAL_SPEED as f64;

    if let Some(landing) = travel.jump {
        let to = DVec2::new(landing.x as f64, landing.y as f64);
        let now = towards(from, to, distance);
        *smooth = SmoothPosition { x: now.x, y: now.y };
        let over_its_edge = nearest_cell(now) == landing;
        if now == to || (!travel.route.is_empty() && over_its_edge) {
            travel.jump = None;
        }
        return;
    }

    if !travel.route.is_empty() {
        let mut now = from;
        let mut left = distance;
        let mut at = (tile.x, tile.y);
        // Normally part of one step a frame, but a slow frame may cover more.
        while let Some(&next) = travel.route.front() {
            let to = DVec2::new(next.0 as f64, next.1 as f64);
            // How much longer this step takes than on flat ground (less than 1
            // when running makes it quicker).
            let slowness = if next != at {
                let ends = (
                    ground.elevation(at.0, at.1, conditions),
                    ground.elevation(next.0, next.1, conditions),
                );
                let road = roads.kind_of(at, next);
                let on = footing(ground, conditions, road, next);
                world_step_time(at, next, ends, on, world_gait) as f64 / step_cost(at, next) as f64
            } else {
                1.0
            };
            let moved = towards(now, to, left / slowness);
            left = (left - now.distance(moved) * slowness).max(0.0);
            now = moved;
            let out_of_cell = (now - DVec2::new(at.0 as f64, at.1 as f64))
                .abs()
                .max_element()
                >= 0.5;
            if next != at && out_of_cell {
                let step = Step::new((next.0 - at.0).signum(), (next.1 - at.1).signum())
                    .expect("the next cell isn't this one");
                // The route was legal when it was found, but cells can change.
                let legal = step_on_world_map_by(ground, conditions, at, step, world_gait);
                if legal != Some(next) {
                    // The tide or a thaw has cut the way: find another from here.
                    let goal = travel.goal;
                    let path =
                        find_world_path(ground, conditions, roads, rivers, at, goal, world_gait);
                    travel.route_arrives = path.reached;
                    travel.route = path.cells.into();
                    if travel.route.is_empty() {
                        travel.route.push_back(at);
                    }
                    continue;
                }
                // Paced by distance above, which comes to the same as this.
                let ends = (
                    ground.elevation(at.0, at.1, conditions),
                    ground.elevation(next.0, next.1, conditions),
                );
                let on = footing(ground, conditions, roads.kind_of(at, next), next);
                let cost = world_step_time(at, next, ends, on, world_gait);
                energy.energy.spend(tick, speed_now.0, cost);
                stepped.write(Stepped::between(step, at, next));
                let jumped = next != step.from(at);
                at = next;
                if jumped {
                    travel.jump = Some(MapPosition { x: at.0, y: at.1 });
                    break;
                }
            }
            if now != to {
                break;
            }
            travel.route.pop_front();
            if travel.route.is_empty() && travel.route_arrives {
                arrived.write(Arrived { x: at.0, y: at.1 });
            }
        }
        *smooth = SmoothPosition { x: now.x, y: now.y };
        let reached = MapPosition { x: at.0, y: at.1 };
        tile.set_if_neq(reached);
        travel.last = reached;
    }
}
