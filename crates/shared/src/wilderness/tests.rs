use std::collections::VecDeque;

use super::*;
use crate::dungeon::{Cell, Level};
use crate::poi::{ChunkPos, PoiMap};
use crate::site::Exit;

fn world() -> WorldParams {
    WorldParams::from_seed(Seed::new(12345))
}

fn pois(world: &WorldParams) -> PoiMap {
    PoiMap::new(world, Seed::new(999))
}

/// The plan of the chunk a cell is in, with no poi.
fn bare_plan(wild: &Wilderness, x: i64, y: i64) -> ChunkPlan {
    ChunkPlan::new(wild, wild.chunk_of(x, y), None)
}

/// The first point in a stretch of the planet that `wanted` picks, by its
/// sample and its neighbours' heights.
fn find_point(wild: &Wilderness, wanted: impl Fn(&Wilderness, i64, i64) -> bool) -> (i64, i64) {
    (-300..300)
        .step_by(7)
        .flat_map(|y| (0..wild.points()).step_by(13).map(move |x| (x, y)))
        .find(|&(x, y)| wanted(wild, x, y))
        .expect("the test world has such a place")
}

fn count(level: &Level, cell: Cell) -> usize {
    let (min_x, min_y, width, height) = level.bounds();
    (min_y..min_y + height)
        .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
        .filter(|&(x, y)| level.cell(x, y) == cell)
        .count()
}

