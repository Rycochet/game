//! What a move on a level takes, measured along it: its length, the smooth
//! ground's rises and falls (`relief::Relief`), wading and terrain, and jumps,
//! by the mover's gait. The one rule players' moves, mobs' moves and routes on
//! levels are priced by, on both sides.

use super::{Gait, SLOPE_COST, STRAIGHT_COST, Terrain, footing_on};
use crate::dungeon::Level;
use crate::relief::{Relief, UNITS_PER_16TH};
use crate::spot::{CELL_SPAN, Spot};

/// How far apart a move's ground is sampled, in units: a quarter of a cell,
/// so a hump between two spots still counts.
pub const PIECE: u64 = 60;

/// Where a move on a level goes, and who goes it.
#[derive(Clone, Copy, Debug)]
pub struct LevelWay<'a> {
    pub level: &'a Level,
    /// The level's ground, if it isn't flat (`relief::LevelGround::relief`).
    pub relief: Option<&'a Relief>,
    /// The wilderness's terrain where the move goes (`WildHeights::terrain`),
    /// slowing open ground and grass: it changes only every few world map
    /// points, so one is enough for a move.
    pub terrain: Option<Terrain>,
    pub gait: Gait,
}

impl LevelWay<'_> {
    /// What a move from `from` to `to` takes, in units of walking on flat open
    /// ground (a cell's width is `CELL_SPAN`): each quarter cell of it its
    /// length plus `SLOPE_COST` per 16th of an elevation it climbs or
    /// descends (less by Climbing), waded (by Swimming) in shallows or water,
    /// or slowed by its terrain (less by Pioneering). A jump (`jumped`) is
    /// taken whole: its length and slope, less by Jumping, then by where it
    /// lands. Not squeezes: those are the passage's (`clearance::Passage`).
    pub fn cost(&self, from: Spot, to: Spot, jumped: bool) -> u64 {
        if jumped {
            let cost = from.distance(to) + self.slope(self.climb(from, to));
            let cost = self.gait.jump(cost as u32);
            return u64::from(self.footed(to, cost));
        }
        self.pieces(from, to)
            .map(|(end, length, climb)| {
                let cost = length + self.slope(climb);
                u64::from(self.footed(end, cost as u32))
            })
            .sum()
    }

    /// How far a move climbs and descends, in height units (`Relief::at`):
    /// what Climbing is practised by. 0 on flat ground.
    pub fn climb(&self, from: Spot, to: Spot) -> u32 {
        self.pieces(from, to).map(|(_, _, climb)| climb).sum()
    }

    /// The pieces a move is sampled in: where each ends, its length (adding
    /// up to the move's exactly) and how far it climbs or descends.
    fn pieces(&self, from: Spot, to: Spot) -> impl Iterator<Item = (Spot, u64, u32)> + '_ {
        let length = from.distance(to);
        let count = length.div_ceil(PIECE).max(1) as i64;
        let height = |at: Spot| self.relief.map_or(0, |relief| relief.at(at.x, at.y));
        let mut last = (from, height(from), 0);
        (1..=count).map(move |i| {
            let end = from.towards(to, i, count);
            let gone = length * i as u64 / count as u64;
            let here = height(end);
            let piece = (end, gone - last.2, here.abs_diff(last.1));
            last = (end, here, gone);
            piece
        })
    }

    /// A climb's extra cost in units of walking, by Climbing.
    fn slope(&self, climb: u32) -> u64 {
        // SLOPE_COST is in step costs (STRAIGHT_COST a cell) per 16th.
        let cost = self.gait.climb(climb * SLOPE_COST);
        u64::from(cost) * CELL_SPAN as u64 / (UNITS_PER_16TH as u64 * u64::from(STRAIGHT_COST))
    }

    /// `cost` waded or slowed by the ground at `at`.
    fn footed(&self, at: Spot, cost: u32) -> u32 {
        let footing = footing_on(self.level, at.cell(), self.terrain);
        match (footing.wades(), footing.slowed_by()) {
            (true, _) => self.gait.swim(cost),
            (false, Some(terrain)) => self.gait.cross(terrain, cost),
            (false, None) => cost,
        }
    }

    /// The steepness of a move for practice, as the wilderness counts it:
    /// 16ths of an elevation a cell (`Moved::elevations`).
    pub fn steepness(&self, from: Spot, to: Spot) -> u32 {
        let length = from.distance(to).max(1);
        let climb = u64::from(self.climb(from, to));
        (climb * CELL_SPAN as u64 / (UNITS_PER_16TH as u64 * length)) as u32
    }
}

#[cfg(test)]
mod tests;
