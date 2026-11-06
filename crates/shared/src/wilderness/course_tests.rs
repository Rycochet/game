use super::*;
use crate::dungeon::{Cell, Level};
use crate::poi::ChunkPos;
use crate::road::{Road, RoadKind, RoadLink};

fn world() -> WorldParams {
    WorldParams::from_seed(Seed::new(12345))
}

/// Every block of a chunk's middle, made with `plan`.
fn middle_blocks(wild: &Wilderness, plan: &ChunkPlan) -> Vec<Level> {
    let (min_x, min_y, width, height) = wild.chunk_bounds(plan.chunk());
    let first = wild.block_of(min_x + width / 4, min_y + height / 4);
    let last = wild.block_of(min_x + 3 * width / 4, min_y + 3 * height / 4);
    (first.y..=last.y)
        .flat_map(|y| (first.x..=last.x).map(move |x| BlockPos { x, y }))
        .map(|block| wild.block(block, plan))
        .collect()
}

fn count(levels: &[Level], wanted: impl Fn(Cell) -> bool) -> usize {
    levels
        .iter()
        .map(|level| {
            let (min_x, min_y, width, height) = level.bounds();
            (min_y..min_y + height)
                .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
                .filter(|&(x, y)| wanted(level.cell(x, y)))
                .count()
        })
        .sum()
}

#[test]
fn rivers_run_through_the_land_as_water_with_shallow_banks() {
    let wild = Wilderness::new(&world());
    // A chunk inland with a river through its middle.
    let plan = (0..400)
        .map(|n| ChunkPos {
            x: n % 40,
            y: n / 40 - 5,
        })
        .map(|chunk| ChunkPlan::new(&wild, chunk, None, &[], &[]))
        .find(|plan| {
            let (x, y, width, height) = wild.chunk_bounds(plan.chunk());
            let middle = (x + width / 4, y + height / 4, width / 2, height / 2);
            let (px, py) = wild.point_of(x + width / 2, y + height / 2);
            plan.rivers.iter().any(|river| river.near(middle, 0.0))
                && wild.sample_point(px, py).height > wild.low_water + 2.0
        })
        .expect("a river inland");
    let blocks = middle_blocks(&wild, &plan);
    assert!(count(&blocks, |cell| cell == Cell::Shallows) > 50);
    // No tree stands in or right beside a river.
    for level in &blocks {
        let (min_x, min_y, width, height) = level.bounds();
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                let nearest = plan
                    .rivers
                    .iter()
                    .map(|r| r.outside(x, y))
                    .fold(f64::MAX, f64::min);
                if nearest <= RIVER_CLEARED {
                    assert_ne!(level.cell(x, y), Cell::Tree, "({x}, {y})");
                }
            }
        }
    }
}

#[test]
fn roads_are_surfaced_by_kind_and_bridge_water() {
    let wild = Wilderness::new(&world());
    // Across a chunk with a river, east to west through its middle.
    let plan = (0..400)
        .map(|n| ChunkPos {
            x: n % 40,
            y: n / 40 - 5,
        })
        .map(|chunk| ChunkPlan::new(&wild, chunk, None, &[], &[]))
        .find(|plan| plan.has_rivers() && plan.cutout().is_none())
        .expect("a river");
    let chunk = plan.chunk();
    let (x, y, width, height) = wild.chunk_bounds(chunk);
    let river = plan
        .rivers
        .iter()
        .find(|r| r.near((x, y, width, height), 0.0));
    let river = river.expect("a river in the chunk");
    // A road straight through the river's first cell inside the chunk.
    let (cx, cy) = (x..x + width)
        .flat_map(|cx| (y..y + height).map(move |cy| (cx, cy)))
        .find(|&(cx, cy)| river.outside(cx, cy) <= 0.0)
        .expect("a river cell");
    let (px, py) = wild.point_of(cx, cy);
    let road = Road {
        link: RoadLink::new(
            chunk,
            ChunkPos {
                x: chunk.x + 1,
                ..chunk
            },
        ),
        kind: RoadKind::Road,
        points: (px - 6..=px + 6).map(|px| (px, py)).collect(),
    };
    let plan = ChunkPlan::new(&wild, chunk, None, &[], &[&road]);
    assert!(plan.has_roads());
    let block = wild.block(wild.block_of(cx, cy), &plan);
    let mut level = vec![block];
    let (bx, by, _, _) = level[0].bounds();
    for dx in [-1, 1] {
        level.push(wild.block(wild.block_of(bx + dx * BLOCK_SIZE, by), &plan));
    }
    assert!(count(&level, |cell| cell == Cell::Cobblestone) > 20);
    assert!(count(&level, |cell| cell == Cell::Bridge) > 0);
}
