//! Turns a mouse click on the dungeon into a walkable cell, by following the
//! camera ray to the floor (raised in the wilderness: `Ground`) unless a solid
//! wall side is in the way. Stairs don't catch clicks above the floor, so what
//! is behind them can be clicked.

use bevy::prelude::*;
use shared::dungeon::{Level, Sight};
use shared::site::BuildingLook;

use super::ground_hit::Ground;
use super::mesh::WallFace;
use super::{CELL_SIZE, WALL_HEIGHT};

/// The building whose door the ray hits before any other solid wall piece.
pub fn door_clicked(
    ray: Ray3d,
    buildings: &[BuildingLook],
    solid_faces: &[WallFace],
    ground: Ground,
) -> Option<u16> {
    let direction = *ray.direction;
    let mut nearest: Option<(f32, u16)> = None;
    for building in buildings {
        let (dx, dy) = building.faces;
        let normal = Vec3::new(dx as f32, 0.0, dy as f32);
        let centre = Vec3::new(
            building.door.0 as f32 * CELL_SIZE,
            ground.at(building.door.0, building.door.1) + WALL_HEIGHT / 2.0,
            building.door.1 as f32 * CELL_SIZE,
        ) + normal * CELL_SIZE * 0.05;
        let face = WallFace {
            centre,
            normal,
            half_across: CELL_SIZE / 2.0,
            half_up: WALL_HEIGHT / 2.0,
        };
        if let Some(distance) = hits_face(ray.origin, direction, &face)
            && nearest.is_none_or(|(best, _)| distance < best)
        {
            nearest = Some((distance, building.id));
        }
    }
    let (distance, building) = nearest?;
    // A door stands just in front of its own wall, so that doesn't count.
    let blocked = solid_faces
        .iter()
        .any(|face| hits_face(ray.origin, direction, face).is_some_and(|d| d < distance - 0.5));
    (!blocked).then_some(building)
}

/// The seen walkable cell the ray first lands on, if no solid wall side is hit
/// first. See-through sides don't block: the floor shows through them.
pub fn pick(
    ray: Ray3d,
    level: &Level,
    sight: &Sight,
    solid_faces: &[WallFace],
    ground: Ground,
) -> Option<(i64, i64)> {
    let direction = *ray.direction;
    // Only rays going down reach the floor.
    let (distance, point, (x, y)) = ground.hit(ray)?;
    let blocked = solid_faces
        .iter()
        .any(|face| hits_face(ray.origin, direction, face).is_some_and(|d| d < distance));
    if blocked {
        return None;
    }
    let can_go = |&(x, y): &(i64, i64)| level.is_walkable(x, y) && sight.is_seen(x, y);
    if can_go(&(x, y)) {
        return Some((x, y));
    }
    // A wall takes up only the middle of its cell, so the rest looks like the
    // floor beside it: a click there means the cell on that side.
    let (ox, oz) = (
        point.x - x as f32 * CELL_SIZE,
        point.z - y as f32 * CELL_SIZE,
    );
    let beyond = |offset: f32| {
        if offset.abs() <= CELL_SIZE * 0.05 {
            0
        } else {
            offset.signum() as i64
        }
    };
    let sideways = (x + beyond(ox), y);
    let ahead = (x, y + beyond(oz));
    let tries = if ox.abs() > oz.abs() {
        [sideways, ahead]
    } else {
        [ahead, sideways]
    };
    tries.into_iter().find(can_go)
}

/// The cell under the point the ray reaches the floor at, walkable or not.
pub fn floor_cell(ray: Ray3d, ground: Ground) -> Option<(i64, i64)> {
    ground.hit(ray).map(|(_, _, cell)| cell)
}

/// How far along the ray it crosses the wall side, if it does from the front.
pub(super) fn hits_face(origin: Vec3, direction: Vec3, face: &WallFace) -> Option<f32> {
    let towards = direction.dot(face.normal);
    if towards >= 0.0 {
        return None;
    }
    let distance = (face.centre - origin).dot(face.normal) / towards;
    let offset = origin + direction * distance - face.centre;
    // Sideways along the side, and up it.
    let across = offset.dot(Vec3::Y.cross(face.normal)).abs();
    let up = offset.y.abs();
    (distance > 0.0 && across <= face.half_across && up <= face.half_up).then_some(distance)
}

#[cfg(test)]
mod tests;
