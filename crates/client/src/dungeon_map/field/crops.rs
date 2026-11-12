//! Everything standing in the fields that anyone can see.

use bevy::prelude::*;
use shared::dungeon::Sight;
use shared::site::FieldLook;

use super::super::CELL_SIZE;
use super::super::mesh::Builder;
use super::super::town::{self};
use super::{CELL_PIXELS, CROP_ROWS, CROP_TILES, HALF_CELL, Stage};

/// Everything standing in the fields, wherever anyone can see it, each on its
/// cell's `ground`: nothing is
/// drawn of it from memory, or in a ruin, whose fields have run to seed.
pub fn build_crops(
    fields: &[FieldLook],
    sight: &Sight,
    remembered: &Sight,
    ruined: bool,
    ground: &dyn Fn(Vec3) -> f32,
) -> Builder {
    let mut crops = Builder::default();
    if ruined {
        return crops;
    }
    for field in fields {
        let Some((tile, pixels)) = Stage::of(field).crop() else {
            continue;
        };
        let rect = field.rect;
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                if sight.is_seen(x, y) && remembered.is_seen(x, y) {
                    let from = crops.positions.len();
                    cell_crops(&mut crops, town::cell_centre(x, y), tile, pixels);
                    crops.drape(from, ground);
                }
            }
        }
    }
    crops
}

/// A `#` of crops in a cell: two rows each way.
pub(super) fn cell_crops(crops: &mut Builder, centre: Vec3, tile: f32, pixels: f32) {
    let top = pixels / CELL_PIXELS;
    let up = Vec3::Y * top * CELL_SIZE;
    let uv = |u: f32, v: f32| [(tile + u) / CROP_TILES, v];
    let uvs = [
        uv(0.0, 1.0),
        uv(1.0, 1.0),
        uv(1.0, 1.0 - top),
        uv(0.0, 1.0 - top),
    ];
    let colours = [Vec3::ONE; 4];
    for row in CROP_ROWS {
        let offset = row * CELL_SIZE;
        for (across, normal, along) in [
            (Vec3::X * offset, Vec3::X, Vec3::Z),
            (Vec3::Z * offset, Vec3::Z, Vec3::X),
        ] {
            let middle = centre + across;
            let (a, b) = (middle - along * HALF_CELL, middle + along * HALF_CELL);
            crops.free([a, b, b + up, a + up], normal, uvs, colours);
        }
    }
}
