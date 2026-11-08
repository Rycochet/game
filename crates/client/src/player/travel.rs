//! Travelling along a found route to a clicked point.

use bevy::math::DVec2;
use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::energy::{NORMAL_SPEED, site_step_cost};
use shared::movement::{Step, step_cost, step_on_level, step_on_world_map, world_step_time};
use shared::pathfinding::{Path, find_path, find_world_path, straighten};
use shared::sheet::Feeling;

use super::{
    Arrived, DUNGEON_TRAVEL_SPEED, Player, PlayerEnergy, Stepped, TRAVEL_SPEED, Travel, TravelTo,
    action_tick,
};
use crate::common::{
    CurrentDungeon, CurrentWorld, GameClock, GameView, GodMode, MapPosition, Other, PlayerGait,
    PlayerSpeed, SmoothPosition,
};

/// Moves along the route found when the journey was asked for, always ending
/// exactly on its last point, announcing each step: at constant speed, except
/// that world map slopes are slower, by what they cost, and Running quicker. Any other change to
/// `MapPosition` (a step, the server) cancels travel and jumps there. A step
/// is taken on leaving a cell, and a jump is shown from here too: straight to
/// the cell it lands on, whatever is asked for meanwhile. A journey carries on
/// from that cell's edge; otherwise the jump ends on its centre.
#[allow(clippy::too_many_arguments)]
pub(super) fn travel(
    time: Res<Time>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    world: Option<Res<CurrentWorld>>,
    clock: Res<GameClock>,
    mut requests: MessageReader<TravelTo>,
    mut arrived: MessageWriter<Arrived>,
    mut stepped: MessageWriter<Stepped>,
    speed_now: Res<PlayerSpeed>,
    gait: Res<PlayerGait>,
    mut energy: ResMut<PlayerEnergy>,
    god: Res<GodMode>,
    others: Query<(&Other, &MapPosition), Without<Player>>,
    player: Single<(&mut MapPosition, &mut SmoothPosition, &mut Travel), With<Player>>,
) {
    let (mut tile, mut smooth, mut travel) = player.into_inner();
    let enemies = enemy_cells(&others, *god);
    // Only through cells the player has seen, so that a route gives nothing away
    // (and memory may be out of date: the server checks), round enemies, and
    // never through a World Gate on the way somewhere else.
    let (remembered, level, enemies) = (&dungeon.remembered, &dungeon.level, &enemies);
    let route_to = |goal: (i64, i64)| {
        move |x, y| {
            remembered.is_seen(x, y)
                && !enemies.contains(&(x, y))
                && ((x, y) == goal || level.cell(x, y) != Cell::WorldGate)
        }
    };

    // On the world map, the ground and the conditions now; in a dungeon, `None`.
    let ground = (*view.get() == GameView::WorldMap)
        .then_some(world.as_ref())
        .flatten()
        .map(|world| {
            let conditions = world.conditions(&clock);
            (&world.ground, conditions, world.roads.steps())
        });

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
        let path = match ground {
            Some((ground, conditions, roads)) => {
                find_world_path(ground, conditions, roads, from, to, gait.0)
            }
            None => {
                let known = route_to(to);
                let path = find_path(&dungeon.level, from, to, None, known);
                straighten(&dungeon.level, from, &path, known)
            }
        };
        travel.goal = to;
        travel.route_arrives = path.reached;
        travel.route = path.cells.into();
        if travel.route.is_empty() {
            // Nowhere to go: back to the middle of this cell.
            travel.route.push_back(from);
        }
    }

    let from = DVec2::new(smooth.x, smooth.y);
    let speed = match view.get() {
        GameView::WorldMap => TRAVEL_SPEED,
        GameView::Dungeon => DUNGEON_TRAVEL_SPEED,
        // Travel doesn't run while connecting.
        GameView::Connecting => return,
    };
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
            let slowness = match ground {
                Some((ground, conditions, roads)) if next != at => {
                    let ends = (
                        ground.elevation(at.0, at.1, conditions),
                        ground.elevation(next.0, next.1, conditions),
                    );
                    let road = roads.kind_of(at, next);
                    world_step_time(at, next, ends, road, gait.0) as f64
                        / step_cost(at, next) as f64
                }
                _ => 1.0,
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
                let legal = match ground {
                    Some((ground, conditions, _)) => {
                        step_on_world_map(ground, conditions, at, step)
                    }
                    None => step_on_level(&dungeon.level, at, step),
                };
                let in_the_way = enemies.contains(&next);
                if in_the_way && ground.is_none() {
                    // An enemy has come between: go round it, from here.
                    let known = route_to(travel.goal);
                    let path = find_path(&dungeon.level, at, travel.goal, None, known);
                    let path = Path {
                        cells: straighten(&dungeon.level, at, &path, known).cells,
                        ..path
                    };
                    travel.route_arrives = path.reached;
                    travel.route = path.cells.into();
                    now = DVec2::new(at.0 as f64, at.1 as f64);
                    if travel.route.is_empty() {
                        break;
                    }
                    continue;
                }
                if legal != Some(next) {
                    // The tide or a thaw has cut the way: find another from here.
                    if let Some((ground, conditions, roads)) = ground {
                        let path =
                            find_world_path(ground, conditions, roads, at, travel.goal, gait.0);
                        travel.route_arrives = path.reached;
                        travel.route = path.cells.into();
                        if travel.route.is_empty() {
                            travel.route.push_back(at);
                        }
                        continue;
                    }
                    travel.route.clear();
                    now = DVec2::new(at.0 as f64, at.1 as f64);
                    break;
                }
                // Paced by distance above, which comes to the same as this.
                let cost = match ground {
                    Some((ground, conditions, roads)) => {
                        let ends = (
                            ground.elevation(at.0, at.1, conditions),
                            ground.elevation(next.0, next.1, conditions),
                        );
                        world_step_time(at, next, ends, roads.kind_of(at, next), gait.0)
                    }
                    None => site_step_cost(at, next),
                };
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

/// `distance` along the straight line from one point to another, stopping
/// exactly on it.
pub(super) fn towards(from: DVec2, to: DVec2, distance: f64) -> DVec2 {
    if from.distance(to) <= distance {
        to
    } else {
        from + (to - from).normalize() * distance
    }
}

/// The cells of mobs that won't move aside for the player: none in god mode.
pub(super) fn enemy_cells(
    others: &Query<(&Other, &MapPosition), Without<Player>>,
    god: GodMode,
) -> Vec<(i64, i64)> {
    others
        .iter()
        .filter(|(other, _)| !god.0 && other.feeling == Feeling::Unfriendly)
        .map(|(_, at)| (at.x, at.y))
        .collect()
}

/// The grid point a position is nearest: its cell, in a dungeon.
pub(super) fn nearest_cell(position: DVec2) -> MapPosition {
    MapPosition {
        x: position.x.round() as i64,
        y: position.y.round() as i64,
    }
}
