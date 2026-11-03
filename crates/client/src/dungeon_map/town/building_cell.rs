//! One cell of a building, in quarters.

use bevy::prelude::*;
use shared::dungeon::RoomId;
use shared::site::BuildingLook;

use super::super::CELL_SIZE;
use super::super::walls::{Layer, Layers, Scene};
use super::{
    HALF_CELL, Piece, Roof, WALL_OUT, build_door_or_window, cell_centre, roof_centre, roof_tile,
    strip, wall_face, wall_tile,
};

/// One cell of a building, in quarters: the part of a quarter inside the wall
/// has roof; beyond the wall it has ground, unless the building carries on.
pub(super) fn build_cell(
    layers: &mut Layers,
    scene: &Scene,
    building: &BuildingLook,
    (x, y): (i64, i64),
    id: RoomId,
    seen: bool,
    hides: bool,
) {
    let level = scene.level;
    let centre = cell_centre(x, y);
    let inside = |dx: i64, dy: i64| level.room(x + dx, y + dy) == Some(id);
    let neighbours: [[bool; 3]; 3] = std::array::from_fn(|row| {
        std::array::from_fn(|column| inside(column as i64 - 1, row as i64 - 1))
    });
    let roof_layer = if hides {
        Layer::SeeThrough
    } else if seen {
        Layer::Solid
    } else {
        Layer::Memory
    };
    let tile = roof_tile(building.roof);
    if neighbours.iter().flatten().all(|&inside| inside) {
        roof_centre(layers, scene, roof_layer, centre);
        return;
    }
    let wall = wall_tile(building.walls);
    let roof = Roof {
        centre,
        neighbours,
        layer: roof_layer,
        tile,
    };
    for (sx, sz) in [(1i64, 1i64), (1, -1), (-1, 1), (-1, -1)] {
        let (fx, fz) = (sx as f32, sz as f32);
        let east = inside(sx, 0);
        let south = inside(0, sz);
        let corner = inside(sx, sz);
        let span = |low: f32, high: f32, sign: f32| {
            let (a, b) = (low * sign, high * sign);
            (a.min(b), a.max(b))
        };
        let (x_in, x_out) = (span(0.0, WALL_OUT, fx), span(WALL_OUT, HALF_CELL, fx));
        let (z_in, z_out) = (span(0.0, WALL_OUT, fz), span(WALL_OUT, HALF_CELL, fz));
        let patch = |(x0, x1): (f32, f32), (z0, z1): (f32, f32)| {
            Rect::from_corners(Vec2::new(x0, z0), Vec2::new(x1, z1))
        };
        let beside_x = (x + sx, y);
        let beside_z = (x, y + sz);
        let diagonal = (x + sx, y + sz);
        roof.patch(layers, scene, patch(x_in, z_in));
        if east {
            roof.patch(layers, scene, patch(x_out, z_in));
        } else {
            strip(layers, scene, centre, patch(x_out, z_in), &[beside_x]);
        }
        if south {
            roof.patch(layers, scene, patch(x_in, z_out));
        } else {
            strip(layers, scene, centre, patch(x_in, z_out), &[beside_z]);
        }
        if east && south && corner {
            roof.patch(layers, scene, patch(x_out, z_out));
        } else {
            let open = [beside_x, beside_z, diagonal];
            strip(layers, scene, centre, patch(x_out, z_out), &open);
        }
        // The walls round what has roof.
        let point = |a: f32, b: f32| centre + Vec3::new(a * fx, 0.0, b * fz);
        let across_z = |a: f32, b: f32| (a * fz / CELL_SIZE + 0.5, b * fz / CELL_SIZE + 0.5);
        let across_x = |a: f32, b: f32| (a * fx / CELL_SIZE + 0.5, b * fx / CELL_SIZE + 0.5);
        let x_normal = Vec3::X * fx;
        let z_normal = Vec3::Z * fz;
        if !east {
            let piece = Piece {
                from: point(WALL_OUT, 0.0),
                to: point(WALL_OUT, WALL_OUT),
                normal: x_normal,
                across: across_z(0.0, WALL_OUT),
            };
            wall_face(layers, scene, ((x, y), beside_x), piece, wall, hides);
        }
        if !south {
            let piece = Piece {
                from: point(0.0, WALL_OUT),
                to: point(WALL_OUT, WALL_OUT),
                normal: z_normal,
                across: across_x(0.0, WALL_OUT),
            };
            wall_face(layers, scene, ((x, y), beside_z), piece, wall, hides);
        }
        if east && !(south && corner) {
            let outer = if south { diagonal } else { beside_z };
            let piece = Piece {
                from: point(WALL_OUT, WALL_OUT),
                to: point(HALF_CELL, WALL_OUT),
                normal: z_normal,
                across: across_x(WALL_OUT, HALF_CELL),
            };
            wall_face(layers, scene, ((x, y), outer), piece, wall, hides);
        }
        if south && !(east && corner) {
            let outer = if east { diagonal } else { beside_x };
            let piece = Piece {
                from: point(WALL_OUT, WALL_OUT),
                to: point(WALL_OUT, HALF_CELL),
                normal: x_normal,
                across: across_z(WALL_OUT, HALF_CELL),
            };
            wall_face(layers, scene, ((x, y), outer), piece, wall, hides);
        }
    }
    build_door_or_window(layers, scene, building, (x, y), hides);
}
