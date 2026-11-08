use super::*;
use shared::movement::{WildHeights, WorldGround};
use shared::world::{WorldParams, WorldSeed};

/// A steep mountainside of the default world (8 elevations a world map
/// point), as the client keeps its heights.
fn mountain() -> (WildCellHeights, (i64, i64)) {
    let world = WorldParams::from_seed(WorldSeed::new(12345).terrain());
    let ground = WorldGround::new(&world);
    let middle = (-212 * 16, 1036 * 16);
    let bounds = (middle.0 - 32, middle.1 - 32, 64, 64);
    let heights = WildCellHeights {
        bounds,
        heights: Some(WildHeights::new(&ground, bounds)),
        ..default()
    };
    (heights, middle)
}

fn middle_of(ground: Ground, (x, y): (i64, i64)) -> Vec3 {
    Vec3::new(x as f32 * CELL_SIZE, ground.at(x, y), y as f32 * CELL_SIZE)
}

#[test]
fn the_mountain_is_steep() {
    let (heights, (x, y)) = mountain();
    let ground = Ground(Some(&heights));
    let rise = (ground.at(x, y) - ground.at(x, y + 8)).abs()
        + (ground.at(x, y) - ground.at(x + 8, y)).abs();
    assert!(rise > CELL_SIZE, "rises {rise} over 8 cells");
}

#[test]
fn rays_land_on_the_cell_aimed_at() {
    let (heights, (mx, my)) = mountain();
    let ground = Ground(Some(&heights));
    // From high above, from each side, at the middle of many cells.
    for (dx, dz) in [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.0), (0.6, 0.8)] {
        for y in my - 10..my + 10 {
            for x in mx - 10..mx + 10 {
                let target = middle_of(ground, (x, y));
                let from = target + Vec3::new(dx, 3.0, dz).normalize() * 2000.0;
                let ray = Ray3d::new(from, Dir3::new(target - from).unwrap());
                let (_, point, cell) = ground.hit(ray).expect("reaches the ground");
                assert_eq!(cell, (x, y), "aimed at {:?}, landed at {point}", (x, y));
            }
        }
    }
}

#[test]
fn a_ray_into_a_step_lands_on_the_higher_cell() {
    let (heights, (mx, my)) = mountain();
    let ground = Ground(Some(&heights));
    let mut tried = 0;
    for y in my - 10..my + 10 {
        for x in mx - 10..mx + 10 {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let foot = ground.at(x + dx, y + dy);
                if ground.at(x, y) - foot < CELL_SIZE * 0.2 {
                    continue;
                }
                // Low down, from the lower cell's side, at the face between.
                let out = Vec3::new(dx as f32, 0.0, dy as f32);
                let face = Vec3::new(x as f32, 0.0, y as f32) * CELL_SIZE
                    + out * (CELL_SIZE / 2.0)
                    + Vec3::Y * (foot + 1.0);
                let from = face + (out + Vec3::Y * 0.02).normalize() * 20.0;
                let ray = Ray3d::new(from, Dir3::new(face - from).unwrap());
                assert_eq!(ground.hit(ray).map(|(_, _, cell)| cell), Some((x, y)));
                tried += 1;
            }
        }
    }
    assert!(tried > 10, "only {tried} steps");
}

#[test]
fn flat_ground_is_the_floor() {
    let from = Vec3::new(3.0 * CELL_SIZE, 400.0, 2.0 * CELL_SIZE);
    let ray = Ray3d::new(from, Dir3::NEG_Y);
    let (distance, _, cell) = Ground::FLAT.hit(ray).unwrap();
    assert_eq!(cell, (3, 2));
    assert!((distance - 400.0).abs() < 1e-3);
}

#[test]
fn no_heights_is_the_floor_too() {
    // What clicks pass everywhere but the wilderness: the resource, empty.
    let none = WildCellHeights::default();
    let ground = Ground(Some(&none));
    let from = Vec3::new(3.0 * CELL_SIZE, 400.0, 2.0 * CELL_SIZE);
    let (distance, _, cell) = ground.hit(Ray3d::new(from, Dir3::NEG_Y)).unwrap();
    assert_eq!(cell, (3, 2));
    assert!((distance - 400.0).abs() < 1e-3);
    // Slanted, as from the camera.
    let target = Vec3::new(-5.0 * CELL_SIZE, 0.0, 7.0 * CELL_SIZE);
    let camera = target + Vec3::new(0.0, 600.0, 800.0);
    let ray = Ray3d::new(camera, Dir3::new(target - camera).unwrap());
    assert_eq!(ground.hit(ray).map(|(_, _, cell)| cell), Some((-5, 7)));
}