/// A number summing up a level's cells, for golden tests.
fn fingerprint(level: &Level) -> u64 {
    let (min_x, min_y, width, height) = level.bounds();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            hash = (hash ^ level.cell(x, y) as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

#[test]
fn cells_and_points_agree() {
    let wild = Wilderness::new(&world());
    for (px, py) in [(0, 0), (5, -3), (wild.points() - 1, 40)] {
        let (x, y) = wild.centre_of(px, py);
        assert_eq!(wild.point_of(x, y), (px, py));
        assert_eq!(wild.point_of(x + wild.width(), y), (px, py));
    }
    assert_eq!(wild.point_of(-1, -1), (wild.points() - 1, -1));
}

#[test]
fn blocks_tile_the_planet_one_chunk_each() {
    let wild = Wilderness::new(&world());
    let mut across = 0;
    for x in 0..wild.block_columns() {
        let (min_x, min_y, width, height) = wild.block_bounds(BlockPos { x, y: -3 });
        assert_eq!(min_x, across);
        assert!(width > 0 && width % SCALE == 0);
        assert_eq!(
            wild.chunk_of(min_x, min_y),
            wild.chunk_of(min_x + width - 1, min_y + height - 1)
        );
        across += width;
    }
    assert_eq!(across, wild.width());
    let chunks: i64 = (0..wild.columns())
        .map(|x| wild.chunk_bounds(ChunkPos { x, y: 0 }).2)
        .sum();
    assert_eq!(chunks, wild.width());
    assert_eq!(wild.block_of(-1, 0).x, wild.block_columns() - 1);
}

#[test]
fn cells_do_not_depend_on_the_block_made() {
    let wild = Wilderness::new(&world());
    // A coast, so water and shallows are compared too.
    let (px, py) = find_point(&wild, |wild, x, y| {
        let here = wild.sample_point(x, y).height;
        let east = wild.sample_point(x + 1, y).height;
        here >= wild.low_water && east < wild.low_water
    });
    let (x, y) = wild.centre_of(px, py);
    let block = wild.block_of(x, y);
    let plan = bare_plan(&wild, x, y);
    let made = wild.block(block, &plan);
    let (min_x, min_y, _, _) = made.bounds();
    let chunk = wild.chunk_bounds(plan.chunk());
    // The same cells, as part of a rectangle inside the chunk lying across it.
    let rect_x = (min_x - 20).max(chunk.0);
    let rect_y = (min_y + 30).min(chunk.1 + chunk.3 - 70);
    let other = wild.cells((rect_x, rect_y, 70, 70), &plan);
    let mut compared = 0;
    for y in rect_y..rect_y + 70 {
        for x in rect_x..rect_x + 70 {
            if made.contains(x, y) {
                assert_eq!(made.cell(x, y), other.cell(x, y), "({x}, {y})");
                compared += 1;
            }
        }
    }
    assert!(compared > 500, "{compared}");
    assert!(count(&made, Cell::Shallows) > 0 || count(&made, Cell::Water) > 0);
}

#[test]
fn shallows_reach_land_and_deep_water_does_not() {
    let wild = Wilderness::new(&world());
    let (px, py) = find_point(&wild, |wild, x, y| {
        let here = wild.sample_point(x, y).height;
        here >= wild.low_water
            && (2..=5).all(|d| wild.sample_point(x + d, y).height < wild.low_water)
    });
    let (x, y) = wild.centre_of(px, py);
    let margin = SHALLOWS_REACH;
    let plan = bare_plan(&wild, x, y);
    let chunk = wild.chunk_bounds(plan.chunk());
    let min_x = (x - 40).max(chunk.0);
    let min_y = (y - 40).max(chunk.1);
    let level = wild.cells((min_x, min_y, 96, 96), &plan);
    let land = |x: i64, y: i64| !matches!(level.cell(x, y), Cell::Water | Cell::Shallows);
    let near_land = |x: i64, y: i64| {
        (-margin..=margin).any(|dy| {
            (-margin..=margin)
                .any(|dx| dx * dx + dy * dy <= margin * margin && land(x + dx, y + dy))
        })
    };
    let (mut shallow, mut deep) = (0, 0);
    for y in min_y + margin..min_y + 96 - margin {
        for x in min_x + margin..min_x + 96 - margin {
            match level.cell(x, y) {
                Cell::Shallows => {
                    assert!(near_land(x, y), "({x}, {y})");
                    shallow += 1;
                }
                Cell::Water => {
                    assert!(!near_land(x, y), "({x}, {y})");
                    deep += 1;
                }
                _ => {}
            }
        }
    }
    assert!(shallow > 0 && deep > 0, "{shallow} {deep}");
}

#[test]
fn thick_woods_stay_open_to_walk_through() {
    let wild = Wilderness::new(&world());
    let (px, py) = find_point(&wild, |wild, x, y| {
        (0..6).all(|d| wild.sample_point(x + d % 3, y + d / 3).trees > 0.9)
    });
    let (x, y) = wild.centre_of(px, py);
    let plan = bare_plan(&wild, x, y);
    let level = wild.block(wild.block_of(x, y), &plan);
    let cells = (BLOCK_SIZE * BLOCK_SIZE) as usize;
    let trees = count(&level, Cell::Tree);
    assert!(trees * 100 > cells * 25, "too few trees: {trees}");
    assert!(
        trees * 100 <= cells * (MOST_TREES_PERCENT as usize + 3),
        "{trees}"
    );
    // Nearly every walkable cell can be reached from every other.
    let (min_x, min_y, width, height) = level.bounds();
    let walkable: Vec<_> = (min_y..min_y + height)
        .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
        .filter(|&(x, y)| level.is_walkable(x, y))
        .collect();
    let mut reached = vec![false; cells];
    let index = |(x, y): (i64, i64)| ((y - min_y) * width + x - min_x) as usize;
    let mut queue = VecDeque::from([walkable[0]]);
    reached[index(walkable[0])] = true;
    let mut found = 1;
    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ] {
            let next = (x + dx, y + dy);
            if level.is_walkable(next.0, next.1) && !reached[index(next)] {
                reached[index(next)] = true;
                found += 1;
                queue.push_back(next);
            }
        }
    }
    assert!(
        found * 100 >= walkable.len() * 99,
        "{found} of {}",
        walkable.len()
    );
}

