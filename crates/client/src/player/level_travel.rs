//! Moving on a level or in the Hall: walking freely the way the keys say
//! (`Walking`), journeys to clicked cells along routes at any angle, and
//! showing moves already taken (jumps, steps asked for). Every straight move
//! is announced (`Moved`) for the server to check.

use bevy::math::DVec2;
use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::energy::{ACTION_TICKS_PER_SECOND, NORMAL_SPEED};
use shared::spot::{CELL_SPAN, Spot};

use super::doors::AskDoor;
use super::level_costs::LevelCosts;
use super::level_moves::{door_ahead, follow_route, jump_ahead};
use super::level_route::level_route;
use super::route_help::enemy_cells;
use super::stride::{Stride, Walker};
use super::{
    Arrived, DUNGEON_TRAVEL_SPEED, Moved, Player, PlayerEnergy, Travel, TravelTo, Walking,
    action_tick,
};
use crate::common::{
    CurrentDungeon, GameView, GodMode, MapPosition, Other, PlayerGait, PlayerSpeed, SmoothPosition,
    Sneaking, SpotPosition, WildCellHeights,
};

/// How far ahead of the energy it has the player may walk, in seconds: moves
/// go no quicker than the server takes them, so its queue never fills.
const AHEAD: f64 = 0.25;

/// Moves the player on a level or in the Hall. Any other change to
/// `SpotPosition` (the server, arriving) ends what was under way and puts
/// them there.
#[allow(clippy::too_many_arguments)]
pub(super) fn travel_on_level(
    time: Res<Time>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    (mut requests, mut walking): (MessageReader<TravelTo>, MessageReader<Walking>),
    mut arrived: MessageWriter<Arrived>,
    (mut moved, mut asks): (MessageWriter<Moved>, MessageWriter<AskDoor>),
    (speed_now, gait, sneaking): (Res<PlayerSpeed>, Res<PlayerGait>, Res<Sneaking>),
    heights: Res<WildCellHeights>,
    mut energy: ResMut<PlayerEnergy>,
    god: Res<GodMode>,
    others: Query<(&Other, &MapPosition), Without<Player>>,
    player: Single<
        (
            &mut MapPosition,
            &mut SpotPosition,
            &mut SmoothPosition,
            &mut Travel,
        ),
        With<Player>,
    >,
) {
    let (mut cell, mut spot, mut smooth, mut travel) = player.into_inner();
    // The world map is `travel`'s.
    if *view.get() != GameView::Dungeon {
        requests.clear();
        walking.clear();
        return;
    }
    if spot.0 != travel.last_spot {
        travel.spots.clear();
        travel.landing = None;
        travel.stride = Stride::at(spot.0);
        travel.last_spot = spot.0;
        *smooth = SmoothPosition::of_units((spot.0.x, spot.0.y));
    }
    let tick = action_tick(&time);
    let stopped = energy.stopped_for.min(time.delta_secs_f64());
    energy.stopped_for -= stopped;
    let seconds = time.delta_secs_f64() - stopped;
    let units_a_second = DUNGEON_TRAVEL_SPEED * CELL_SPAN as f64;
    let mut distance = units_a_second * seconds * f64::from(speed_now.0) / NORMAL_SPEED as f64;
    let walk = walking.read().last().copied();
    if let Some(request) = requests.read().last() {
        travel.goal = (request.x, request.y);
        travel.asked = true;
        travel.rerouted = false;
    }

    // A move already taken, being shown: nothing else until it lands.
    if let Some(landing) = travel.landing {
        let now = DVec2::new(smooth.x, smooth.y) * CELL_SPAN as f64;
        let to = DVec2::new(landing.x as f64, landing.y as f64);
        let shown = if now.distance(to) <= distance {
            travel.landing = None;
            to
        } else {
            now + (to - now).normalize() * distance
        } / CELL_SPAN as f64;
        *smooth = SmoothPosition {
            x: shown.x,
            y: shown.y,
        };
        if travel.landing.is_some() {
            return;
        }
        distance = 0.0;
    }

    let enemies = enemy_cells(&others, *god);
    let level = &dungeon.level;
    let walker = Walker {
        level,
        size: shared::creature::Size::Medium,
        free: |cell: (i64, i64)| !enemies.contains(&cell),
        squeezes: walk.is_none(),
    };
    let costs = LevelCosts {
        level,
        heights: &heights,
        gait: gait.0,
        sneaking: sneaking.0,
        speed: speed_now.0,
    };
    // Not further ahead of the energy than the server will let it be.
    let ahead = (AHEAD * ACTION_TICKS_PER_SECOND) as u64;
    if energy.energy.ready_at(tick, speed_now.0) > tick + ahead {
        distance = 0.0;
    }
    let mut moves = Vec::new();
    let at = travel.stride.at;

    if let Some(Walking { x, y }) = walk {
        travel.spots.clear();
        travel.asked = false;
        let want = DVec2::new(x, y);
        let ahead = Spot::new(
            at.x + (want.x * CELL_SPAN as f64).round() as i64,
            at.y + (want.y * CELL_SPAN as f64).round() as i64,
        );
        let distance = distance / costs.slowness(at.cell(), ahead.cell());
        let walked = travel
            .stride
            .walk(&walker, want, distance, None, &mut moves);
        if walked.stuck && walked.distance < 1.0 {
            if let Some(door) = door_ahead(level, at, want) {
                super::doors::wait_at_door(&mut travel, door, &mut asks);
            } else if distance > 0.0
                && let Some(landing) = jump_ahead(&walker, at, want)
            {
                moves.extend(travel.stride.finish());
                moves.push((travel.stride.at, landing));
                travel.stride = Stride::at(landing);
                travel.landing = Some(landing);
            }
        }
    } else if travel.asked {
        travel.asked = false;
        let goal = travel.goal;
        let remembered = &dungeon.remembered;
        let known = |x, y| {
            remembered.is_seen(x, y)
                && !enemies.contains(&(x, y))
                && ((x, y) == goal || level.cell(x, y) != Cell::WorldGate)
        };
        moves.extend(travel.stride.finish());
        let path = level_route(level, travel.stride.at, goal, known);
        travel.route_arrives = path.reached;
        travel.spots = path.spots.into_iter().skip(1).collect();
        if travel.spots.is_empty() && path.reached {
            arrived.write(Arrived {
                x: goal.0,
                y: goal.1,
            });
        }
    } else if travel.spots.is_empty() {
        // Let go of the keys: the last stretch walked is sent.
        moves.extend(travel.stride.finish());
    }
    if walk.is_none() && !travel.spots.is_empty() {
        let costs = |a, b| costs.slowness(a, b);
        if follow_route(&mut travel, &walker, distance, costs, &mut moves, &mut asks) {
            arrived.write(Arrived {
                x: travel.goal.0,
                y: travel.goal.1,
            });
        }
    }

    for &(from, to) in &moves {
        energy
            .energy
            .spend(tick, speed_now.0, costs.move_time(from, to));
        moved.write(Moved { from, to });
    }
    let now = travel.landing.unwrap_or(travel.stride.at);
    if spot.0 != now {
        spot.0 = now;
        let (x, y) = now.cell();
        cell.set_if_neq(MapPosition { x, y });
    }
    travel.last_spot = now;
    travel.last = *cell;
    if travel.landing.is_none() {
        let shown = SmoothPosition::of_units((now.x, now.y));
        smooth.set_if_neq(shown);
    }
}
