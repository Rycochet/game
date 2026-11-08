//! What steps on the world map cost: distance, slope, the way underfoot
//! (`Footing`: a road, water, or a terrain), and the walker's skills (`Gait`).

use super::{Footing, Gait, SLOPE_COST, step_cost};

/// What a step on the world map costs, given the elevation at each end and
/// what it lands on (`Footing`): its distance, plus the slope (less with
/// `gait`'s Climbing); then along a road divided by the road's speed, in water
/// (a river or the sea) waded (`Gait::swim`), and otherwise slowed by its
/// terrain (less with Pioneering, `Gait::cross`). Water counts as flat. Routes
/// are found by this; the time a step takes is `world_step_time`.
pub fn world_step_cost(
    from: (i64, i64),
    to: (i64, i64),
    elevations: (Option<i32>, Option<i32>),
    footing: Footing,
    gait: Gait,
) -> u32 {
    let cost = step_cost(from, to) + gait.climb(climbed(elevations) * SLOPE_COST);
    match footing {
        Footing {
            road: Some(kind), ..
        } => cost * 100 / kind.speed_percent(),
        Footing { wet: true, .. } => gait.swim(cost),
        Footing {
            terrain: Some(terrain),
            ..
        } => gait.cross(terrain, cost),
        _ => cost,
    }
}

/// How long a step on the world map takes: `world_step_cost`, quicker with
/// `gait`'s Running unless wading. Running is left out of routes, as it speeds
/// every step alike and would make the search's estimate too high.
pub fn world_step_time(
    from: (i64, i64),
    to: (i64, i64),
    elevations: (Option<i32>, Option<i32>),
    footing: Footing,
    gait: Gait,
) -> u32 {
    let cost = world_step_cost(from, to, elevations, footing, gait);
    if footing.wades() {
        cost
    } else {
        gait.run(cost)
    }
}

/// How many elevations a step climbs or descends: none to or from water.
pub fn climbed(elevations: (Option<i32>, Option<i32>)) -> u32 {
    match elevations {
        (Some(a), Some(b)) => a.abs_diff(b),
        _ => 0,
    }
}
