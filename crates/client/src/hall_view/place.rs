//! Standing the gates of the stretch of the Hall the player has round them,
//! with water running down in their openings (coloured by `visited`).

use bevy::prelude::*;
use shared::hall::{GatePlace, LEVEL_ROWS, LEVEL_TOP};
use shared::protocol::LevelKind;
use std::collections::BTreeMap;

use super::GateModel;
use super::visited::{GateWater, WaterMaterials};
use crate::common::{CurrentDungeon, CurrentWorld, LightSource, MapPosition, SmoothPosition};
use crate::dungeon_map::CELL_SIZE;
use shared::wilderness::SCALE;

/// How fast the water runs down, in pictures a second.
const FLOW: f32 = 0.6;

/// How far a World Gate's light reaches in the wilderness, in cells (owner).
const GATE_LIGHT_RADIUS: u32 = 8;

/// The gates standing (each frame, with its runes and water as children), by
/// their cell as the client counts it, and the stone they share.
#[derive(Resource, Default)]
pub(super) struct Gates {
    standing: BTreeMap<(i64, i64), Entity>,
    stone: Option<Handle<StandardMaterial>>,
}

impl Gates {
    /// Each gate's frame, by its cell.
    pub(super) fn standing(&self) -> &BTreeMap<(i64, i64), Entity> {
        &self.standing
    }
}

/// When the stretch of the Hall changes, stands the gates on it and takes
/// away those off it; anywhere else, takes them all away.
pub(super) fn place_gates(
    dungeon: Res<CurrentDungeon>,
    world: Option<Res<CurrentWorld>>,
    mut gates: ResMut<Gates>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if !dungeon.is_changed() {
        return;
    }
    let in_wilderness = dungeon
        .info
        .as_ref()
        .is_some_and(|info| matches!(info.kind, LevelKind::Wilderness));
    let wanted: BTreeMap<(i64, i64), GatePlace> = match dungeon.info.as_ref() {
        Some(info) => match info.kind {
            LevelKind::Hall { origin } => {
                let (min_x, _, width, _) = info.bounds;
                (LEVEL_TOP..LEVEL_TOP + LEVEL_ROWS)
                    .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
                    .filter_map(|(x, y)| GatePlace::at(x + origin, y).map(|gate| ((x, y), gate)))
                    .collect()
            }
            LevelKind::Wilderness => {
                world.map_or_else(BTreeMap::new, |world| wilderness_gates(&world, info.bounds))
            }
            LevelKind::Poi(_) => BTreeMap::new(),
        },
        None => BTreeMap::new(),
    };
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
                GateWater(gate.number()),
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
        if in_wilderness {
            // A light round the gate, as on the world map: `dungeon_map` lights
            // the ground and tokens from any `LightSource` with a position.
            // The Hall has its own even light.
            let position = MapPosition {
                x: cell.0,
                y: cell.1,
            };
            commands.entity(frame).insert((
                position,
                SmoothPosition::from(position),
                LightSource {
                    radius: GATE_LIGHT_RADIUS,
                    colour: Color::WHITE,
                },
            ));
        }
        gates.standing.insert(cell, frame);
    }
}

/// The World Gates standing in the wilderness's window, by their cell (the
/// middle cell of their world map point, in the window's frame), each built as
/// the Hall's gate it leads to.
fn wilderness_gates(
    world: &CurrentWorld,
    (min_x, min_y, width, height): (i64, i64, i64, i64),
) -> BTreeMap<(i64, i64), GatePlace> {
    let planet = world.surface.width() * SCALE;
    world
        .gates
        .iter()
        .filter_map(|gate| {
            let (x, y) = (gate.x * SCALE + SCALE / 2, gate.y * SCALE + SCALE / 2);
            let x = [-planet, 0, planet]
                .into_iter()
                .map(|turn| x + turn)
                .find(|x| (min_x..min_x + width).contains(x))?;
            (min_y..min_y + height)
                .contains(&y)
                .then(|| ((x, y), GatePlace::from_number(gate.place)))
        })
        .collect()
}

/// Runs the water down every gate's opening, by sliding its pictures.
pub(super) fn run_water(
    gates: Res<Gates>,
    waters: Res<WaterMaterials>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if gates.standing.is_empty() {
        return;
    }
    let down = (time.elapsed_secs() * FLOW).fract();
    for handle in [&waters.blue, &waters.orange] {
        if let Some(mut water) = materials.get_mut(handle) {
            water.uv_transform = bevy::math::Affine2::from_translation(Vec2::new(0.0, -down));
        }
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
