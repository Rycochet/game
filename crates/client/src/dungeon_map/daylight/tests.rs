use shared::dungeon::{Cell, Level};
use shared::protocol::SeenStairs;
use shared::rng::Seed;
use shared::site::BuildingLook;
use shared::site::{StairsDirection, StairsStyle};
use std::collections::BTreeSet;

use super::*;

/// `#` wall, `.` floor, `,` ground, `W` window, `+` door.
fn level(rows: &[&str]) -> Level {
    let mut level = Level::new(0, 0, rows[0].len() as i64, rows.len() as i64);
    for (y, row) in rows.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let cell = match c {
                '#' => Cell::Wall,
                '.' => Cell::Floor,
                ',' => Cell::Ground,
                'W' => Cell::Window,
                '+' => Cell::Door,
                _ => panic!("{c}"),
            };
            level.set(x as i64, y as i64, cell);
        }
    }
    level
}

const ROOM: [&str; 5] = [
    ",,###########",
    ",,#.........#",
    ",,W.........#",
    ",,#.........#",
    ",,###########",
];

fn spilling(level: &Level, travel: Vec2) -> Spill {
    Spill::work_out(level, &[], travel, false)
}

#[test]
fn light_comes_in_through_a_window_and_fades() {
    let level = level(&ROOM);
    let spill = spilling(&level, Vec2::ZERO);
    let along: Vec<f32> = (2..9).map(|x| spill.at(x, 2)).collect();
    assert!(along[0] > 0.4, "{along:?}");
    assert!(along.windows(2).all(|pair| pair[0] >= pair[1]), "{along:?}");
    assert_eq!(spill.at(8, 2), 0.0);
}

#[test]
fn more_open_cells_are_no_brighter() {
    let wide = level(&[
        ",,,,,,,,,",
        ",,,,,,,,,",
        "####W####",
        "#.......#",
        "#.......#",
        "#########",
    ]);
    // From the window's own column, whatever else reaches it, no more than one.
    let amount = spilling(&wide, Vec2::ZERO).at(4, 3);
    assert!(amount <= SPILL_STRENGTH, "{amount}");
}

#[test]
fn walls_stop_it() {
    let shut = level(&[",,#.....", ",,#.....", ",,#....."]);
    let spill = spilling(&shut, Vec2::ZERO);
    assert_eq!(spill.at(3, 1), 0.0);
}

#[test]
fn a_low_light_stretches_it_along_its_way() {
    let level = level(&ROOM);
    let going_in = spilling(&level, Vec2::X);
    let going_out = spilling(&level, Vec2::NEG_X);
    let flat = spilling(&level, Vec2::ZERO);
    assert!(going_in.at(7, 2) > flat.at(7, 2));
    assert!(going_in.at(5, 2) > going_out.at(5, 2));
}

#[test]
fn stairs_open_to_the_sky_light_their_surroundings() {
    let cave = level(&["#####", "#...#", "#...#", "#...#", "#####"]);
    let stair = |open_to_sky| SeenStairs {
        open_to_sky,
        x: 2,
        y: 2,
        direction: StairsDirection::Up,
        descends: (0, 1),
        style: StairsStyle::Spiral,
    };
    let lit = Spill::work_out(&cave, &[stair(true)], Vec2::ZERO, false);
    assert_eq!(lit.at(2, 2), SPILL_STRENGTH);
    assert!(lit.at(1, 1) > 0.0);
    let shut = Spill::work_out(&cave, &[stair(false)], Vec2::ZERO, false);
    assert_eq!(shut.at(2, 2), 0.0);
}

#[test]
fn covered_ground_has_only_what_spills_under_it() {
    let hut = level(&[",,,,,,,,", ",,,,,,,,", ",,,,,,,,"]);
    let mut hut = hut;
    for y in 0..3 {
        for x in 2..8 {
            hut.set(x, y, Cell::CoveredGround);
        }
    }
    let spill = spilling(&hut, Vec2::ZERO);
    let light = Vec3::ONE;
    let grid = light_from_outside(&hut, &spill, light, false, false);
    assert_eq!(grid.at(0, 1), light);
    let edge = grid.at(2, 1).x;
    assert!(edge > 0.0 && edge < 1.0);
    assert!(grid.at(7, 1).x < edge);
}

#[test]
fn water_is_only_under_the_sky_in_open_air_places() {
    let pool = level(&["...", "...", "..."]);
    let mut pool = pool;
    pool.set(1, 1, Cell::Water);
    let spill = Spill::default();
    let inside = light_from_outside(&pool, &spill, Vec3::ONE, false, false);
    assert_eq!(inside.at(1, 1), Vec3::ZERO);
    let open = light_from_outside(&pool, &spill, Vec3::ONE, false, true);
    assert_eq!(open.at(1, 1), Vec3::ONE);
}

fn building(id: u16, window: (i64, i64)) -> BuildingLook {
    use shared::site::{BuildingKind, RoofKind, WallKind};
    BuildingLook {
        id,
        kind: BuildingKind::House,
        walls: WallKind::ALL[0],
        roof: RoofKind::ALL[0],
        rects: vec![shared::site::Rect {
            x: window.0,
            y: window.1,
            width: 1,
            height: 1,
        }],
        door: (0, 0),
        faces: (0, 1),
    }
}

#[test]
fn a_lit_window_lights_the_cells_before_it_and_only_if_on() {
    let mut street = level(&[
        ",,,,,,,,,",
        ",,,,,,,,,",
        ",,,,,,,,,",
        ",,,,,,,,,",
        "#########",
    ]);
    street.set(4, 4, Cell::BlindWindow);
    let buildings = [building(7, (4, 4))];
    let on = BTreeSet::from([7]);
    let glow = Spill::from_windows(&street, &buildings, &on);
    assert_eq!(glow.at(4, 3), SPILL_STRENGTH);
    assert!(glow.at(4, 1) > 0.0);
    assert_eq!(glow.at(8, 0), 0.0);
    let dark = Spill::from_windows(&street, &buildings, &BTreeSet::new());
    assert_eq!(dark.at(4, 3), 0.0);
}

#[test]
fn lights_come_on_over_about_an_hour() {
    let seed = Seed::new(5);
    let delays: BTreeSet<u64> = (0..100).map(|id| lights_delay(seed, (3, 4), id)).collect();
    assert!(delays.iter().all(|&delay| delay < LIGHTS_SPREAD_MINUTES));
    assert!(delays.len() > 30, "{delays:?}");
    // The same for everyone, and different in another town.
    assert_eq!(lights_delay(seed, (3, 4), 9), lights_delay(seed, (3, 4), 9));
    let elsewhere =
        (0..100).filter(|&id| lights_delay(seed, (3, 4), id) != lights_delay(seed, (9, 4), id));
    assert!(elsewhere.count() > 50);
}

#[test]
fn a_town_has_it_all_over() {
    let town = level(&["#.,"]);
    let grid = light_from_outside(&town, &Spill::default(), Vec3::splat(0.5), true, false);
    assert_eq!(grid.at(0, 0), Vec3::splat(0.5));
    assert_eq!(grid.at(1, 0), Vec3::splat(0.5));
}
