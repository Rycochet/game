use super::*;
use crate::movement::{WorldConditions, world_step_cost};
use crate::world::{WorldParams, WorldSeed};

fn ground() -> (WorldParams, WorldGround) {
    let world = WorldParams::from_seed(WorldSeed::new(12345).terrain());
    let ground = WorldGround::new(&world);
    (world, ground)
}

/// A dry point with a good slope east of it, for climbing.
fn hillside(ground: &WorldGround, world: &WorldParams) -> (i64, i64) {
    let conditions = WorldConditions {
        water: world.high_water(),
        ice: false,
    };
    for y in (-1500..1500).step_by(8) {
        for x in (-1500..1500).step_by(8) {
            let here = ground.elevation(x, y, conditions);
            let east = ground.elevation(x + 1, y, conditions);
            if let (Some(a), Some(b)) = (here, east)
                && b - a >= 4
            {
                return (x, y);
            }
        }
    }
    panic!("no hillside");
}

#[test]
fn cells_per_point_matches_the_wilderness() {
    assert_eq!(CELLS_PER_POINT, crate::wilderness::SCALE);
}

#[test]
fn heights_change_smoothly_between_points() {
    let (world, ground) = ground();
    let (px, py) = hillside(&ground, &world);
    let (x0, y0) = (px * CELLS_PER_POINT, py * CELLS_PER_POINT);
    let heights = WildHeights::new(&ground, (x0 - 32, y0 - 32, 96, 96));
    let most = 4 * 16 * 16;
    for y in y0 - 16..y0 + 32 {
        for x in x0 - 16..x0 + 32 {
            let step = heights.at(x, y).abs_diff(heights.at(x + 1, y));
            assert!(step <= most / 16, "{step} between ({x}, {y}) and the next");
        }
    }
    // A point's own cells lean towards it: its corner cell is nearly its height.
    let corner = heights.at(x0, y0);
    let point = ground.broad_elevation(px, py) * 16;
    assert!(
        corner.abs_diff(point) <= 16 * 16,
        "{corner} against {point}"
    );
}

#[test]
fn the_sea_is_flat() {
    let (_, ground) = ground();
    let heights = WildHeights::new(&ground, (0, 0, 1, 1));
    assert!(heights.at(0, 0) >= heights.floor);
    let deep = WildHeights {
        min_px: 0,
        min_py: 0,
        columns: 2,
        rows: 2,
        points: vec![-1000; 4],
        floor: -16,
        flats: Vec::new(),
    };
    assert_eq!(deep.at(3, 5), -16);
    assert_eq!(deep.at(7, 9), -16);
    assert_eq!(deep.at(16, 0), 0, "outside the rectangle");
}

#[test]
fn sixteen_cells_cost_what_a_world_map_step_does() {
    // Two points 4 elevations apart: a straight line of 16 cells across them.
    let heights = WildHeights {
        min_px: 0,
        min_py: 0,
        columns: 3,
        rows: 2,
        points: vec![0, 64, 128, 0, 64, 128],
        floor: -1000,
        flats: Vec::new(),
    };
    let walked: u32 = (8..24)
        .map(|x| {
            let ends = (heights.at(x, 0), heights.at(x + 1, 0));
            wild_step_cost((x, 0), (x + 1, 0), ends, Gait::UNSKILLED)
        })
        .sum();
    let world = world_step_cost(
        (0, 0),
        (1, 0),
        (Some(0), Some(4)),
        None,
        false,
        Gait::UNSKILLED,
    );
    assert_eq!(walked, 16 * world);
}

#[test]
fn flat_ground_is_level_at_its_middle_height() {
    let (world, ground) = ground();
    let (px, py) = hillside(&ground, &world);
    let (x0, y0) = (px * CELLS_PER_POINT, py * CELLS_PER_POINT);
    let flat = FlatGround::of_cutout(&ground, (x0, y0, 20, 10));
    let sloped = WildHeights::new(&ground, (x0 - 4, y0 - 4, 40, 40));
    assert_eq!(flat.height, sloped.at(x0 + 10, y0 + 5));
    let heights = sloped.clone().with_flats(vec![flat]);
    for y in y0..y0 + 10 {
        for x in x0..x0 + 20 {
            assert_eq!(heights.at(x, y), flat.height);
        }
    }
    // Outside it, the slopes as before.
    assert_eq!(heights.at(x0 + 25, y0), sloped.at(x0 + 25, y0));
    assert_ne!(
        sloped.at(x0, y0),
        sloped.at(x0 + 19, y0),
        "a slope to level"
    );
}
