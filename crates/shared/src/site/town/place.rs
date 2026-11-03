//! Placing a plot on a road: choosing its size and the building's shape.

use super::super::grid::{chance, roll};
use super::{BUILDINGS_PER_GARDEN_SIDE, PLOTS_PER_GARDEN, Plot, ROADS, Town};

impl Town {
    /// Puts a garden or building on a road, `a` along it, extending the road
    /// in front of it if it needs to and can. `deep` asks for a depth, to line
    /// up with a neighbour. The new plot joins the list.
    pub(super) fn place_on(
        &mut self,
        index: usize,
        side: i64,
        a: i64,
        width: i64,
        deep: Option<i64>,
    ) -> bool {
        let smallest_garden = (self.target / BUILDINGS_PER_GARDEN_SIDE).max(3);
        let garden = self.rng.range_u64(0, PLOTS_PER_GARDEN) == 0;
        let (width, deep) = if garden {
            (
                roll(&mut self.rng, smallest_garden, smallest_garden + 2),
                roll(&mut self.rng, smallest_garden, smallest_garden + 2),
            )
        } else {
            let most = self.deepest();
            (width, deep.unwrap_or_else(|| roll(&mut self.rng, 3, most)))
        };
        let road = self.roads[index];
        let reaches = a >= road.along.0 && a + width <= road.along.1;
        let frame = road.frame(side, a);
        if !self.is_free(&frame, width, deep) {
            return false;
        }
        // The road in front, from where it ends to the far end of the plot.
        let extended = (road.along.0.min(a), road.along.1.max(a + width));
        if !reaches && road.fixed {
            return false;
        }
        let extra = [
            road.area(extended.0, road.along.0),
            road.area(road.along.1, extended.1),
        ];
        if !reaches && !self.extension_fits(index, extra) {
            return false;
        }
        if !garden && self.line_too_long(&frame, width, deep) {
            return false;
        }
        for area in extra {
            self.paint(area, ROADS[road.kind].0);
        }
        self.roads[index].along = extended;
        let placed = if garden {
            self.place_garden(&frame, width, deep)
        } else {
            let mask = self.building_shape(width, deep);
            self.place_building(&frame, width, deep, &mask, true)
        };
        if placed {
            self.roads[index].run += width;
            self.plots.push(Plot {
                road: index,
                side,
                along: (a, a + width),
                deep,
            });
        }
        placed
    }

    /// Which cells of a plot a building or garden covers: mostly all, in
    /// bigger plots sometimes an L, T or H. The front row is always kept
    /// wide enough for a door, and every arm is at least 2 wide.
    pub(super) fn building_shape(&mut self, width: i64, deep: i64) -> Vec<bool> {
        let mut mask = vec![true; (width * deep) as usize];
        let percent = (15 + self.target / 3).min(50) as u32;
        if width < 5 || deep < 5 || !chance(&mut self.rng, percent) {
            return mask;
        }
        let mut cut = |x0: i64, x1: i64, y0: i64, y1: i64| {
            for y in y0..y1 {
                for x in x0..x1 {
                    mask[(y * width + x) as usize] = false;
                }
            }
        };
        let choice = roll(&mut self.rng, 0, 2);
        if choice == 2 && width >= 8 {
            // An H: a bite out of the middle of the front and back, its arms
            // at least 3 wide, the bar between them at least 2 deep.
            let bite = roll(&mut self.rng, 2, width - 6);
            let x0 = (width - bite) / 2;
            cut(x0, x0 + bite, 0, 1);
            cut(x0, x0 + bite, deep - 1, deep);
        } else if choice == 1 && width >= 7 {
            // A T: the back corners gone, the stem at least 3 wide.
            let each = roll(&mut self.rng, 2, (width - 3) / 2);
            let bar = roll(&mut self.rng, 2, deep - 2);
            cut(0, each, bar, deep);
            cut(width - each, width, bar, deep);
        } else {
            // An L: one back corner gone.
            let across = roll(&mut self.rng, 2, width - 3);
            let bar = roll(&mut self.rng, 2, deep - 2);
            if chance(&mut self.rng, 50) {
                cut(0, across, bar, deep);
            } else {
                cut(width - across, width, bar, deep);
            }
        }
        mask
    }
}
