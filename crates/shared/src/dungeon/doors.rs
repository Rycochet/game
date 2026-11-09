//! Doors on a level: how each hangs, and what that means for the rules. A door
//! whose swing hasn't been set acts as before doors could close: a door open,
//! a secret door closed (a wall).

use super::{CELL_UNITS, Cell, HALF_CELL, Level, WALL_HALF_UNITS, WallBar};
use crate::door::Swing;

impl Level {
    /// How the door at `(x, y)` hangs, if it has been set.
    pub fn swing(&self, x: i64, y: i64) -> Option<Swing> {
        self.swings.get(&(x, y)).copied()
    }

    /// Sets how the door at `(x, y)` hangs; nothing if it isn't a door.
    pub fn set_swing(&mut self, x: i64, y: i64, swing: Swing) {
        if matches!(self.cell(x, y), Cell::Door | Cell::SecretDoor) {
            self.swings.insert((x, y), swing);
        }
    }

    /// Every door's swing that has been set, by cell.
    pub fn swings(&self) -> impl Iterator<Item = ((i64, i64), Swing)> + '_ {
        self.swings.iter().map(|(&at, &swing)| (at, swing))
    }

    /// Whether `(x, y)` is a closed door (a secret door unless it is open).
    pub fn is_shut(&self, x: i64, y: i64) -> bool {
        match self.cell(x, y) {
            Cell::Door => self.swing(x, y).is_some_and(Swing::is_shut),
            Cell::SecretDoor => self.swing(x, y).is_none_or(Swing::is_shut),
            _ => false,
        }
    }

    /// Whether `(x, y)` is a wall: a secret door is one while it is closed.
    pub fn is_wall(&self, x: i64, y: i64) -> bool {
        let cell = self.cell(x, y);
        cell.is_wall() && !(cell == Cell::SecretDoor && !self.is_shut(x, y))
    }

    /// Whether `(x, y)` is a hole in a wall (a door, open or not, or a window),
    /// joining the walls either side: an open secret door is one too.
    pub fn is_opening(&self, x: i64, y: i64) -> bool {
        let cell = self.cell(x, y);
        cell.is_opening() || (cell == Cell::SecretDoor && !self.is_shut(x, y))
    }

    /// Where a closed door at `(x, y)` stands, if there is one: a bar the
    /// width of its cell, along its wall. (A closed secret door is a wall.)
    pub fn door_bar(&self, x: i64, y: i64) -> Option<WallBar> {
        if self.cell(x, y) != Cell::Door || !self.is_shut(x, y) {
            return None;
        }
        let (cx, cy) = (x * CELL_UNITS, y * CELL_UNITS);
        let (t, h) = (WALL_HALF_UNITS, HALF_CELL);
        Some(if self.opening_runs_east_west(x, y) {
            ((cx - h, cy - t), (cx + h, cy + t))
        } else {
            ((cx - t, cy - h), (cx + t, cy + h))
        })
    }

    /// The same level with every closed door open (locked ones too, for
    /// now): for finding routes through doors someone can open on the way.
    pub fn with_doors_open(&self) -> Level {
        let mut open = self.clone();
        let shut: Vec<(i64, i64)> = (open.swings.iter())
            .filter(|(_, swing)| swing.is_shut())
            .map(|(&at, _)| at)
            .collect();
        for at in shut {
            // Which side doesn't matter for routes.
            open.swings
                .insert(at, Swing::Open(crate::door::Side::North));
        }
        // Secret doors with no swing set are closed too.
        let (min_x, min_y, width, height) = open.bounds();
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                if open.cell(x, y) == Cell::SecretDoor && open.swing(x, y).is_none() {
                    open.swings
                        .insert((x, y), Swing::Open(crate::door::Side::North));
                }
            }
        }
        open
    }
}
