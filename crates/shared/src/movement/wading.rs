//! Wading: steps into shallow water (`Cell::Shallows`), or swimming into deep
//! water (`Cell::Water`, for a good swimmer: `step_on_level_by`), on any level
//! take longer, less so with Swimming (`Gait::swim`); jumps take less with
//! Jumping (`Gait::jump`).

use super::{Footing, Gait, Terrain, step_cost, wild_step_cost};
use crate::dungeon::{Cell, Level};

/// Whether a step landing on `to` wades or swims.
pub fn wades(level: &Level, to: (i64, i64)) -> bool {
    matches!(level.cell(to.0, to.1), Cell::Shallows | Cell::Water)
}

/// What a step landing on `to` stands on: water if it wades, and `terrain`
/// (the wilderness's, `WildHeights::terrain`) on open ground or grass only, not
/// roads, floors or anything else made.
pub fn footing_on(level: &Level, to: (i64, i64), terrain: Option<Terrain>) -> Footing {
    footing_of(level.cell(to.0, to.1), terrain)
}

/// `footing_on` for the cell a step lands on.
pub fn footing_of(cell: Cell, terrain: Option<Terrain>) -> Footing {
    Footing {
        road: None,
        wet: matches!(cell, Cell::Shallows | Cell::Water),
        terrain: terrain.filter(|_| matches!(cell, Cell::Ground | Cell::Grass)),
    }
}

/// Whether a step from `from` lands on `to` by jumping (two cells away).
pub fn is_jump(from: (i64, i64), to: (i64, i64)) -> bool {
    (to.0 - from.0).abs() > 1 || (to.1 - from.1).abs() > 1
}

/// What a step on a level costs, in `step_cost` units: its distance, its slope
/// in the wilderness (with the heights at each end, `WildHeights::at`), less
/// by Jumping for a jump, and waded or slowed by its terrain as `footing`
/// says (`footing_on`). A good swimmer or jumper goes
/// quicker than they walk, so this can be less than `step_cost`: routes by it
/// allow for that (`Gait::cheapest_per_mille`).
pub fn level_step_cost(
    from: (i64, i64),
    to: (i64, i64),
    heights: Option<(i32, i32)>,
    footing: Footing,
    gait: Gait,
) -> u32 {
    let cost = match heights {
        Some(ends) => wild_step_cost(from, to, ends, gait),
        None => step_cost(from, to),
    };
    let cost = if is_jump(from, to) {
        gait.jump(cost)
    } else {
        cost
    };
    match (footing.wades(), footing.slowed_by()) {
        (true, _) => gait.swim(cost),
        (false, Some(terrain)) => gait.cross(terrain, cost),
        (false, None) => cost,
    }
}
