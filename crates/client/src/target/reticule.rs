//! The reticule round the target: a black ring with four ticks pointing in, flat
//! on top of the target's token, under its picture, turning slowly.

use std::f32::consts::{PI, TAU};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::common::{Other, Preferences, Target};
use crate::view3d::{TOKEN_DIAMETER, TOKEN_HALF_THICKNESS, TokenView};

/// The ring, in 3D units: a little inside the token's edge.
const OUTER: f32 = TOKEN_DIAMETER * 0.47;
const INNER: f32 = TOKEN_DIAMETER * 0.40;
/// The ticks reach in this far, and are this wide.
const TICK_IN: f32 = TOKEN_DIAMETER * 0.28;
const TICK_WIDTH: f32 = TOKEN_DIAMETER * 0.07;
/// The white outline's width round the black, in 3D units.
const OUTLINE: f32 = TOKEN_DIAMETER * 0.025;
/// How high above the token's top the ring and its outline lie, so they don't
/// flicker in and out of it.
const RING_LIFT: f32 = 0.3;
const OUTLINE_LIFT: f32 = 0.15;
/// One turn every this many seconds.
const TURN_SECONDS: f32 = 8.0;
/// Pieces round the ring.
const SEGMENTS: u32 = 48;

/// Black, seen on any token by its white outline (owner, 2026-11-07).
const RING: Color = Color::BLACK;
const OUTLINE_COLOUR: Color = Color::WHITE;

/// The reticule's root.
#[derive(Component)]
pub(super) struct Reticule;

/// Makes the reticule, hidden until there is a target in sight.
pub(super) fn spawn_reticule(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let unlit = |colour: Color| StandardMaterial {
        base_color: colour,
        unlit: true,
        // Seen from above only, but either winding then shows.
        cull_mode: None,
        ..default()
    };
    let ring = materials.add(unlit(RING));
    let outline = materials.add(unlit(OUTLINE_COLOUR));
    commands
        .spawn((Reticule, Transform::default(), Visibility::Hidden))
        .with_children(|reticule| {
            reticule.spawn((
                Mesh3d(meshes.add(reticule_mesh(0.0))),
                MeshMaterial3d(ring),
                Transform::from_xyz(0.0, RING_LIFT, 0.0),
            ));
            reticule.spawn((
                Mesh3d(meshes.add(reticule_mesh(OUTLINE))),
                MeshMaterial3d(outline),
                Transform::from_xyz(0.0, OUTLINE_LIFT, 0.0),
            ));
        });
}

/// Puts the reticule on the target's token while it can be seen, turning it
/// (it holds still with Reduce Motion), and hides it otherwise.
pub(super) fn place_reticule(
    time: Res<Time>,
    target: Res<Target>,
    preferences: Res<Preferences>,
    others: Query<(Entity, &Other)>,
    views: Query<(&TokenView, &Transform, &InheritedVisibility), Without<Reticule>>,
    mut reticule: Single<(&mut Transform, &mut Visibility), With<Reticule>>,
) {
    let owner = target
        .0
        .and_then(|id| others.iter().find(|(_, other)| other.id == id))
        .map(|(entity, _)| entity);
    let view = owner.and_then(|owner| {
        views
            .iter()
            .find(|(view, _, shown)| view.owner == owner && shown.get())
    });
    let (transform, visibility) = &mut *reticule;
    let Some((_, token, _)) = view else {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    };
    visibility.set_if_neq(Visibility::Inherited);
    transform.translation = token.translation + Vec3::Y * TOKEN_HALF_THICKNESS;
    if !preferences.graphics.reduce_motion {
        let turn = time.elapsed_secs() / TURN_SECONDS * TAU;
        transform.rotation = Quat::from_rotation_y(-turn);
    }
}

/// The reticule's flat shape, facing up, grown by `grow` all round.
pub(super) fn reticule_mesh(grow: f32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut quad = |corners: [Vec2; 4]| {
        let start = positions.len() as u32;
        positions.extend(corners.map(|c| [c.x, 0.0, c.y]));
        // Wound to face up (+Y).
        indices.extend([start, start + 2, start + 1, start, start + 3, start + 2]);
    };
    let (outer, inner) = (OUTER + grow, INNER - grow);
    for step in 0..SEGMENTS {
        let at = |s: u32| {
            let angle = s as f32 / SEGMENTS as f32 * TAU;
            Vec2::new(angle.cos(), angle.sin())
        };
        let (a, b) = (at(step), at(step + 1));
        quad([a * inner, a * outer, b * outer, b * inner]);
    }
    for quarter in 0..4 {
        let way = Vec2::from_angle(quarter as f32 * PI / 2.0 + PI / 4.0);
        let side = way.perp() * (TICK_WIDTH / 2.0 + grow);
        let (from, to) = (way * (TICK_IN - grow), way * (INNER + grow));
        quad([from - side, to - side, to + side, from + side]);
    }
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}
