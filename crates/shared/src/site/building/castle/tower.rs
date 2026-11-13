//! One of a castle's towers: a square room on each of its floors, its own
//! stairs between them, and a door into the courtyard.

use super::super::super::{Exit, Stairs, StairsDirection};
use super::TowerSpot;
use crate::dungeon::{Cell, Level, RoomId};
use crate::site::Rect;

/// A tower as planned.
#[derive(Clone, Debug)]
pub(in crate::site) struct Tower {
    pub(super) spot: TowerSpot,
    pub(super) floors: u32,
    /// Its door, on the ground floor.
    door: (i64, i64),
    /// `stairs[n]` joins floor `n` to floor `n + 1`.
    stairs: Vec<(i64, i64)>,
}

impl Tower {
    /// The tower at `spot` with `floors` floors, on the wall round `wall`
    /// among towers at `spots`.
    pub(super) fn new(spot: TowerSpot, floors: u32, wall: Rect, spots: &[TowerSpot]) -> Self {
        let rect = spot.rect();
        // Two cells in from opposite corners: never where the door leads in,
        // which is beside its wall.
        let stairs = [(2, 2), (rect.width - 3, rect.height - 3)]
            .into_iter()
            .take(floors.saturating_sub(1) as usize)
            .map(|(dx, dy)| (rect.x + dx, rect.y + dy))
            .collect();
        Self {
            spot,
            floors,
            door: door(rect, wall, spots),
            stairs,
        }
    }

    /// Builds its floor `index` on `level` as room `room`: the stairs on it.
    pub(super) fn build(&self, level: &mut Level, index: u32, room: RoomId) -> Vec<Stairs> {
        let rect = self.spot.rect();
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                if is_edge(rect, x, y) {
                    level.set(x, y, Cell::Wall);
                    level.set_room(x, y, None);
                } else {
                    level.set(x, y, Cell::Floor);
                    level.set_room(x, y, Some(room));
                }
            }
        }
        if index == 0 {
            level.set(self.door.0, self.door.1, Cell::Door);
        }
        let mut stairs = Vec::new();
        let n = index as usize;
        if n > 0 {
            let (x, y) = self.stairs[n - 1];
            stairs.push(Stairs {
                x,
                y,
                direction: StairsDirection::Down,
                to: Exit::Level(index - 1),
            });
        }
        if index + 1 < self.floors {
            let (x, y) = self.stairs[n];
            stairs.push(Stairs {
                x,
                y,
                direction: StairsDirection::Up,
                to: Exit::Level(index + 1),
            });
        }
        stairs
    }
}

fn is_edge(rect: Rect, x: i64, y: i64) -> bool {
    x == rect.x || y == rect.y || x == rect.x + rect.width - 1 || y == rect.y + rect.height - 1
}

/// The door of a tower on `rect`: on its edge, not a corner, between its room
/// and the courtyard (inside the wall round `wall`, off the wall itself and
/// every tower), and of those the nearest the middle of the castle.
fn door(rect: Rect, wall: Rect, spots: &[TowerSpot]) -> (i64, i64) {
    let (x1, y1) = (wall.x + wall.width - 1, wall.y + wall.height - 1);
    let courtyard = |x: i64, y: i64| {
        x > wall.x
            && x < x1
            && y > wall.y
            && y < y1
            && !spots.iter().any(|spot| spot.rect().contains(x, y))
    };
    // Twice the castle's middle, so it stays in whole numbers.
    let middle = (2 * wall.x + wall.width - 1, 2 * wall.y + wall.height - 1);
    let distance = |(x, y): (i64, i64)| (2 * x - middle.0).pow(2) + (2 * y - middle.1).pow(2);
    let inside = |x: i64, y: i64| rect.contains(x, y) && !is_edge(rect, x, y);
    (rect.y..rect.y + rect.height)
        .flat_map(|y| (rect.x..rect.x + rect.width).map(move |x| (x, y)))
        .filter(|&(x, y)| is_edge(rect, x, y))
        .filter(|&(x, y)| {
            [(0, 1), (1, 0), (0, -1), (-1, 0)]
                .iter()
                .any(|&(dx, dy)| courtyard(x + dx, y + dy) && inside(x - dx, y - dy))
        })
        .min_by_key(|&spot| distance(spot))
        .expect("a tower on a wall has a side in the courtyard")
}
