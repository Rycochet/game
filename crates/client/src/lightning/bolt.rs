//! Bolts of lightning as meshes: made where a view puts them, glowing, then
//! fading away and gone.

use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::envelope;
use super::shape::{Segment, segments};
use crate::common::Preferences;

/// A bolt's colour: white with a little blue, glowing (lit by nothing).
const BOLT_COLOUR: Vec3 = Vec3::new(0.85, 0.9, 1.0);

/// On a bolt drawn by `spawn_bolt`: how long it has been showing.
#[derive(Component, Debug, Default)]
pub(super) struct Bolt {
    age: f32,
}

/// Draws a bolt: the shape with this seed, `height` tall, striking `ground`,
/// its main stroke `width` across. The caller adds what its view needs (a
/// marker to hide it with the scene, say); it fades and goes by itself.
pub fn spawn_bolt(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    seed: u64,
    ground: Vec3,
    height: f32,
    width: f32,
) -> Entity {
    let mesh = meshes.add(bolt_mesh(&segments(seed), height, width));
    let material = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(BOLT_COLOUR.x, BOLT_COLOUR.y, BOLT_COLOUR.z, 0.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(ground),
            NotShadowCaster,
            Bolt::default(),
        ))
        .id()
}

/// Every frame: each bolt's brightness by its age, and gone once it fades.
pub(super) fn fade_bolts(
    mut commands: Commands,
    time: Res<Time>,
    preferences: Res<Preferences>,
    mut bolts: Query<(Entity, &mut Bolt, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let gentle = preferences.graphics.reduce_motion;
    for (entity, mut bolt, material) in &mut bolts {
        match envelope::bolt_alpha(bolt.age, gentle) {
            Some(alpha) => {
                if let Some(mut material) = materials.get_mut(&material.0) {
                    material.base_color.set_alpha(alpha);
                }
                bolt.age += time.delta_secs();
            }
            None => commands.entity(entity).despawn(),
        }
    }
}

/// Each piece as two crossed strips, so it looks the same from any side.
fn bolt_mesh(pieces: &[Segment], height: f32, width: f32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    for piece in pieces {
        let (from, to) = (piece.from * height, piece.to * height);
        let along = (to - from).normalize_or(Vec3::NEG_Y);
        let across = along.cross(Vec3::X).normalize_or(Vec3::Z);
        let other = along.cross(across).normalize_or(Vec3::X);
        for side in [across, other] {
            let half = side * width * piece.width / 2.0;
            let start = positions.len() as u32;
            for corner in [from - half, from + half, to + half, to - half] {
                positions.push(corner.to_array());
                normals.push(side.cross(along).to_array());
            }
            indices.extend([start, start + 1, start + 2, start, start + 2, start + 3]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}
