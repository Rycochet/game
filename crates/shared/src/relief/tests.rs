use super::*;
use crate::dungeon::{Cell, Level, RoomId};
use crate::movement::WorldGround;
use crate::poi::{ChunkPos, PoiMap};
use crate::rng::Seed;
use crate::site::{Generator, Site};
use crate::wilderness::{ChunkPlan, Cutout, Wilderness};
use crate::world::WorldParams;

fn sloping() -> Relief {
    // A plane rising east, with a bump at corner (2, 1).
    Relief::from_fn((0, 0, 4, 3), |x, y| {
        (x * 40 + y * 10) as i32 + if (x, y) == (2, 1) { 90 } else { 0 }
    })
}

#[test]
fn corners_and_middles_are_where_the_surface_is() {
    let relief = sloping();
    let cell = i64::from(UNITS_PER_CELL);
    for y in 0..3 {
        for x in 0..4 {
            let (ux, uy) = (x * cell, y * cell);
            assert_eq!(relief.at(ux, uy), relief.centre(x, y));
            assert_eq!(relief.at(ux - HALF, uy - HALF), relief.corner(x, y));
        }
    }
}

#[test]
fn the_surface_has_no_steps() {
    let relief = sloping();
    // Along rows and columns a unit at a time, no jump bigger than the
    // steepest triangle allows.
    let most = 3;
    for uy in -HALF..3 * 2 * HALF - HALF {
        for ux in -HALF..4 * 2 * HALF - HALF - 1 {
            let step = relief.at(ux, uy).abs_diff(relief.at(ux + 1, uy));
            assert!(step <= most, "({ux}, {uy}): {step}");
            let step = relief.at(ux, uy).abs_diff(relief.at(ux, uy + 1));
            assert!(step <= most, "({ux}, {uy}) down: {step}");
        }
    }
}

#[test]
fn stairs_are_level() {
    let mut relief = sloping();
    relief.level_stairs(&[(2, 1)]);
    let [nw, ne, sw, se] = relief.corners_of(2, 1);
    assert!(
        nw == ne && ne == sw && sw == se,
        "{:?}",
        relief.corners_of(2, 1)
    );
}

#[test]
fn a_cave_swells_gently_and_lies_flat_round_pools() {
    let mut level = Level::new(0, 0, 40, 40);
    for y in 0..40 {
        for x in 0..40 {
            level.set(x, y, Cell::Floor);
        }
    }
    level.set(10, 10, Cell::Water);
    for y in 20..30 {
        for x in 20..30 {
            level.set_room(x, y, Some(RoomId(1)));
        }
    }
    let ground = LevelGround::Cave { seed: 77 };
    let world = WorldGround::new(&WorldParams::from_seed(Seed::new(1)));
    let relief = ground.relief(&world, &level, &[]).expect("a cave swells");
    let mut highest = 0;
    for y in 0..=40 {
        for x in 0..=40 {
            let height = relief.corner(x, y);
            assert!(height.abs() <= CAVE_SWELL, "({x}, {y}): {height}");
            highest = highest.max(height.abs());
        }
    }
    assert!(highest > CAVE_SWELL / 2, "it swells: {highest}");
    assert_eq!(relief.corners_of(10, 10), [0; 4], "the pool is level");
    for y in 21..30 {
        for x in 21..30 {
            assert!(
                relief.corner(x, y).abs() <= ROOM_SWELL,
                "({x}, {y}) in a cave"
            );
        }
    }
}

/// Every cutout set in whole near the middle of a world, with its plan's
/// chunk, until `count` are found.
fn cutouts(count: usize) -> (Wilderness, WorldGround, Vec<(Cutout, ChunkPlan)>) {
    let params = WorldParams::from_seed(Seed::new(12345));
    let wild = Wilderness::new(&params);
    let pois = PoiMap::new(&params, Seed::new(999));
    let ground = WorldGround::new(&params);
    let found = (0..60)
        .flat_map(|x| (-30..30).map(move |y| ChunkPos { x, y }))
        .map(|chunk| ChunkPlan::with_new_site(&wild, &pois, chunk))
        .filter_map(|plan| {
            let cutout = plan.cutout().filter(|c| c.offset().is_some())?.clone();
            Some((cutout, plan))
        })
        .take(count)
        .collect();
    (wild, ground, found)
}

/// The wilderness's cells over `bounds` and a cell round them, from the
/// blocks they are in (all in `plan`'s chunk).
fn wild_cells(wild: &Wilderness, plan: &ChunkPlan, (x, y, w, h): (i64, i64, i64, i64)) -> Level {
    let mut level = Level::new(x - 1, y - 1, w + 2, h + 2);
    let mut blocks = std::collections::BTreeMap::new();
    for cy in y - 1..y + h + 1 {
        for cx in x - 1..x + w + 1 {
            let block = wild.block_of(cx, cy);
            let cells = blocks
                .entry((block.x, block.y))
                .or_insert_with(|| wild.block(block, plan));
            level.set(cx, cy, cells.cell(cx, cy));
        }
    }
    level
}

#[test]
fn a_sites_first_level_has_its_cutouts_slopes() {
    let (wild, ground, found) = cutouts(12);
    assert!(found.len() >= 6, "found {}", found.len());
    let mut sloping = 0;
    for (cutout, plan) in &found {
        let poi = *cutout.poi();
        let generator = Generator::for_site(poi.kind, poi.seed, poi.desert);
        let site = Site::with_generator(poi.kind, poi.seed, generator.version(), generator)
            .expect("the current version runs");
        let first = site.level(0);
        let stairs: Vec<_> = first.stairs.iter().map(|s| (s.x, s.y)).collect();
        let as_site = LevelGround::of_site(&wild, &ground, (poi, &site), 0, &first);
        let site_relief = as_site
            .relief(&ground, &first.level, &stairs)
            .expect("set in whole: the land's");
        let offset = cutout.offset().expect("set in whole");

        // The wilderness round it, from its blocks, with the cutout's flats.
        let block = wild_cells(&wild, plan, cutout.level().bounds());
        let heights =
            WildHeights::new(&ground, block.bounds()).with_flats(&cutout.flats(&ground), &block);
        let wild_stairs: Vec<_> = cutout.stairs().iter().map(|s| (s.x, s.y)).collect();
        let wild_relief = Relief::of_land(&heights, block.bounds(), (0, 0), &wild_stairs);

        let (min_x, min_y, width, height) = first.level.bounds();
        let (mut low, mut high) = (i32::MAX, i32::MIN);
        for y in min_y..=min_y + height {
            for x in min_x..=min_x + width {
                let here = site_relief.corner(x, y);
                let there = wild_relief.corner(x + offset.0, y + offset.1);
                assert_eq!(here, there, "{:?} corner ({x}, {y})", poi.kind);
                (low, high) = (low.min(here), high.max(here));
            }
        }
        if high > low {
            sloping += 1;
        }
    }
    assert!(sloping > 0, "some of them slope");
}

#[test]
fn a_building_is_flat() {
    let site = Site::new(crate::poi::PoiKind::Castle, Seed::new(5));
    assert!(!site.is_cave());
}
