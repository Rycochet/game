//! A cave's floor: a gentle swell from a hash of position, on a coarse grid
//! of points blended across the cells between them, nearly flat inside its
//! caves (cells with a `RoomId`) and level round liquid, so pools lie flat.

use super::Relief;
use crate::dungeon::Level;
use crate::rng::Seed;

/// How far a cave's floor rises or falls, at most, in units (a quarter of a
/// cell).
pub const CAVE_SWELL: i32 = 60;

/// The same inside its caves (`RoomId`): nearly flat.
pub const ROOM_SWELL: i32 = 12;

/// Cells between the points the swell is blended from.
const SPACING: i64 = 4;

/// The swell of `level`'s floor, from `seed` (`LevelGround::Cave`).
pub(super) fn cave(level: &Level, seed: u64) -> Relief {
    let seed = Seed::new(seed);
    let range = 2 * CAVE_SWELL as u64 + 1;
    let point = |px: i64, py: i64| (seed.at(&[px, py]) % range) as i64 - i64::from(CAVE_SWELL);
    Relief::from_fn(level.bounds(), |x, y| {
        let round = [(x - 1, y - 1), (x, y - 1), (x - 1, y), (x, y)];
        if round.iter().any(|&(cx, cy)| level.cell(cx, cy).is_liquid()) {
            return 0;
        }
        let (px, py) = (x.div_euclid(SPACING), y.div_euclid(SPACING));
        let (east, south) = (x.rem_euclid(SPACING), y.rem_euclid(SPACING));
        let blended = point(px, py) * (SPACING - east) * (SPACING - south)
            + point(px + 1, py) * east * (SPACING - south)
            + point(px, py + 1) * (SPACING - east) * south
            + point(px + 1, py + 1) * east * south;
        let swell = blended.div_euclid(SPACING * SPACING) as i32;
        let in_a_room = round.iter().any(|&(cx, cy)| level.room(cx, cy).is_some());
        match in_a_room {
            true => swell * ROOM_SWELL / CAVE_SWELL,
            false => swell,
        }
    })
}
