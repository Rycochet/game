//! What grows in a herbalist's patch: only looks, chosen here, the same for
//! everyone since it comes from the server's clock and the place.
//!
//! The ground is always the land's own (`Cell::HerbBed` is drawn as outside
//! ground). Plants grow one after another down a fixed list, each through its
//! four stages a game day apart: bare, dots, stalks, then full grown. Each tile
//! starts somewhere of its own in that list, down to the hour, so tiles are at
//! different stages and move on at different times of day. Plants have a mesh of
//! their own, which only changes with what is in sight and the hour.

use bevy::prelude::*;
use shared::dungeon::{Cell, Level, Sight};
use shared::rng::Seed;

use super::CELL_SIZE;
use super::mesh::Builder;
use super::town::cell_centre;

/// The plants' picture: a column for each shape (herbs, then mushrooms); rows
/// of dots, stalks, then full grown in each colour. `tools/make_town_atlas.py`
/// must match.
pub const HERBS_PICTURE: &str = "herb_plants.png";
const COLUMNS: f32 = 10.0;
const ROWS: f32 = 5.0;

/// Every plant, as its column and full-grown row in the picture, in the order
/// they follow each other: mixed up by hand, herbs and mushrooms together.
const PLANT_LIST: [(u8, u8); 30] = [
    (3, 3),
    (7, 2),
    (0, 4),
    (9, 3),
    (2, 2),
    (5, 4),
    (4, 3),
    (8, 2),
    (1, 2),
    (6, 3),
    (3, 4),
    (5, 2),
    (0, 2),
    (9, 4),
    (2, 3),
    (7, 3),
    (4, 2),
    (6, 4),
    (1, 4),
    (8, 3),
    (2, 4),
    (5, 3),
    (3, 2),
    (9, 2),
    (0, 3),
    (6, 2),
    (4, 4),
    (7, 4),
    (1, 3),
    (8, 4),
];

/// Game hours between stages, and how many stages a plant has.
const HOURS_PER_STAGE: u32 = 24;
const STAGES: u32 = 4;

/// How long the whole list takes, in game hours: a tile starts anywhere in it.
const LIST_HOURS: u64 = PLANT_LIST.len() as u64 * STAGES as u64 * HOURS_PER_STAGE as u64;

const HALF_CELL: f32 = CELL_SIZE / 2.0;

/// How far dots lie above the ground, to stay in front of it.
const DOTS_LIFT: f32 = 0.05;

/// Where upright plants stand, each way, as a share of a cell from its centre:
/// through the middle of the two plants on each picture, making a `#`.
const PLANT_ROWS: [f32; 2] = [-0.25, 0.25];

/// For choosing where each tile starts.
const STARTS: Seed = Seed::new(0x68_6572_6273);

/// How many stages a shift can go before everything looks the same again.
pub const HERB_STAGES: u32 = (LIST_HOURS / HOURS_PER_STAGE as u64) as u32;

/// The hour to grow by: the clock's, and whole stages more to look at them.
pub fn growth(hour: u32, shift: u32) -> u32 {
    hour.wrapping_add(shift.wrapping_mul(HOURS_PER_STAGE))
}

/// Which of `PLANT_LIST` a tile has at an hour, and at which stage.
fn tile_growth(place: (i64, i64), (x, y): (i64, i64), hour: u32) -> (usize, u32) {
    let start = STARTS.at(&[place.0, place.1, x, y]) % LIST_HOURS;
    let hours = (u64::from(hour) + start) % LIST_HOURS;
    let stage = hours / u64::from(HOURS_PER_STAGE);
    (
        (stage / u64::from(STAGES)) as usize,
        (stage % u64::from(STAGES)) as u32,
    )
}

/// A tile of the plants' picture, at `(u, v)` within it.
fn plant_uv(column: u8, row: u8, u: f32, v: f32) -> [f32; 2] {
    [
        (f32::from(column) + u) / COLUMNS,
        (f32::from(row) + v) / ROWS,
    ]
}

/// Everything growing where anyone can see it. Nothing is drawn of it from
/// memory: it will have grown since.
pub fn build_plants(
    level: &Level,
    sight: &Sight,
    remembered: &Sight,
    place: (i64, i64),
    hour: u32,
) -> Builder {
    let mut plants = Builder::default();
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            let here = level.cell(x, y) == Cell::HerbBed
                && sight.is_seen(x, y)
                && remembered.is_seen(x, y);
            if here {
                let (plant, stage) = tile_growth(place, (x, y), hour);
                tile(&mut plants, cell_centre(x, y), PLANT_LIST[plant], stage);
            }
        }
    }
    plants
}

