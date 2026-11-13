//! A wall round a town (owner, 2026-11-13): two cells of ground inside it,
//! three outside, and a gate of two doors wherever a road reaches it.
//!
//! Hamlets are walled most often, cities least; each size is mostly built of
//! its own material (hamlets logs, villages planks, towns stone, cities
//! dressed blocks), the others less likely the further they are from it.

use super::super::Rect;
use super::super::wall::{Gate, WallLook, WallMaterial, build_wall};
use super::{BORDER, Block, Town};

/// Ground left inside the wall, round everything built.
pub(super) const INSIDE: i64 = 2;

/// How far the level reaches round what is built when walled: the ground
/// inside, the wall, and the usual border outside it.
pub(super) const WALLED_BORDER: i64 = INSIDE + 1 + BORDER;

use WallMaterial::{Block as Blocks, Log, Plank, Stone};

/// What each size's wall may be made of (owner's figures, in percent).
pub(in crate::site) const HAMLET_WALLS: &[(WallMaterial, u32)] =
    &[(Log, 50), (Plank, 35), (Stone, 12), (Blocks, 3)];
pub(in crate::site) const VILLAGE_WALLS: &[(WallMaterial, u32)] =
    &[(Log, 22), (Plank, 50), (Stone, 22), (Blocks, 6)];
pub(in crate::site) const TOWN_WALLS: &[(WallMaterial, u32)] =
    &[(Log, 6), (Plank, 22), (Stone, 50), (Blocks, 22)];
pub(in crate::site) const CITY_WALLS: &[(WallMaterial, u32)] =
    &[(Log, 3), (Plank, 12), (Stone, 35), (Blocks, 50)];

impl Town {
    /// `built` grown until no road runs along where the wall would go: any
    /// road reaching the wall's rectangle has its whole width inside, so
    /// roads only ever cross the wall, through a gate.
    pub(super) fn wall_box(&self, mut built: Block) -> Block {
        loop {
            let reach = INSIDE + 1;
            let ring = Block {
                x0: built.x0 - reach,
                y0: built.y0 - reach,
                x1: built.x1 + reach,
                y1: built.y1 + reach,
            };
            let mut grown = built;
            for road in &self.roads {
                let area = road.area(road.along.0, road.along.1);
                let meets = area.x0 < ring.x1
                    && ring.x0 < area.x1
                    && area.y0 < ring.y1
                    && ring.y0 < area.y1;
                if !meets {
                    continue;
                }
                if road.east_west {
                    grown.y0 = grown.y0.min(road.across.0);
                    grown.y1 = grown.y1.max(road.across.1);
                } else {
                    grown.x0 = grown.x0.min(road.across.0);
                    grown.x1 = grown.x1.max(road.across.1);
                }
            }
            if (grown.x0, grown.y0, grown.x1, grown.y1) == (built.x0, built.y0, built.x1, built.y1)
            {
                return built;
            }
            built = grown;
        }
    }

    /// Builds the wall round `built` (from `wall_box`), with a gate centred on
    /// each road where it crosses, once roads run out to the level's edge.
    pub(super) fn build_town_wall(&mut self, built: Block, material: WallMaterial) -> WallLook {
        let reach = INSIDE + 1;
        let rect = Rect {
            x: built.x0 - reach,
            y: built.y0 - reach,
            width: built.width() + 2 * reach,
            height: built.y1 - built.y0 + 2 * reach,
        };
        let (x1, y1) = (rect.x + rect.width - 1, rect.y + rect.height - 1);
        let mut gates = Vec::new();
        // Along each side, corners left out: north, south, then west, east.
        for y in [rect.y, y1] {
            let cells: Vec<i64> = (rect.x + 1..x1).collect();
            for start in self.road_runs(&cells, |a| (a, y)) {
                gates.push(Gate {
                    x: start,
                    y,
                    runs_east_west: true,
                });
            }
        }
        for x in [rect.x, x1] {
            let cells: Vec<i64> = (rect.y + 1..y1).collect();
            for start in self.road_runs(&cells, |a| (x, a)) {
                gates.push(Gate {
                    x,
                    y: start,
                    runs_east_west: false,
                });
            }
        }
        let look = WallLook {
            rect,
            material,
            gates,
            towers: Vec::new(),
            gaps: Vec::new(),
        };
        build_wall(&mut self.level, &look);
        look
    }

    /// Where a gate goes on each run of road among `cells` along a side
    /// (`at` turning a place along it into a cell): the first of its two
    /// cells, in the middle of the run, and never past the side's end.
    fn road_runs(&self, cells: &[i64], at: impl Fn(i64) -> (i64, i64)) -> Vec<i64> {
        let road = |a: i64| {
            let (x, y) = at(a);
            self.level.cell(x, y).is_road()
        };
        let last = cells.last().copied().unwrap_or(0);
        let mut starts = Vec::new();
        let mut i = 0;
        while i < cells.len() {
            if !road(cells[i]) {
                i += 1;
                continue;
            }
            let first = cells[i];
            while i < cells.len() && road(cells[i]) {
                i += 1;
            }
            let length = cells[i - 1] - first + 1;
            starts.push((first + (length - 2).max(0) / 2).min(last - 1));
        }
        starts
    }
}
