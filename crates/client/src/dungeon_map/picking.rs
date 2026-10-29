//! Turns a mouse click on the dungeon into a walkable cell, by following the
//! camera ray to the floor (or the stairs up standing on it) unless a solid
//! wall side is in the way.

use bevy::prelude::*;
use shared::dungeon::{Level, Sight};

use super::mesh::{STAIRS_UP_HEIGHT, WallFace};
use super::{CELL_SIZE, WALL_HEIGHT};

/// The seen walkable cell the ray first lands on, if no solid wall side is hit
/// first. See-through sides don't block: the floor shows through them.
/// `stairs_up` are cells whose stairs stand up from the floor: the ray lands
/// on them if it hits their block first.
pub fn pick(
    ray: Ray3d,
    level: &Level,
    sight: &Sight,
    solid_faces: &[WallFace],
    stairs_up: &[(i64, i64)],
) -> Option<(i64, i64)> {
    let direction = *ray.direction;
    // Only rays going down reach the floor (at height 0).
    if direction.y >= 0.0 {
        return None;
    }
    let floor_distance = -ray.origin.y / direction.y;
    let point = ray.origin + direction * floor_distance;
    let floor_cell = (
        (point.x / CELL_SIZE).round() as i64,
        (point.z / CELL_SIZE).round() as i64,
    );
    let (distance, (x, y)) = stairs_up
        .iter()
        .filter(|&&(x, y)| sight.is_seen(x, y))
        .filter_map(|&cell| hits_stairs(ray.origin, direction, cell).map(|d| (d, cell)))
        .fold((floor_distance, floor_cell), |best, hit| {
            if hit.0 < best.0 { hit } else { best }
        });
    let blocked = solid_faces
        .iter()
        .any(|face| hits_face(ray.origin, direction, face).is_some_and(|d| d < distance));
    if blocked {
        return None;
    }
    (level.is_walkable(x, y) && sight.is_seen(x, y)).then_some((x, y))
}

/// How far along the ray it enters the block of stairs up on `cell`, if it
/// does (slab test against the box).
fn hits_stairs(origin: Vec3, direction: Vec3, (x, y): (i64, i64)) -> Option<f32> {
    let centre = Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE);
    let half = CELL_SIZE / 2.0;
    let low = centre - Vec3::new(half, 0.0, half);
    let high = centre + Vec3::new(half, STAIRS_UP_HEIGHT, half);
    let mut near = f32::NEG_INFINITY;
    let mut far = f32::INFINITY;
    for axis in 0..3 {
        let (o, d) = (origin[axis], direction[axis]);
        if d.abs() < f32::EPSILON {
            if o < low[axis] || o > high[axis] {
                return None;
            }
            continue;
        }
        let (a, b) = ((low[axis] - o) / d, (high[axis] - o) / d);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    (near <= far && near > 0.0).then_some(near)
}

/// How far along the ray it crosses the wall side, if it does from the front.
fn hits_face(origin: Vec3, direction: Vec3, face: &WallFace) -> Option<f32> {
    let towards = direction.dot(face.normal);
    if towards >= 0.0 {
        return None;
    }
    let distance = (face.centre - origin).dot(face.normal) / towards;
    let offset = origin + direction * distance - face.centre;
    // Sideways along the side, and up it.
    let across = offset.dot(Vec3::Y.cross(face.normal)).abs();
    let up = offset.y.abs();
    (distance > 0.0 && across <= CELL_SIZE / 2.0 && up <= WALL_HEIGHT / 2.0).then_some(distance)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            pick(ray(above, target), &level, &sight, &[], &[]),
            Some((3, -2))
        );
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
                &[],
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
                &[],
                &[]
            ),
            None
        );
    }

    #[test]
    fn solid_sides_block_clicks() {
        let (level, sight) = setup();
        // Looking from the east, just over the floor, at the short wall's
        // east side: the floor beyond it is not reached.
        let face = WallFace {
            centre: Vec3::new(-1.5 * CELL_SIZE, WALL_HEIGHT / 2.0, 0.0),
            normal: Vec3::X,
        };
        let from = Vec3::new(4.0 * CELL_SIZE, WALL_HEIGHT * 0.9, 0.0);
        let through = ray(from, Vec3::new(-1.5 * CELL_SIZE, WALL_HEIGHT / 2.0, 0.0));
        assert!(hits_face(through.origin, *through.direction, &face).is_some());
        assert_eq!(pick(through, &level, &sight, &[face], &[]), None);
        // A ray landing in front of it is fine.
        let in_front = ray(from, Vec3::new(1.0 * CELL_SIZE, 0.0, 0.0));
        assert_eq!(pick(in_front, &level, &sight, &[face], &[]), Some((1, 0)));
    }

    #[test]
    fn stairs_up_catch_rays_before_the_floor_behind() {
        let (level, sight) = setup();
        // A low ray over (2, 0) would land on the floor further west, but the
        // stairs standing on (2, 0) are in the way.
        let from = Vec3::new(6.0 * CELL_SIZE, STAIRS_UP_HEIGHT / 2.0, 0.0);
        let low = ray(from, Vec3::new(-CELL_SIZE, 0.0, 0.0));
        assert_eq!(pick(low, &level, &sight, &[], &[]), Some((-1, 0)));
        assert_eq!(pick(low, &level, &sight, &[], &[(2, 0)]), Some((2, 0)));
    }
}