#[test]
fn cutouts_sit_on_their_poi_inside_their_chunk() {
    let world = world();
    let (wild, pois) = (Wilderness::new(&world), pois(&world));
    let (mut whole, mut from_above) = (0, 0);
    for cy in -6..6 {
        for cx in 0..20 {
            let chunk = ChunkPos { x: cx, y: cy };
            let plan = ChunkPlan::with_new_site(&wild, &pois, chunk);
            let Some(cutout) = plan.cutout() else {
                continue;
            };
            let poi = cutout.poi();
            let centre = wild.centre_of(poi.x, poi.y);
            assert!(cutout.contains(centre.0, centre.1), "{poi:?}");
            let (min_x, min_y, width, height) = cutout.level().bounds();
            let (chunk_x, chunk_y, chunk_width, chunk_height) = wild.chunk_bounds(chunk);
            assert!(min_x >= chunk_x + CLEARED_TO && min_y >= chunk_y + CLEARED_TO);
            assert!(min_x + width <= chunk_x + chunk_width - CLEARED_TO);
            assert!(min_y + height <= chunk_y + chunk_height - CLEARED_TO);
            match cutout.offset() {
                Some((ox, oy)) => {
                    whole += 1;
                    assert!(ox % 8 == 0 && oy % 8 == 0);
                    assert!(cutout.stairs().iter().all(|s| s.to != Exit::WorldMap));
                }
                None => {
                    from_above += 1;
                    assert_eq!(cutout.stairs().len(), 1);
                    assert_eq!((cutout.stairs()[0].x, cutout.stairs()[0].y), centre);
                }
            }
        }
    }
    assert!(whole > 50 && from_above > 0, "{whole} {from_above}");
}

#[test]
fn a_cutout_is_its_sites_first_level_in_a_clearing() {
    let world = world();
    let (wild, pois) = (Wilderness::new(&world), pois(&world));
    let plan = (0..400)
        .map(|n| ChunkPos {
            x: n % 40,
            y: n / 40 - 5,
        })
        .map(|chunk| ChunkPlan::with_new_site(&wild, &pois, chunk))
        .find(|plan| {
            plan.cutout().is_some_and(|cutout| {
                let (px, py) = (cutout.poi().x, cutout.poi().y);
                cutout.offset().is_some() && wild.sample_point(px, py).trees > 0.6
            })
        })
        .expect("a poi in woods");
    let cutout = plan.cutout().unwrap();
    let poi = cutout.poi();
    let site = crate::site::Site::new(poi.kind, poi.seed);
    let (ox, oy) = cutout.offset().unwrap();
    let (min_x, min_y, width, height) = cutout.level().bounds();
    let blocks: Vec<Level> = [(min_x, min_y), (min_x + width, min_y + height)]
        .iter()
        .flat_map(|&(x, y)| {
            let first = wild.block_of(x - CLEARED_TO, y - CLEARED_TO);
            let last = wild.block_of(x + CLEARED_TO, y + CLEARED_TO);
            (first.y..=last.y).flat_map(move |by| (first.x..=last.x).map(move |bx| (bx, by)))
        })
        .map(|(x, y)| wild.block(BlockPos { x, y }, &plan))
        .collect();
    let first = site.level(0).level;
    for level in &blocks {
        let (bx, by, bw, bh) = level.bounds();
        for y in by..by + bh {
            for x in bx..bx + bw {
                if cutout.contains(x, y) {
                    let generator =
                        crate::site::Generator::for_site(poi.kind, poi.seed, poi.desert);
                    if generator.ruin() == 0 {
                        assert_eq!(level.cell(x, y), first.cell(x - ox, y - oy));
                    }
                } else if cutout.distance_squared(x, y) < CLEARED * CLEARED {
                    assert_ne!(level.cell(x, y), Cell::Tree, "({x}, {y})");
                }
            }
        }
    }
}

#[test]
fn golden_block() {
    let wild = Wilderness::new(&world());
    let (px, py) = find_point(&wild, |wild, x, y| wild.sample_point(x, y).trees > 0.5);
    let (x, y) = wild.centre_of(px, py);
    let level = wild.block(wild.block_of(x, y), &bare_plan(&wild, x, y));
    assert_eq!(
        ((px, py), fingerprint(&level)),
        ((0, -300), 7151180237052611686)
    );
}
