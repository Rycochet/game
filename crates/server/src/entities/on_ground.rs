//! Mobs' moves on a level's own ground: slower up and down slopes and through
//! shallows, by the same shared rule as players' (`movement::LevelWay`).

use shared::dungeon::Level;
use shared::movement::{Gait, LevelWay};
use shared::relief::Relief;
use shared::spot::Spot;

/// What a mob's move from `from` to `to`, costing `cost` on flat dry ground
/// (squeezes and all), takes on this level's ground: `cost` in proportion to
/// what the ground makes of its length. A move two cells or more across is a
/// jump. Mobs have no movement skills yet.
pub(super) fn on_ground(
    level: &Level,
    relief: Option<&Relief>,
    (from, to): (Spot, Spot),
    cost: u32,
) -> u32 {
    let length = from.distance(to);
    if length == 0 {
        return cost;
    }
    let way = LevelWay {
        level,
        relief,
        terrain: None,
        gait: Gait::UNSKILLED,
    };
    let (a, b) = (from.cell(), to.cell());
    let jumped = (a.0 - b.0).abs().max((a.1 - b.1).abs()) > 1;
    (u64::from(cost) * way.cost(from, to, jumped) / length) as u32
}

#[cfg(test)]
mod tests;
