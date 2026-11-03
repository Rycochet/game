//! Growing round a corner onto a new side road, and behind other buildings.

use super::super::grid::{chance, roll};
use super::{Frame, Plot, ROADS, Road, Town};
use crate::dungeon::Cell;

impl Town {
    /// A new side road past the plot's end, of a surface no better, running
    /// back along its flank, with something on it: behind the plot, or on its
    /// far side level with the plot's front.
    pub(super) fn add_corner(&mut self, plot: Plot, sign: i64) -> bool {
        let road = self.roads[plot.road];
        let wide = (road.across.1 - road.across.0 - 1).clamp(2, 3);
        let band = if sign > 0 {
            (plot.along.1, plot.along.1 + wide)
        } else {
            (plot.along.0 - wide, plot.along.0)
        };
        let widest = self.widest();
        let width = roll(&mut self.rng, 3, widest);
        let length = plot.deep + width;
        let along = if plot.side > 0 {
            (road.across.1, road.across.1 + length)
        } else {
            (road.across.0 - length, road.across.0)
        };
        let side_road = Road {
            east_west: !road.east_west,
            across: band,
            along,
            kind: self.road_kind(road.kind),
            fixed: false,
            run: 0,
        };
        let area = side_road.area(along.0, along.1);
        if !self.road_fits(&side_road, area, None) {
            return false;
        }
        // It has to meet this road, which may need to reach it first.
        let reaches = band.0 >= road.along.0 && band.1 <= road.along.1;
        let extended = (road.along.0.min(band.0), road.along.1.max(band.1));
        let extra = [
            road.area(extended.0, road.along.0),
            road.area(road.along.1, extended.1),
        ];
        if !reaches && (road.fixed || !self.extension_fits(plot.road, extra)) {
            return false;
        }
        // Painted first, as doors look for a road; put back if nothing fits.
        let mut painted = Vec::new();
        for y in area.y0..area.y1 {
            for x in area.x0..area.x1 {
                if self.level.cell(x, y) == Cell::Ground {
                    self.level.set(x, y, ROADS[side_road.kind].0);
                    painted.push((x, y));
                }
            }
        }
        let index = self.roads.len();
        self.roads.push(side_road);
        // Towards the plot's side of the new road, starting behind the plot.
        let (towards, away) = (-sign, sign);
        let behind = if plot.side > 0 {
            along.0 + plot.deep
        } else {
            along.1 - plot.deep - width
        };
        let level_with = if plot.side > 0 {
            along.0
        } else {
            along.1 - width
        };
        let tries = if chance(&mut self.rng, 60) {
            [(towards, behind), (away, level_with)]
        } else {
            [(away, level_with), (towards, behind)]
        };
        if tries
            .into_iter()
            .any(|(side, a)| self.place_on(index, side, a, width, None))
        {
            for part in extra {
                self.paint(part, ROADS[road.kind].0);
            }
            self.roads[plot.road].along = extended;
            self.roads[plot.road].run = 0;
            return true;
        }
        self.roads.pop();
        for (x, y) in painted {
            self.level.set(x, y, Cell::Ground);
        }
        false
    }

    /// A house behind the plot, lined up with one end of it, its door away
    /// from the plot onto bare ground.
    pub(super) fn add_behind(&mut self, plot: Plot) -> bool {
        let front = self.roads[plot.road].frame(plot.side, plot.along.0);
        let room = plot.along.1 - plot.along.0;
        if room < 3 {
            return false;
        }
        let width = roll(&mut self.rng, 3, room.min(5));
        let deep = roll(&mut self.rng, 3, 4);
        let gap = roll(&mut self.rng, 1, 2);
        let offset = if chance(&mut self.rng, 50) {
            0
        } else {
            room - width
        };
        let frame = Frame {
            corner: front.at(offset, plot.deep + gap + deep - 1),
            along: front.along,
            inward: front.out(),
        };
        let mask = vec![true; (width * deep) as usize];
        if self.line_too_long(&frame, width, deep)
            || !self.place_building(&frame, width, deep, &mask, false)
        {
            return false;
        }
        self.behind += 1;
        if let Some(placed) = self.placed.last() {
            let (x, y) = placed.door;
            self.door_fronts
                .push((x + placed.faces.0, y + placed.faces.1));
        }
        true
    }
}
