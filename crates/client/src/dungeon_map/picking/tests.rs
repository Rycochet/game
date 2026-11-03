use super::*;
use crate::dungeon_map::WALL_HEIGHT;

fn setup() -> (Level, Sight) {
    let level = Level::test_room();
    let mut sight = Sight::new(&level);
    sight.add_view(&level, 0, 0, None);
    (level, sight)
}

fn ray(origin: Vec3, towards: Vec3) -> Ray3d {
    Ray3d::new(origin, Dir3::new(towards - origin).unwrap())
}

#[test]
fn straight_down_picks_the_cell_below() {
    let (level, sight) = setup();
    let above = Vec3::new(3.0 * CELL_SIZE + 5.0, 500.0, -2.0 * CELL_SIZE);
    let target = above.with_y(0.0);
    assert_eq!(pick(ray(above, target), &level, &sight, &[]), Some((3, -2)));
}

#[test]
fn walls_and_unseen_floors_are_not_picked() {
    let (level, sight) = setup();
    let camera = Vec3::new(0.0, 500.0, 800.0);
    // The wall cell itself.
    assert_eq!(
        pick(
            ray(camera, Vec3::new(-2.0 * CELL_SIZE, 0.0, 0.0)),
            &level,
            &sight,
            &[]
        ),
        None
    );
    // Floor hidden behind the short wall.
    assert_eq!(
        pick(
            ray(camera, Vec3::new(-5.0 * CELL_SIZE, 0.0, 0.0)),
            &level,
            &sight,
            &[]
        ),
        None
    );
}

#[test]
fn clicks_beside_a_wall_mean_the_cell_beside_it() {
    let (level, sight) = setup();
    // East of the short wall's line, in its cell.
    let above = Vec3::new(-2.0 * CELL_SIZE + 6.0, 500.0, 0.0);
    let down = ray(above, above.with_y(0.0));
    assert_eq!(pick(down, &level, &sight, &[]), Some((-1, 0)));
}

#[test]
fn solid_sides_block_clicks() {
    let (level, sight) = setup();
    // Looking from the east, just over the floor, at the short wall's
    // east side: the floor beyond it is not reached.
    let face = WallFace {
        centre: Vec3::new(-1.5 * CELL_SIZE, WALL_HEIGHT / 2.0, 0.0),
        normal: Vec3::X,
        half_across: CELL_SIZE / 2.0,
        half_up: WALL_HEIGHT / 2.0,
    };
    let from = Vec3::new(4.0 * CELL_SIZE, WALL_HEIGHT * 0.9, 0.0);
    let through = ray(from, Vec3::new(-1.5 * CELL_SIZE, WALL_HEIGHT / 2.0, 0.0));
    assert!(hits_face(through.origin, *through.direction, &face).is_some());
    assert_eq!(pick(through, &level, &sight, &[face]), None);
    // A ray landing in front of it is fine.
    let in_front = ray(from, Vec3::new(1.0 * CELL_SIZE, 0.0, 0.0));
    assert_eq!(pick(in_front, &level, &sight, &[face]), Some((1, 0)));
}
