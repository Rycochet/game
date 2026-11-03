//! A pool of water or lava in a cave.

use super::super::grid::{self, STEPS, roll};
use super::Cave;
use crate::dungeon::{Cell, Level, RoomId};
use crate::rng::Rng;

/// A small pool of `liquid` in a random cave, kept only if everything can
/// still be reached and it stays clear of the stairs and the way in.
pub(super) fn add_pool(
    level: &mut Level,
    rng: &mut Rng,
    caves: &[Cave],
    liquid: Cell,
    keep: &[(i64, i64)],
    arrival: (i64, i64),
) {
    let room = rng.range_u64(0, caves.len() as u64) as usize;
    let cave = caves[room];
    let clear = |x: i64, y: i64| {
        keep.iter()
            .all(|&(kx, ky)| (kx - x).abs() > 1 || (ky - y).abs() > 1)
    };
    let mut changed = Vec::new();
    let (mut x, mut y) = (
        cave.x + roll(rng, -cave.rx / 2, cave.rx / 2),
        cave.y + roll(rng, -cave.ry / 2, cave.ry / 2),
    );
    for _ in 0..roll(rng, 4, 14) {
        if level.cell(x, y) == Cell::Floor
            && level.room(x, y) == Some(RoomId(room as u16))
            && clear(x, y)
        {
            level.set(x, y, liquid);
            changed.push((x, y));
        }
        let (dx, dy) = STEPS[rng.range_u64(0, 4) as usize];
        if level.room(x + dx, y + dy) == Some(RoomId(room as u16)) {
            (x, y) = (x + dx, y + dy);
        }
    }
    if !grid::all_reached(level, &[arrival]) {
        for (x, y) in changed {
            level.set(x, y, Cell::Floor);
        }
    }
}
