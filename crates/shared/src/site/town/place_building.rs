//! Placing a garden or a building with its door.

use super::super::grid::roll;
use super::{Frame, Placed, Town};
use crate::dungeon::Cell;

impl Town {
    pub(super) fn place_garden(&mut self, frame: &Frame, width: i64, deep: i64) -> bool {
        if !self.is_free(frame, width, deep) {
            return false;
        }
        // Gardens never touch, so each keeps its own hedge.
        let touching = (-1..=deep).any(|d| {
            (-1..=width).any(|a| {
                let (x, y) = frame.at(a, d);
                self.level.cell(x, y) == Cell::Garden
            })
        });
        if touching {
            return false;
        }
        let mask = self.building_shape(width, deep);
        for d in 0..deep {
            for a in 0..width {
                if mask[(d * width + a) as usize] {
                    let (x, y) = frame.at(a, d);
                    self.level.set(x, y, Cell::Garden);
                }
            }
        }
        true
    }

    /// A building on a plot, its door in the front wall: facing a road if
    /// `on_road`, otherwise bare ground.
    pub(super) fn place_building(
        &mut self,
        frame: &Frame,
        width: i64,
        deep: i64,
        mask: &[bool],
        on_road: bool,
    ) -> bool {
        if !self.is_free(frame, width, deep) {
            return false;
        }
        let covered = |a: i64, d: i64| {
            (0..width).contains(&a) && (0..deep).contains(&d) && mask[(d * width + a) as usize]
        };
        // In a straight stretch of the front wall, with building behind.
        let doors: Vec<i64> = (0..width)
            .filter(|&a| {
                covered(a, 0) && covered(a - 1, 0) && covered(a + 1, 0) && covered(a, 1) && {
                    let (x, y) = frame.at(a, -1);
                    let cell = self.level.cell(x, y);
                    if on_road {
                        cell.is_road()
                    } else {
                        cell == Cell::Ground && self.inside(x, y)
                    }
                }
            })
            .collect();
        if doors.is_empty() {
            return false;
        }
        let door = doors[roll(&mut self.rng, 0, doors.len() as i64 - 1) as usize];
        let mut cells = Vec::new();
        for d in 0..deep {
            for a in 0..width {
                if covered(a, d) {
                    cells.push(frame.at(a, d));
                }
            }
        }
        self.add_building(cells, frame.at(door, 0), frame.out(), on_road);
        true
    }

    /// Records a building and marks its cells.
    fn add_building(
        &mut self,
        cells: Vec<(i64, i64)>,
        door: (i64, i64),
        faces: (i64, i64),
        on_road: bool,
    ) {
        for &(x, y) in &cells {
            self.level.set(x, y, Cell::Wall);
        }
        self.placed.push(Placed {
            cells,
            door,
            faces,
            on_road,
        });
    }
}
