//! A quest dungeon's stages (owner): on every level but its deepest, the door
//! that cuts the way down off from where the level is come into is locked,
//! and its key, a quest item, is carried by one of that level's creatures on
//! the near side. The door is the one nearest the stairs down, so as much of
//! the level as can be is before it.

use std::collections::BTreeSet;

use super::QuestDungeon;
use crate::door::reach_without;
use crate::dungeon::Cell;
use crate::site::{Exit, SiteLevel};

impl QuestDungeon {
    /// Level `index`'s stage door, if it has one: a door that, shut, leaves
    /// its stairs down out of reach from where it is come into. None on the
    /// deepest level, or where every way down has more than one door.
    pub fn stage_door(&self, index: u32, level: &SiteLevel) -> Option<(i64, i64)> {
        if index + 1 >= self.levels {
            return None;
        }
        let down = (level.stairs.iter()).find(|s| s.to == Exit::Level(index + 1))?;
        let down = (down.x, down.y);
        let from = match index {
            0 => Exit::WorldMap,
            _ => Exit::Level(index - 1),
        };
        let start = [level.arrive(from)];
        let (min_x, min_y, width, height) = level.level.bounds();
        let mut best: Option<(i64, (i64, i64))> = None;
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                if level.level.cell(x, y) != Cell::Door {
                    continue;
                }
                let shut = BTreeSet::from([(x, y)]);
                if reach_without(&level.level, &start, &shut).contains(&down) {
                    continue;
                }
                let apart = (x - down.0).abs().max((y - down.1).abs());
                if best.is_none_or(|(least, _)| apart < least) {
                    best = Some((apart, (x, y)));
                }
            }
        }
        best.map(|(_, at)| at)
    }

    /// Who of `holders` (spawn numbers and places on level `index`) carries the
    /// key to its stage door at `door`: the first reachable with it shut.
    pub fn key_holder(
        &self,
        index: u32,
        level: &SiteLevel,
        door: (i64, i64),
        holders: &[(u32, (i64, i64))],
    ) -> Option<u32> {
        let from = match index {
            0 => Exit::WorldMap,
            _ => Exit::Level(index - 1),
        };
        let shut = BTreeSet::from([door]);
        let near = reach_without(&level.level, &[level.arrive(from)], &shut);
        (holders.iter())
            .find(|(_, at)| near.contains(at))
            .map(|&(number, _)| number)
    }
}
