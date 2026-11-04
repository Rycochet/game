//! Standing the gates of the stretch of the Hall the player has round them,
//! with water running down in their openings.

use bevy::prelude::*;
use shared::hall::{GatePlace, LEVEL_ROWS, LEVEL_TOP};
use shared::protocol::LevelKind;
use std::collections::BTreeMap;

use super::GateModel;
use crate::common::CurrentDungeon;
use crate::dungeon_map::CELL_SIZE;

/// How fast the water runs down, in pictures a second.
const FLOW: f32 = 0.6;

/// The gates standing (each frame, with its runes and water as children), by
/// their cell as the client counts it, and the materials they share.
#[derive(Resource, Default)]
pub(super) struct Gates {
    standing: BTreeMap<(i64, i64), Entity>,
    stone: Option<Handle<StandardMaterial>>,
    water: Option<Handle<StandardMaterial>>,
}

/// When the stretch of the Hall changes, stands the gates on it and takes
/// away those off it; anywhere else, takes them all away.
pub(super) fn place_gates(
    dungeon: Res<CurrentDungeon>,
    mut gates: ResMut<Gates>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    if !dungeon.is_changed() {
        return;
    }
    let hall = dungeon.info.as_ref().and_then(|info| match info.kind {
        LevelKind::Hall { origin } => Some((origin, info.bounds)),
        LevelKind::Poi(_) => None,
    });
    let Some((origin, (min_x, _, width, _))) = hall else {
        remove(&mut gates, &mut commands);
        return;
    };
    let wanted: BTreeMap<(i64, i64), GatePlace> = (LEVEL_TOP..LEVEL_TOP + LEVEL_ROWS)
        .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
        .filter_map(|(x, y)| GatePlace::at(x + origin, y).map(|gate| ((x, y), gate)))
        .collect();
    gates.standing.retain(|cell, frame| {
        let stays = wanted.contains_key(cell);
        if !stays {
            commands.entity(*frame).despawn();
        }
        stays
    });
    let stone = gates
        .stone
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                unlit: true,
                ..default()
            })
        })
        .clone();
    let water = gates
        .water
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color_texture: Some(images.add(GateModel::water_picture())),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                double_sided: true,
                ..default()
            })
        })
        .clone();
    for (cell, gate) in wanted {
        if gates.standing.contains_key(&cell) {
            continue;
        }
        let design = gate.design();
        let runes = commands
            .spawn((
                Mesh3d(meshes.add(GateModel::runes(design, gate.code()).mesh(CELL_SIZE))),
                MeshMaterial3d(stone.clone()),
            ))
            .id();
        let sheet = commands
            .spawn((
                Mesh3d(meshes.add(GateModel::water(design, CELL_SIZE))),
                MeshMaterial3d(water.clone()),
            ))
            .id();
        let at = Vec3::new(cell.0 as f32 * CELL_SIZE, 0.0, cell.1 as f32 * CELL_SIZE);
        let frame = commands
            .spawn((
                Mesh3d(meshes.add(GateModel::frame(design).mesh(CELL_SIZE))),
                MeshMaterial3d(stone.clone()),
                Transform::from_translation(at),
                Visibility::Visible,
            ))
            .add_children(&[runes, sheet])
            .id();
        gates.standing.insert(cell, frame);
    }
}

/// Runs the water down every gate's opening, by sliding its picture.
pub(super) fn run_water(
    gates: Res<Gates>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if gates.standing.is_empty() {
        return;
    }
    if let Some(mut water) = gates.water.as_ref().and_then(|w| materials.get_mut(w)) {
        let down = (time.elapsed_secs() * FLOW).fract();
        water.uv_transform = bevy::math::Affine2::from_translation(Vec2::new(0.0, -down));
    }
}

/// Leaving the dungeon view: the gates go.
pub(super) fn remove_gates(mut gates: ResMut<Gates>, mut commands: Commands) {
    remove(&mut gates, &mut commands);
}

fn remove(gates: &mut Gates, commands: &mut Commands) {
    for (_, frame) in std::mem::take(&mut gates.standing) {
        commands.entity(frame).despawn();
    }
}
