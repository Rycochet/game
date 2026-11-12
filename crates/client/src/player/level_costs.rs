//! What steps on a level cost the player, as the server prices them: slopes in
//! the wilderness, wading, terrain, jumps, and a sneaking pace while sneaking.

use shared::clearance::{Other, crowd_per_mille};
use shared::creature::Size;
use shared::dungeon::Level;
use shared::energy::{SITE_STEP_SCALE, site_move_cost, site_step_cost, sneak_step_energy};
use shared::movement::{Gait, footing_on, level_step_cost, move_on_level};
use shared::spot::Spot;

use crate::common::WildCellHeights;

/// Everything a level step's cost depends on.
pub(super) struct LevelCosts<'a> {
    pub level: &'a Level,
    pub heights: &'a WildCellHeights,
    pub gait: Gait,
    pub sneaking: bool,
    /// The player's speed, which a sneaking pace is worked out from.
    pub speed: u32,
    /// How big the player is, and who else is about: squeezing through gaps
    /// and past others is slower (`shared::clearance`).
    pub size: Size,
    pub others: &'a [Other],
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

    /// The energy a move in units takes, as the server prices it: within one
    /// cell by its length alone; into another cell as a step between the two
    /// cells' middles, in proportion to its length; slower for squeezing.
    pub(super) fn move_time(&self, from: Spot, to: Spot) -> u32 {
        let (a, b) = (from.cell(), to.cell());
        let squeeze = u64::from(self.squeeze(from, to));
        if a == b {
            return (u64::from(site_move_cost(from, to)) * squeeze / 1000) as u32;
        }
        let between = Spot::middle_of(a).distance(Spot::middle_of(b)).max(1);
        (u64::from(self.time(a, b)) * from.distance(to) / between * squeeze / 1000) as u32
    }

    /// How much slower a move is for squeezing, in thousandths, as the server
    /// works it out: through a gap, or past someone.
    fn squeeze(&self, from: Spot, to: Spot) -> u32 {
        let landing = move_on_level(self.level, self.size, self.gait, from, to);
        let gap = landing.map_or(1000, |landing| landing.squeeze_per_mille());
        let crowd = crowd_per_mille(self.size, from, to, self.others).unwrap_or(1000);
        gap.max(crowd)
    }

    /// How much longer than on flat ground walking from cell `from` into `to`
    /// takes (less than 1 when a gait makes it quicker): slopes, wading,
    /// terrain and sneaking. Free movement is paced by distance, with this.
    pub(super) fn slowness(&self, from: (i64, i64), to: (i64, i64)) -> f64 {
        if from == to {
            return 1.0;
        }
        let flat = site_step_cost(from, to).max(1);
        f64::from(self.time(from, to)) / f64::from(flat)
    }
}
