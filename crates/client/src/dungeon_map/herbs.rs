//! What grows in a herbalist's patch: only looks, chosen here, the same for
//! everyone since it comes from the server's clock and the place.
//!
//! The ground is always the land's own (`Cell::HerbBed` is drawn as outside
//! ground). Every two game days everything moves on a stage: bare, dots, stalks,
//! then full grown; after that a new cycle gives each tile a new plant, herb or
//! mushroom, of a new shape and colour. Plants have a mesh of their own, which
//! only changes with what is in sight and the stage.

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
const SHAPES: u64 = 5;
const COLOURS: u64 = 3;
const COLUMNS: f32 = 10.0;
const ROWS: f32 = 5.0;

/// Game hours between stages, and how many stages there are.
const HOURS_PER_STAGE: u32 = 48;
const STAGES: u32 = 4;

const HALF_CELL: f32 = CELL_SIZE / 2.0;

/// How far dots lie above the ground, to stay in front of it.
const DOTS_LIFT: f32 = 0.05;

/// For choosing what grows on a tile.
const PLANTS: Seed = Seed::new(0x68_6572_6273);

/// How far everything has grown: stages since the world began.
pub fn growth(hour: u32, shift: u32) -> u32 {
    (hour / HOURS_PER_STAGE).wrapping_add(shift)
}

/// The picture's column and full-grown row of what grows on a tile in a cycle,
/// at a place on the world map.
fn plant(place: (i64, i64), (x, y): (i64, i64), cycle: u32) -> (u64, u64) {
    let hash = PLANTS.at(&[place.0, place.1, x, y, i64::from(cycle)]);
    let mushroom = hash & 1;
    let shape = (hash >> 8) % SHAPES;
    let colour = (hash >> 16) % COLOURS;
    (mushroom * SHAPES + shape, 2 + colour)
}

/// A tile of the plants' picture, at `(u, v)` within it.
fn plant_uv(column: u64, row: u64, u: f32, v: f32) -> [f32; 2] {
    [(column as f32 + u) / COLUMNS, (row as f32 + v) / ROWS]
}

/// Everything growing where anyone can see it. Nothing is drawn of it from
/// memory: it will have grown since.
pub fn build_plants(
    level: &Level,
    sight: &Sight,
    remembered: &Sight,
    place: (i64, i64),
    grown: u32,
) -> Builder {
    let mut plants = Builder::default();
    let (stage, cycle) = (grown % STAGES, grown / STAGES);
    if stage == 0 {
        return plants;
    }
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            let here = level.cell(x, y) == Cell::HerbBed
                && sight.is_seen(x, y)
                && remembered.is_seen(x, y);
            if here {
                let (column, full) = plant(place, (x, y), cycle);
                tile(&mut plants, cell_centre(x, y), column, stage, full);
            }
        }
    }
    plants
}

/// One tile's plants: dots flat on the ground, the rest upright, crossed.
fn tile(plants: &mut Builder, centre: Vec3, column: u64, stage: u32, full: u64) {
    let colours = [Vec3::ONE; 4];
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
    for (normal, along) in [(Vec3::X, Vec3::Z), (Vec3::Z, Vec3::X)] {
        let (a, b) = (centre - along * HALF_CELL, centre + along * HALF_CELL);
        plants.free([a, b, b + up, a + up], normal, uvs, colours);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two game days a stage, starting bare; four stages to a cycle.
    #[test]
    fn plants_move_on_every_two_days() {
        assert_eq!(growth(47, 0), 0);
        assert_eq!(growth(48, 0), 1);
        assert_eq!(growth(4 * 48, 0) / STAGES, 1);
        assert_eq!(growth(4 * 48, 0) % STAGES, 0);
        assert_eq!(growth(0, 3), 3);
    }

    /// Herbs and mushrooms in every shape and colour turn up, on one tile over
    /// the cycles, and it can change from one to the next.
    #[test]
    fn each_cycle_can_grow_something_else() {
        let looks: std::collections::BTreeSet<_> = (0..2000)
            .map(|cycle| plant((5, 9), (1, 2), cycle))
            .collect();
        assert_eq!(looks.len(), 10 * 3);
        assert_ne!(
            (0..8).map(|c| plant((5, 9), (1, 2), c)).collect::<Vec<_>>(),
            vec![plant((5, 9), (1, 2), 0); 8]
        );
        for (column, row) in looks {
            assert!(column < 10 && (2..5).contains(&row));
        }
    }

    #[test]
    fn nothing_is_drawn_while_bare_or_unseen() {
        let mut level = Level::new(0, 0, 2, 1);
        level.set(0, 0, Cell::HerbBed);
        level.set(1, 0, Cell::HerbBed);
        let mut seen = Sight::new(&level);
        seen.insert(0, 0);
        let none = Sight::new(&level);
        assert!(
            build_plants(&level, &seen, &seen, (0, 0), 0)
                .indices
                .is_empty()
        );
        assert!(
            build_plants(&level, &none, &none, (0, 0), 3)
                .indices
                .is_empty()
        );
        // One tile: flat dots are one surface, crossed stalks two.
        assert_eq!(
            build_plants(&level, &seen, &seen, (0, 0), 1).indices.len(),
            6
        );
        assert_eq!(
            build_plants(&level, &seen, &seen, (0, 0), 2).indices.len(),
            12
        );
    }
}
