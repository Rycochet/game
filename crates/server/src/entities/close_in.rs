//! A mob closing in on an enemy in the next cell that it can't reach yet: its
//! body moved straight towards them, as far as its reach needs (free
//! positions, PR 7). Mobs think in cells, so walking gets them next to their
//! enemy; this is the last part of the way, which a small mob's short reach or
//! an enemy standing off their cell's middle may need.

use shared::clearance::check_segment;
use shared::combat::{Body, reach_between};
use shared::dungeon::Level;
use shared::energy::site_move_cost;
use shared::movement::mob_can_stand;
use shared::spot::HALF_SPAN;

use super::{Mob, Taken, cells_apart};

/// How far inside its reach a mob closing in stops, in units, so an enemy
/// shuffling a little is still reached.
const INSIDE: u64 = 20;

/// Moves `mob` (`me`, where it stands) towards `target` until it would reach
/// them with a weapon reaching `weapon` (`combat::reach_between`): at most half
/// a cell a turn, never into their circle, nor onto a cell anyone but the two
/// of them stands on. What it cost, slower for squeezing; `None` if it can't
/// (not in the next cell, already as near as it goes, or blocked).
pub(super) fn close_in(
    mob: &mut Mob,
    level: &Level,
    (me, target): (&Body, &Body),
    weapon: u64,
    taken: &Taken,
) -> Option<u32> {
    if cells_apart(me.at.cell(), target.at.cell()) > 1 {
        return None;
    }
    let touching = (me.size.radius() + target.size.radius()).unsigned_abs();
    let wanted =
        (reach_between(me.size, weapon, target.size).saturating_sub(INSIDE)).max(touching + 1);
    let apart = me.at.distance(target.at);
    if apart <= wanted {
        return None;
    }
    let by = (apart - wanted).min(HALF_SPAN as u64) as i64;
    let (dx, dy) = (target.at.x - me.at.x, target.at.y - me.at.y);
    let to = me.at.offset(dx * by / apart as i64, dy * by / apart as i64);
    let cell = to.cell();
    let others_there = cell != mob.position && cell != target.at.cell() && taken.contains(&cell);
    if others_there || !mob_can_stand(level, cell.0, cell.1) {
        return None;
    }
    let passage = check_segment(level, me.size, me.at, to)?;
    mob.place(to);
    let cost = u64::from(site_move_cost(me.at, to)) * u64::from(passage.squeeze_per_mille) / 1000;
    Some(cost.max(1) as u32)
}

#[cfg(test)]
mod tests;
