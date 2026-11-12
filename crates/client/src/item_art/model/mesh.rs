//! A model's meshes: each piece flat-faced with its body parts, and the whole as one
//! smooth surface for the outline and the x-ray.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::super::body::{Triangle, mesh_of};
use super::bones::{ATTRIBUTE_BODY_PART, Bones};

/// Points every mesh of a model gets a triangle of no size at, so all have the same
/// bounds, reaching far below the ground. Bevy sorts what it draws with see-through
/// things by the middle of each mesh's bounds: a model's meshes then sort together, and
/// as further than anything really there, so they are drawn before every see-through
/// wall, water and memory, which blend over them where they are in front (`view3d`'s
/// `model`).
pub(super) type Bounds = [Vec3; 3];

/// How far below the ground a model's bounds reach, in millimetres: well past the far
/// end of the camera's view, looking down at its lowest tilt.
const SUNK: f32 = 2.0e6;

/// The bounds for these triangles.
pub(super) fn bounds_of<'a>(triangles: impl Iterator<Item = &'a Triangle>) -> Bounds {
    let [low, high] = triangles.flatten().fold(
        [Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)],
        |[low, high], corner| [low.min(*corner), high.max(*corner)],
    );
    let middle = (low + high) / 2.0;
    [low, high, Vec3::new(middle.x, -SUNK, middle.z)]
}

/// A piece's mesh, flat-faced as its picture's, each corner tagged with the body part
/// it moves with.
pub(super) fn part_mesh(triangles: &[Triangle], bones: &Bones, bounds: Bounds) -> Mesh {
    let mut triangles = triangles.to_vec();
    triangles.extend(bounds.map(|corner| [corner; 3]));
    let triangles = &triangles;
    // A whole triangle moves together, by its middle, so no face is torn apart.
    let parts: Vec<u32> = triangles
        .iter()
        .flat_map(|[a, b, c]| [bones.part_at((*a + *b + *c) / 3.0); 3])
        .collect();
    mesh_of(triangles).with_inserted_attribute(ATTRIBUTE_BODY_PART, parts)
}

/// How finely corners are matched up, in millimetres: corners closer than this are
/// one, so the surface is closed where pieces were built from the same points.
const WELD: f32 = 0.5;

/// Every triangle as one surface, corners that meet joined, with normals smoothed
/// across them: grown along those, it stays whole (`view3d`'s outline).
pub(super) fn hull_mesh(triangles: &[Triangle], bounds: Bounds) -> Mesh {
    let mut index: HashMap<[i32; 3], u32> = HashMap::new();
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut indices: Vec<u32> = Vec::with_capacity(triangles.len() * 3);
    for triangle in triangles {
        // Its size and way out, together: bigger faces count for more.
        let facing = (triangle[1] - triangle[0]).cross(triangle[2] - triangle[0]);
        if facing.length_squared() < 1e-6 {
            continue;
        }
        for corner in triangle {
            let key = (*corner / WELD).round().as_ivec3().to_array();
            let at = *index.entry(key).or_insert_with(|| {
                positions.push(corner.to_array());
                normals.push(Vec3::ZERO);
                positions.len() as u32 - 1
            });
            normals[at as usize] += facing;
            indices.push(at);
        }
    }
    for corner in bounds {
        positions.push(corner.to_array());
        normals.push(Vec3::Y);
        let at = positions.len() as u32 - 1;
        indices.extend([at; 3]);
    }
    let normals: Vec<[f32; 3]> = normals
        .into_iter()
        .map(|normal| normal.normalize_or(Vec3::Y).to_array())
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}
