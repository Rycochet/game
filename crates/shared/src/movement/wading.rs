//! Wading: steps into shallow water (`Cell::Shallows`), or swimming into deep
//! water (`Cell::Water`, for a good swimmer: `step_on_level_by`), on any level
//! take longer, less so with Swimming (`Gait::swim`).

use super::{Gait, step_cost, wild_step_cost};
use crate::dungeon::{Cell, Level};

/// Whether a step landing on `to` wades or swims.
pub fn wades(level: &Level, to: (i64, i64)) -> bool {
    matches!(level.cell(to.0, to.1), Cell::Shallows | Cell::Water)
}

/// What a step on a level costs, in `step_cost` units: its distance, its slope
/// in the wilderness (with the heights at each end, `WildHeights::at`), and
/// waded if `wading`. A good swimmer wades quicker than they walk, so this can
/// be less than `step_cost`: routes by it allow for that (`find_path_by`).
pub fn level_step_cost(
    from: (i64, i64),
    to: (i64, i64),
    heights: Option<(i32, i32)>,
    wading: bool,
    gait: Gait,
) -> u32 {
    let cost = match heights {
        Some(ends) => wild_step_cost(from, to, ends, gait),
        None => step_cost(from, to),
    };
    if wading { gait.swim(cost) } else { cost }
}
