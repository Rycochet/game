//! A planet's World Gates on the world map, as the server tells them
//! (`CurrentWorld::gates`): tokens with the Hall's gate models (`hall_view`),
//! with the runes of the Hall gate each leads to. Clicking one travels there and
//! goes through it (`site`).

use bevy::prelude::*;
use shared::hall::{GateDesign, GatePlace};
use shared::protocol::GateInfo;
use shared::world::WorldId;

use super::pois::MODEL_LIFT;
use crate::common::{Clickable, CurrentWorld, MapPosition, Portal, SmoothPosition, Token};
use crate::hall_view::GateModel;
use crate::view3d::{TOKEN_HALF_THICKNESS, TokenView};

/// How big a gate stands on the world map: grid points to one of the Hall's
/// cells, so it is about as big as a poi's model.
const GATE_SCALE: f32 = 9.0;

/// The gate tokens there are, and the world they are of (`None`: none yet).
#[derive(Resource, Default)]
pub(super) struct SpawnedGates {
    entities: Vec<Entity>,
    world: Option<WorldId>,
}

/// What a gate looks like: its Hall gate's design and code, if it leads to one.
#[derive(Component, Clone, Copy)]
pub(super) struct GateShape(Option<GatePlace>);

/// Spawns every World Gate of the world (there are only a few, far apart: the
/// world map hides those outside its window), once the view is current.
pub(super) fn spawn_gates(
    mut commands: Commands,
    world: Res<CurrentWorld>,
    mut spawned: ResMut<SpawnedGates>,
) {
    if spawned.world == Some(world.id) {
        return;
    }
    spawned.world = Some(world.id);
    for entity in spawned.entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for gate in &world.gates {
        spawned.entities.push(spawn_gate(&mut commands, *gate));
    }
}

fn spawn_gate(commands: &mut Commands, gate: GateInfo) -> Entity {
    let at = MapPosition {
        x: gate.x,
        y: gate.y,
    };
    commands
        .spawn((
            Portal::Gate(gate),
            Clickable,
            at,
            SmoothPosition::from(at),
            Token {
                no_base: true,
                ..default()
            },
            GateShape(gate.place.map(GatePlace::from_number)),
        ))
        .id()
}

/// Leaving the world map, or moving to another world: the gates go.
pub(super) fn despawn_gates(mut commands: Commands, mut spawned: ResMut<SpawnedGates>) {
    spawned.world = None;
    for entity in spawned.entities.drain(..) {
        commands.entity(entity).despawn();
    }
}

/// Gives each new gate token's view its model: the frame, and the runes of
/// the Hall gate it leads to.
pub(super) fn add_gate_models(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut stone: Local<Option<Handle<StandardMaterial>>>,
    views: Query<(Entity, &TokenView), Added<TokenView>>,
    shapes: Query<&GateShape>,
) {
    for (view, token_view) in &views {
        let Ok(GateShape(place)) = shapes.get(token_view.owner) else {
            continue;
        };
        // Its light is in its colours.
        let stone = stone
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    unlit: true,
                    ..default()
                })
            })
            .clone();
        let design = place.map_or(GateDesign::RoundArch, GatePlace::design);
        let mut parts = vec![GateModel::frame(design)];
        if let Some(place) = place {
            parts.push(GateModel::runes(design, place.code()));
        }
        let lift = Transform::from_xyz(0.0, MODEL_LIFT - TOKEN_HALF_THICKNESS, 0.0);
        for part in parts {
            let mesh = commands
                .spawn((
                    Mesh3d(meshes.add(part.mesh(GATE_SCALE))),
                    MeshMaterial3d(stone.clone()),
                    lift,
                ))
                .id();
            commands.entity(view).add_child(mesh);
        }
    }
}