/// One tile's plants: nothing while bare, dots flat on the ground, the rest
/// upright in a `#`.
fn tile(plants: &mut Builder, centre: Vec3, (column, full): (u8, u8), stage: u32) {
    let colours = [Vec3::ONE; 4];
    if stage == 0 {
        return;
    }
    if stage == 1 {
        let at = |east: f32, south: f32| centre + Vec3::new(east, DOTS_LIFT, south);
        let h = HALF_CELL;
        plants.free(
            [at(-h, -h), at(h, -h), at(h, h), at(-h, h)],
            Vec3::Y,
            [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
                .map(|(u, v)| plant_uv(column, 0, u, v)),
            colours,
        );
        return;
    }
    let row = if stage == 2 { 1 } else { full };
    let uvs = [(0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0)];
    let uvs = uvs.map(|(u, v)| plant_uv(column, row, u, v));
    let up = Vec3::Y * CELL_SIZE;
    for offset in PLANT_ROWS {
        for (normal, along) in [(Vec3::X, Vec3::Z), (Vec3::Z, Vec3::X)] {
            let middle = centre + normal * offset * CELL_SIZE;
            let (a, b) = (middle - along * HALF_CELL, middle + along * HALF_CELL);
            plants.free([a, b, b + up, a + up], normal, uvs, colours);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_has_every_plant_once() {
        let found: std::collections::BTreeSet<_> = PLANT_LIST.into_iter().collect();
        assert_eq!(found.len(), PLANT_LIST.len());
        assert!(found.iter().all(|&(c, r)| c < 10 && (2..5).contains(&r)));
    }

    /// A tile moves on a stage a day, through a plant's four stages in order,
    /// then on to the next plant in the list, and all the way round.
    #[test]
    fn a_tile_moves_on_daily_down_the_list() {
        let at = |hour: u32| tile_growth((5, 9), (1, 2), hour);
        let mut seen = std::collections::BTreeSet::new();
        let mut changes = 0;
        for hour in 0..LIST_HOURS as u32 {
            seen.insert(at(hour));
            let (from, to) = (at(hour), at(hour + 1));
            if from != to {
                changes += 1;
                let next = (from.1 + 1) % STAGES;
                assert_eq!(to.1, next);
                let plant = (from.0 + usize::from(next == 0)) % PLANT_LIST.len();
                assert_eq!(to.0, plant);
            }
        }
        assert_eq!(seen.len(), 30 * 4);
        assert_eq!(changes, 30 * 4);
        // A day later, a tile is always a stage on.
        assert_ne!(at(100), at(124));
    }

    /// In one patch, tiles differ, and don't all change at the same hour.
    #[test]
    fn tiles_grow_at_their_own_times() {
        let cells = (0..12).map(|x| (x, 0));
        let looks: std::collections::BTreeSet<_> =
            cells.clone().map(|c| tile_growth((1, 1), c, 500)).collect();
        assert!(looks.len() > 6);
        for hour in 0..24 {
            let moved = cells
                .clone()
                .filter(|&c| tile_growth((1, 1), c, hour) != tile_growth((1, 1), c, hour + 1))
                .count();
            assert!(moved < 12, "all at hour {hour}");
        }
    }

    #[test]
    fn a_shift_is_whole_stages() {
        assert_eq!(growth(5, 2), 53);
    }

    #[test]
    fn stages_draw_nothing_a_flat_surface_or_a_hash() {
        let mut plants = Builder::default();
        tile(&mut plants, Vec3::ZERO, (0, 2), 0);
        assert!(plants.indices.is_empty());
        tile(&mut plants, Vec3::ZERO, (0, 2), 1);
        assert_eq!(plants.indices.len(), 6);
        let mut plants = Builder::default();
        tile(&mut plants, Vec3::ZERO, (0, 2), 3);
        assert_eq!(plants.indices.len(), 4 * 6);
    }

    #[test]
    fn only_tiles_in_sight_are_drawn() {
        let mut level = Level::new(0, 0, 40, 1);
        for x in 0..40 {
            level.set(x, 0, Cell::HerbBed);
        }
        let none = Sight::new(&level);
        assert!(
            build_plants(&level, &none, &none, (0, 0), 3)
                .indices
                .is_empty()
        );
        let mut seen = Sight::new(&level);
        for x in 0..40 {
            seen.insert(x, 0);
        }
        assert!(
            !build_plants(&level, &seen, &seen, (0, 0), 3)
                .indices
                .is_empty()
        );
    }
}
