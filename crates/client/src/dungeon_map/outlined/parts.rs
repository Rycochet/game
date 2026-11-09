//! The three meshes outlines are drawn with, and putting what was built in them.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::Face;

use super::super::Shown;
use crate::view3d::{XRAY_WHITE, XrayMaterial};

/// What was built, each `None` if there is nothing.
pub struct OutlinedMeshes {
    pub(in crate::dungeon_map) black: Option<Mesh>,
    pub(in crate::dungeon_map) fill: Option<Mesh>,
    pub(in crate::dungeon_map) lines: Option<Mesh>,
}

/// Which of the three meshes an entity holds.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(in crate::dungeon_map) enum OutlinedPart {
    Black,
    Fill,
    Lines,
}

/// Spawns the three (empty, hidden) meshes under the scene.
pub(in crate::dungeon_map) fn spawn_outlined(
    commands: &mut Commands,
    scene: Entity,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    xray: &mut Assets<XrayMaterial>,
) {
    let black = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        unlit: true,
        cull_mode: Some(Face::Front),
        ..default()
    });
    let mut empty = || Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0))));
    commands.entity(scene).with_children(|parent| {
        parent.spawn((
            OutlinedPart::Black,
            empty(),
            MeshMaterial3d(black),
            NoFrustumCulling,
            NotShadowCaster,
            Visibility::Hidden,
        ));
        parent.spawn((
            OutlinedPart::Fill,
            empty(),
            MeshMaterial3d(xray.add(XrayMaterial::fill(XRAY_WHITE))),
            NoFrustumCulling,
            NotShadowCaster,
            Visibility::Hidden,
        ));
        parent.spawn((
            OutlinedPart::Lines,
            empty(),
            MeshMaterial3d(xray.add(XrayMaterial::outline(XRAY_WHITE))),
            NoFrustumCulling,
            NotShadowCaster,
            Visibility::Hidden,
        ));
    });
}

/// Puts what `update_meshes` last built into the three meshes, hiding any
/// with nothing in it (Bevy warns of meshes without vertices).
pub(in crate::dungeon_map) fn apply_outlined(
    mut shown: ResMut<Shown>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut parts: Query<(&OutlinedPart, &Mesh3d, &mut Visibility)>,
) {
    let Some(built) = shown.outlined.take() else {
        return;
    };
    let mut built = [
        (OutlinedPart::Black, built.black),
        (OutlinedPart::Fill, built.fill),
        (OutlinedPart::Lines, built.lines),
    ];
    for (part, handle, mut visibility) in &mut parts {
        let Some((_, mesh)) = built.iter_mut().find(|(which, _)| which == part) else {
            continue;
        };
        match mesh.take() {
            Some(mesh) => {
                if let Some(mut stored) = meshes.get_mut(&handle.0) {
                    *stored = mesh;
                }
                visibility.set_if_neq(Visibility::Inherited);
            }
            None => {
                visibility.set_if_neq(Visibility::Hidden);
            }
        }
    }
}
