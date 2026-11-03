//! Whether a road, building or garden fits where it would go.

use super::{BETWEEN_ROADS, BORDER, Block, Frame, LONGEST_WITHOUT_GAP, Road, Town};
use crate::dungeon::Cell;

impl Town {
    /// Whether a road could run over these cells: only bare ground inside the
    /// town's border (it never replaces another road), and no road running the
    /// same way beside it without room for a building between. `own` is the
    /// road being extended, if any.
    pub(super) fn road_fits(&self, road: &Road, area: Block, own: Option<usize>) -> bool {
        let bare = (area.y0..area.y1).all(|y| {
            (area.x0..area.x1).all(|x| self.inside(x, y) && self.level.cell(x, y) == Cell::Ground)
        });
        let along = if road.east_west {
            (area.x0, area.x1)
        } else {
            (area.y0, area.y1)
        };
        bare && self.roads.iter().enumerate().all(|(index, other)| {
            let overlap = along.0 <= other.along.1 && other.along.0 <= along.1;
            Some(index) == own
                || other.east_west != road.east_west
                || !overlap
                || other.across.0 >= road.across.1 + BETWEEN_ROADS
                || road.across.0 >= other.across.1 + BETWEEN_ROADS
        })
    }

    /// Whether a road can be extended over these areas.
    pub(super) fn extension_fits(&self, index: usize, areas: [Block; 2]) -> bool {
        let road = self.roads[index];
        areas.iter().all(|&area| {
            area.x0 >= area.x1 || area.y0 >= area.y1 || self.road_fits(&road, area, Some(index))
        })
    }

    /// Whether a building here would carry on a straight line of buildings
    /// that is already too long, or join two into one: it may take a line past
    /// the limit itself, but not build on one that has reached it.
    pub(super) fn line_too_long(&self, frame: &Frame, width: i64, deep: i64) -> bool {
        let run = |start: (i64, i64), step: (i64, i64)| {
            let mut count = 0;
            let mut at = start;
            while self.level.cell(at.0, at.1).is_wall() && self.inside(at.0, at.1) {
                count += 1;
                at = (at.0 + step.0, at.1 + step.1);
            }
            count
        };
        let along = frame.along;
        let inward = frame.inward;
        let back = (-along.0, -along.1);
        let out = frame.out();
        (0..deep).any(|d| {
            run(frame.at(-1, d), back) + run(frame.at(width, d), along) >= LONGEST_WITHOUT_GAP
        }) || (0..width).any(|a| {
            run(frame.at(a, -1), out) + run(frame.at(a, deep), inward) >= LONGEST_WITHOUT_GAP
        })
    }

    /// Whether a plot is all bare ground.
    pub(super) fn is_free(&self, frame: &Frame, width: i64, deep: i64) -> bool {
        (0..deep).all(|d| {
            (0..width).all(|a| {
                let (x, y) = frame.at(a, d);
                self.inside(x, y)
                    && self.level.cell(x, y) == Cell::Ground
                    && !self.door_fronts.contains(&(x, y))
            })
        })
    }

    /// Inside the town's border.
    pub(super) fn inside(&self, x: i64, y: i64) -> bool {
        let (_, _, span, _) = self.level.bounds();
        (BORDER..span - BORDER).contains(&x) && (BORDER..span - BORDER).contains(&y)
    }
}
