//! Finishing a town: roads out to the edge, and opening shut-in pockets.

use super::super::SiteLevel;
use super::super::grid::{Reach, STEPS};
use super::{BORDER, Placed, ROADS, Rect, Town};
use crate::dungeon::{Cell, Level};

impl Town {
    /// Opens up anything walled in, cuts the level down to what is used, and
    /// decides what each building is.
    pub(super) fn finish(mut self, arms: [bool; 4], middle: (i64, i64), main: i64) -> SiteLevel {
        let (_, _, span, _) = self.level.bounds();
        let entry = if arms[2] {
            (middle.0 + main / 2, span - 1)
        } else if arms[1] {
            (span - 1, middle.1 + main / 2)
        } else if arms[3] {
            (0, middle.1 + main / 2)
        } else {
            (middle.0 + main / 2, 0)
        };
        self.open_pockets(entry);

        // The used area plus the border, always including the middle.
        let (mut x0, mut y0) = (middle.0, middle.1);
        let (mut x1, mut y1) = (middle.0 + main, middle.1 + main);
        for y in 0..span {
            for x in 0..span {
                let cell = self.level.cell(x, y);
                if cell.is_wall() || cell == Cell::Garden {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x + 1);
                    y1 = y1.max(y + 1);
                }
            }
        }
        let (x0, y0) = (x0 - BORDER, y0 - BORDER);
        let (x1, y1) = (x1 + BORDER, y1 + BORDER);
        let mut level = Level::new(x0, y0, x1 - x0, y1 - y0);
        for y in y0..y1 {
            for x in x0..x1 {
                level.set(x, y, self.level.cell(x, y));
            }
        }
        let entrance = (entry.0.clamp(x0, x1 - 1), entry.1.clamp(y0, y1 - 1));
        self.level = level;
        self.roads_to_the_edge();

        let kinds = self.choose_kinds();
        let mut buildings = Vec::new();
        let placed = std::mem::take(&mut self.placed);
        for (index, (building, kind)) in placed.iter().zip(kinds).enumerate() {
            buildings.push(self.make_building(index, building, kind));
        }
        self.add_windows(&buildings);
        SiteLevel {
            level: self.level,
            stairs: Vec::new(),
            entrance: Some(entrance),
            buildings,
            fields: Vec::new(),
            walls: Vec::new(),
        }
    }

    /// Any road reaching the town's border carries on through it to the edge
    /// of the level, like the main roads, unless something is in the way.
    fn roads_to_the_edge(&mut self) {
        let (x0, y0, width, height) = self.level.bounds();
        for road in self.roads.clone() {
            let (first, last) = if road.east_west {
                (x0, x0 + width)
            } else {
                (y0, y0 + height)
            };
            // Each way, out from its end, until the edge or another road.
            let outwards: [Vec<i64>; 2] = [
                if road.along.0 <= first + BORDER {
                    (first..road.along.0).rev().collect()
                } else {
                    Vec::new()
                },
                if road.along.1 >= last - BORDER {
                    (road.along.1..last).collect()
                } else {
                    Vec::new()
                },
            ];
            for way in outwards {
                for a in way {
                    let area = road.area(a, a + 1);
                    let bare = (area.y0..area.y1)
                        .all(|y| (area.x0..area.x1).all(|x| self.level.cell(x, y) == Cell::Ground));
                    if !bare {
                        break;
                    }
                    self.paint(area, ROADS[road.kind].0);
                }
            }
        }
    }

    /// While any ground can't be walked to, takes away a building next to it
    /// (the first that also touches ground that can be).
    fn open_pockets(&mut self, entry: (i64, i64)) {
        loop {
            let mut reach = Reach::new(&self.level);
            reach.spread(&self.level, &[entry]);
            let Some(pocket) = reach.first_unreached(&self.level) else {
                return;
            };
            let mut inside = Reach::new(&self.level);
            inside.spread(&self.level, &[pocket]);
            let touches = |cells: &[(i64, i64)], reached: &Reach| {
                cells
                    .iter()
                    .any(|&(x, y)| STEPS.iter().any(|(dx, dy)| reached.is(x + dx, y + dy)))
            };
            let bordering = |p: &&Placed| touches(&p.cells, &inside);
            let best = self
                .placed
                .iter()
                .filter(bordering)
                .position(|p| touches(&p.cells, &reach));
            let index = match best {
                Some(_) => self
                    .placed
                    .iter()
                    .position(|p| touches(&p.cells, &inside) && touches(&p.cells, &reach)),
                None => self.placed.iter().position(|p| touches(&p.cells, &inside)),
            };
            let Some(index) = index else {
                return;
            };
            let gone = self.placed.remove(index);
            for (x, y) in gone.cells {
                self.level.set(x, y, Cell::Ground);
            }
        }
    }
}

/// Cells as rectangles that don't overlap: runs along each row, merged with
/// the same run on the row before.
pub(super) fn rects_of(cells: &[(i64, i64)]) -> Vec<Rect> {
    let mut sorted: Vec<(i64, i64)> = cells.iter().map(|&(x, y)| (y, x)).collect();
    sorted.sort_unstable();
    let mut rects: Vec<Rect> = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let (y, x) = sorted[i];
        let mut end = i + 1;
        while end < sorted.len() && sorted[end] == (y, x + (end - i) as i64) {
            end += 1;
        }
        let width = (end - i) as i64;
        match rects
            .iter_mut()
            .find(|r| r.x == x && r.width == width && r.y + r.height == y)
        {
            Some(rect) => rect.height += 1,
            None => rects.push(Rect {
                x,
                y,
                width,
                height: 1,
            }),
        }
        i = end;
    }
    rects
}
