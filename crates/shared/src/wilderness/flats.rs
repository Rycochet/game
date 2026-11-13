//! A cutout's covered ground kept level (owner): its cells that aren't out in
//! the open (`Cell::is_outside`) fall into groups, each held at one height,
//! the land's at its ways in, while its outside cells follow the land. So a
//! building's door is level with the street outside it, and a castle's wall
//! may dip under the slope or stand above it. A town's wall is left to follow
//! the land, or it would hold the whole town at one height.

use super::Cutout;
use crate::movement::{FlatGround, WildHeights, WorldGround};

/// Connected covered cells held at one height.
struct Group {
    cells: Vec<(i64, i64)>,
    /// `(min_x, min_y, max_x, max_y)`, inclusive.
    corners: (i64, i64, i64, i64),
}

impl Group {
    fn overlaps(&self, other: &Group) -> bool {
        let (a, b) = (self.corners, other.corners);
        a.0 <= b.2 && b.0 <= a.2 && a.1 <= b.3 && b.1 <= a.3
    }

    fn take_in(&mut self, other: Group) {
        let (a, b) = (self.corners, other.corners);
        self.corners = (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3));
        self.cells.extend(other.cells);
    }

    fn bounds(&self) -> (i64, i64, i64, i64) {
        let (min_x, min_y, max_x, max_y) = self.corners;
        (min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
    }
}

const SIDES: [(i64, i64); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

impl Cutout {
    /// Its covered ground kept level, a `FlatGround` for each group: cells
    /// joined side by side (in a town only within one building, so each
    /// stands at its own height), groups whose rectangles overlap joined
    /// into one. Each is held at the land's average height just outside its
    /// ways in (where its walkable cells meet open ground), or at its middle
    /// if it has none.
    pub fn flats(&self, ground: &WorldGround) -> Vec<FlatGround> {
        let (min_x, min_y, width, height) = self.level().bounds();
        let land = WildHeights::new(ground, (min_x - 1, min_y - 1, width + 2, height + 2));
        joined(self.groups())
            .iter()
            .map(|group| FlatGround {
                bounds: group.bounds(),
                height: self.way_in_height(&land, group),
            })
            .collect()
    }

    /// Whether a cell is its own and covered: not out in the open. In a
    /// town only a building's cells are: its wall (in no room) follows the
    /// land like the streets.
    fn covered(&self, (x, y): (i64, i64)) -> bool {
        let level = self.level();
        let walling = self.poi().kind.is_town() && level.room(x, y).is_none();
        self.contains(x, y) && !level.cell(x, y).is_outside() && !walling
    }

    /// Its covered cells in connected groups, row by row.
    fn groups(&self) -> Vec<Group> {
        let level = self.level();
        let (min_x, min_y, width, height) = level.bounds();
        let by_building = self.poi().kind.is_town();
        let mut grouped = vec![false; (width * height) as usize];
        let index = |(x, y): (i64, i64)| ((y - min_y) * width + x - min_x) as usize;
        let mut groups = Vec::new();
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                if grouped[index((x, y))] || !self.covered((x, y)) {
                    continue;
                }
                grouped[index((x, y))] = true;
                let (mut cells, mut queue) = (Vec::new(), vec![(x, y)]);
                while let Some(at) = queue.pop() {
                    cells.push(at);
                    for (dx, dy) in SIDES {
                        let next = (at.0 + dx, at.1 + dy);
                        let same = !by_building || level.room(next.0, next.1) == level.room(x, y);
                        if self.covered(next) && same && !grouped[index(next)] {
                            grouped[index(next)] = true;
                            queue.push(next);
                        }
                    }
                }
                let xs = cells.iter().map(|c| c.0);
                let ys = cells.iter().map(|c| c.1);
                let corners = (
                    xs.clone().min().unwrap_or(x),
                    ys.clone().min().unwrap_or(y),
                    xs.max().unwrap_or(x),
                    ys.max().unwrap_or(y),
                );
                groups.push(Group { cells, corners });
            }
        }
        groups
    }

    /// The land's average height (rounded) on the open cells beside a group's
    /// walkable cells, or at its middle with none.
    fn way_in_height(&self, land: &WildHeights, group: &Group) -> i32 {
        let level = self.level();
        let (mut sum, mut count) = (0i64, 0i64);
        for &(x, y) in &group.cells {
            if !level.is_walkable(x, y) {
                continue;
            }
            for (dx, dy) in SIDES {
                let next = (x + dx, y + dy);
                if !self.covered(next) {
                    sum += i64::from(land.at(next.0, next.1));
                    count += 1;
                }
            }
        }
        if count == 0 {
            let (min_x, min_y, width, height) = group.bounds();
            return land.at(min_x + width / 2, min_y + height / 2);
        }
        (2 * sum + count).div_euclid(2 * count) as i32
    }
}

/// `groups` with any whose rectangles overlap joined, until none do: each
/// in turn takes in those already done that it overlaps (growing as it does),
/// so none of those done ever overlap.
fn joined(groups: Vec<Group>) -> Vec<Group> {
    let mut done: Vec<Group> = Vec::new();
    for mut group in groups {
        while let Some(i) = done.iter().position(|other| group.overlaps(other)) {
            group.take_in(done.remove(i));
        }
        done.push(group);
    }
    done.sort_by_key(|group| group.corners);
    done
}

#[cfg(test)]
mod tests;
