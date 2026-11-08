//! What steps on a level cost the player, as the server prices them: slopes in
//! the wilderness, wading, terrain, jumps, and a sneaking pace while sneaking.

use shared::dungeon::Level;
use shared::energy::{SITE_STEP_SCALE, sneak_step_energy};
use shared::movement::{Gait, footing_on, level_step_cost};

use crate::common::WildCellHeights;

/// Everything a level step's cost depends on.
pub(super) struct LevelCosts<'a> {
    pub level: &'a Level,
    pub heights: &'a WildCellHeights,
    pub gait: Gait,
    pub sneaking: bool,
    /// The player's speed, which a sneaking pace is worked out from.
    pub speed: u32,
}

impl LevelCosts<'_> {
    /// A step, in `step_cost` units, for routes: its distance, slope, wading
    /// and jumping.
    pub(super) fn cost(&self, from: (i64, i64), to: (i64, i64)) -> u32 {
        let footing = footing_on(self.level, to, self.heights.terrain(to));
        level_step_cost(from, to, self.heights.ends(from, to), footing, self.gait)
    }

    /// The energy a step takes: no quicker than a sneaking pace while sneaking.
    pub(super) fn time(&self, from: (i64, i64), to: (i64, i64)) -> u32 {
        let cost = self.cost(from, to) * SITE_STEP_SCALE;
        if self.sneaking {
            let cells = (to.0 - from.0).abs().max((to.1 - from.1).abs()) as u32;
            sneak_step_energy(cost, cells, self.speed)
        } else {
            cost
        }
    }
}
