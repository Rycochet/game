//! One generated level of a site, and the ways out of it.

use super::farm::FieldLook;
use super::town::TownBuilding;
use super::wall::WallLook;
use super::{Generator, Stairs};
use crate::dungeon::Level;
use crate::poi::PoiKind;

/// Where a way out of a level leads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Exit {
    /// Back out to the world map, at the poi.
    WorldMap,
    /// Another level of the same site, by index.
    Level(u32),
}

/// One generated level of a site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SiteLevel {
    pub level: Level,
    pub stairs: Vec<Stairs>,
    /// Where you arrive from the world map, on the level you enter by.
    pub entrance: Option<(i64, i64)>,
    /// A town's buildings (no other level has any).
    pub buildings: Vec<TownBuilding>,
    /// A farm's fields, on its ground floor (no other level has any).
    pub fields: Vec<FieldLook>,
    /// Walls round the level's buildings (a walled fort's ground floor).
    pub walls: Vec<WallLook>,
}

#[cfg(test)]
mod tests;

/// Whether walking to `(x, y)` leaves for the world map: outside ground or
/// road on the very edge of the level (beside the edge, not just any ground).
pub fn leads_out(level: &Level, x: i64, y: i64) -> bool {
    let cell = level.cell(x, y);
    (cell == crate::dungeon::Cell::Ground || cell.is_road())
        && super::grid::STEPS
            .iter()
            .any(|(dx, dy)| !level.contains(x + dx, y + dy))
}

/// Whether the Leave key takes the player at `(x, y)` on level `index` of a
/// site behind a poi of `kind` out to the world map (owner's rules): only on
/// the level entered from the world map, from any outside cell, but in a cave
/// only from its edge (`leads_out`), not a skylight or ground inside the hill.
pub fn can_leave(kind: PoiKind, index: u32, level: &Level, x: i64, y: i64) -> bool {
    if index != 0 {
        return false;
    }
    match Generator::for_poi(kind) {
        Generator::Cave(_) => leads_out(level, x, y),
        _ => level.cell(x, y).is_outside(),
    }
}

impl SiteLevel {
    /// Where you arrive when you come in from `from`: the entrance from the
    /// world map, or the staircase that leads back to the level you came
    /// from.
    pub fn arrival(&self, from: Exit) -> Option<(i64, i64)> {
        self.stairs
            .iter()
            .find(|stairs| stairs.to == from)
            .map(|stairs| (stairs.x, stairs.y))
            .or(match from {
                Exit::WorldMap => self.entrance,
                Exit::Level(_) => None,
            })
    }

    /// As `arrival`, but somewhere even if nothing leads there (a jump
    /// between levels, from dev tools): the first stairs or the entrance.
    pub fn arrive(&self, from: Exit) -> (i64, i64) {
        self.arrival(from)
            .or_else(|| self.stairs.first().map(|s| (s.x, s.y)))
            .or(self.entrance)
            .unwrap_or((0, 0))
    }
}
