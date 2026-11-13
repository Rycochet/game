//! Where on the level a player is a quest's dots can go (`quests::dots`):
//! stairs towards another level of the site, a giver's door, and a poi in the
//! wilderness. The quests side decides which are known.

use shared::poi::{ChunkPos, Poi};
use shared::site::Exit;
use shared::wilderness::SCALE;

use super::{Place, Visit};
use crate::ServerWorld;
use crate::storage::SiteId;

impl Visit {
    /// The stairs on this level of a site that lead nearer its level `goal`
    /// (by storey: a building's floors go up from the ground and its basement
    /// is below, a cave's levels go down), that `known` says the character
    /// remembers. None in the wilderness, or on `goal` itself.
    pub(crate) fn stairs_towards(
        &self,
        goal: u32,
        known: impl Fn((i64, i64)) -> bool,
    ) -> Vec<(i64, i64)> {
        let Place::Site { site, .. } = &self.place else {
            return Vec::new();
        };
        let last = site.levels().saturating_sub(1);
        let basement = site.is_building() && site.has_basement();
        let storey = |index: u32| -> i64 {
            if basement && index == last {
                -1
            } else {
                index as i64
            }
        };
        let off = |index: u32| (storey(index) - storey(goal)).abs();
        let here = off(self.index);
        (self.level.stairs.iter())
            .filter(|stairs| matches!(stairs.to, Exit::Level(other) if off(other) < here))
            .map(|stairs| (stairs.x, stairs.y))
            .filter(|&cell| known(cell))
            .collect()
    }

    /// The door of building `building` of the town in `town`, if the player is
    /// in that town: on its own first level, or where it is set into the
    /// wilderness round them. In this level's cells.
    pub(crate) fn giver_door(
        &self,
        world: &mut ServerWorld,
        town: ChunkPos,
        building: u16,
    ) -> Option<(i64, i64)> {
        if !self.is_wilderness() {
            let here = self.poi().is_some_and(|poi| poi.chunk == town) && self.index == 0;
            let mut buildings = self.level.buildings.iter();
            let found = buildings.find(|b| b.look.id == building);
            return found.filter(|_| here).map(|b| b.look.door);
        }
        let mirror = (self.mirrors.iter()).find(|m| m.id == SiteId::Poi(town))?;
        let offset = mirror.offset;
        let chunk = super::wild_chunk(world, town)?;
        let found = chunk.town.iter().find(|b| b.look.id == building)?;
        let (x, y) = found.look.door;
        Some((x - offset.0, y - offset.1))
    }

    /// Where `poi` is in the wilderness round the player, in this level's
    /// cells: its way in if it is set into the window, else the middle of its
    /// world map point, the near way round the planet. `None` in a site.
    pub(crate) fn wild_cell_of(&self, poi: &Poi) -> Option<(i64, i64)> {
        let Place::Wilderness { cutouts, width, .. } = &self.place else {
            return None;
        };
        if let Some(set_in) = cutouts.iter().find(|c| c.chunk == poi.chunk) {
            let (min_x, min_y, columns, rows) = set_in.bounds;
            return Some(
                set_in
                    .entrance
                    .unwrap_or((min_x + columns / 2, min_y + rows / 2)),
            );
        }
        let x = poi.x * SCALE + SCALE / 2;
        let from = self.position().0;
        let near = match *width {
            0 => x,
            width => from + (x - from + width / 2).rem_euclid(width) - width / 2,
        };
        Some((near, poi.y * SCALE + SCALE / 2))
    }
}
