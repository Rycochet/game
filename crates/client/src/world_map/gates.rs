//! A planet's World Gates on the world map, as the server tells them
//! (`CurrentWorld::gates`): tokens with the Hall's gate models (`hall_view`):
//! the frame, the runes of the Hall gate each leads to and the water in its
//! opening, just as that Hall gate looks, on a town's octagonal base, standing
//! about as tall as a tall tower. Clicking one travels there and goes through
//! it (`site`).

use std::collections::HashMap;

use bevy::prelude::*;
use shared::hall::GatePlace;
use shared::protocol::GateInfo;
use shared::world::WorldId;

use super::poi_model::PoiModel;
use super::pois::{MODEL_LIFT, PoiMeshes};
use crate::common::{Clickable, CurrentWorld, MapPosition, Portal, SmoothPosition, Token};
use crate::hall_view::{GateModel, GateWater};
use crate::view3d::{TOKEN_HALF_THICKNESS, TokenView};

/// How tall a gate stands on the world map, in grid points: a five-floor
/// tower's height, so it stands out as a tall tower does.
const GATE_HEIGHT: f32 = 11.0;

/// The gate tokens there are, and the world they are of (`None`: none yet).
#[derive(Resource, Default)]
pub(super) struct SpawnedGates {
    entities: Vec<Entity>,
    world: Option<WorldId>,
}

/// The Hall gate a token's gate leads to, which it looks just like.
#[derive(Component, Clone, Copy)]
pub(super) struct GateShape(GatePlace);

/// How much bigger than the Hall's (in cells) a gate is drawn on the maps.
fn scale_of(place: GatePlace) -> f32 {
    GATE_HEIGHT / GateModel::size(place.design()).1
}

/// A gate as a poi's model: the Hall gate `place`'s frame and runes, in grid
/// points.
fn gate_model(place: GatePlace) -> PoiModel {
    let (design, scale) = (place.design(), scale_of(place));
    let parts = [
        GateModel::frame(design),
        GateModel::runes(design, place.code()),
    ];
    let triangles: Vec<_> = parts
        .iter()
        .flat_map(|part| {
            part.triangles()
                .map(|(corners, colour)| (corners.map(|corner| corner * scale), colour))
                .collect::<Vec<_>>()
        })
        .collect();
    PoiModel::from_triangles(triangles)
}

/// The name of the picture on the maps of a gate leading to Hall gate
/// `place` (a `GatePlace::number`), for `minimap`'s store of them.
pub fn gate_picture_name(place: u32) -> String {
    format!("gate:{place}")
}

/// That gate's picture on the maps: its model drawn as pois' are.
pub fn gate_picture(place: u32, size: u32) -> Vec<u8> {
    gate_model(GatePlace::from_number(place)).picture(size)
}

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
            GateShape(GatePlace::from_number(gate.place)),
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

/// A gate's model, outline and water.
type GateMeshSet = (Handle<Mesh>, Handle<Mesh>, Handle<Mesh>);

/// The gates' meshes (model, outline, water), each made once.
#[derive(Default)]
pub(super) struct GateMeshes {
    models: HashMap<u32, GateMeshSet>,
}

/// Gives each new gate token's view its model on an octagonal base, as pois
/// have theirs, and the water in its opening.
pub(super) fn add_gate_models(
    mut commands: Commands,
    mut store: ResMut<PoiMeshes>,
    mut made: Local<GateMeshes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    views: Query<(Entity, &TokenView), Added<TokenView>>,
    shapes: Query<&GateShape>,
) {
    for (view, token_view) in &views {
        let Ok(&GateShape(place)) = shapes.get(token_view.owner) else {
            continue;
        };
        let (mesh, outline, water) = made
            .models
            .entry(place.number())
            .or_insert_with(|| {
                let model = gate_model(place);
                let water = GateModel::water(place.design(), scale_of(place));
                (
                    meshes.add(model.mesh()),
                    meshes.add(model.outline_mesh()),
                    meshes.add(water),
                )
            })
            .clone();
        let pair = (&mut *meshes, &mut *materials);
        store.attach(
            &mut commands,
            pair,
            view,
            token_view.owner,
            (mesh, outline),
            true,
        );
        let sheet = commands
            .spawn((
                Mesh3d(water),
                // Blue, or orange once gone through (`hall_view`).
                GateWater(place.number()),
                Transform::from_xyz(0.0, MODEL_LIFT - TOKEN_HALF_THICKNESS, 0.0),
            ))
            .id();
        commands.entity(view).add_child(sheet);
    }
}
