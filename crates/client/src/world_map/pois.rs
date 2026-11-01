//! The world map's pois (caves, houses, castles, crypts...) as entities
//! showing a model of what they are (`poi_model`), on an invisible token that
//! places them and takes clicks. Which chunks have one, and where, is what the
//! server has sent (`common::CurrentWorld`).
//!
//! Only pois near the player exist as entities, and only while the world
//! map is the current view: they are spawned as the player moves and
//! despawned when they fall out of range or the view changes.

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use shared::poi::PoiKind;
use shared::protocol::SeenPoi;
use shared::site::PoiLook;

use super::HALF_SIZE;
use super::poi_model;
use crate::common::{Clickable, CurrentWorld, MapPosition, Portal, SmoothPosition, Token};
use crate::player::Player;
use crate::view3d::{HoveredToken, TOKEN_DIAMETER, TOKEN_HALF_THICKNESS, TokenView};
use bevy::light::NotShadowCaster;
use bevy::render::render_resource::Face;

/// Pois exist this far from the player (each way, east–west and
/// north–south): the corners of the window when it is turned 45°, plus a
/// token's radius.
const RANGE: i64 = (HALF_SIZE as f32 * std::f32::consts::SQRT_2 + TOKEN_DIAMETER / 2.0) as i64 + 1;

/// How many versions of each town's model there are.
const VARIETIES: u64 = 8;

/// Which model an entity shows.
#[derive(Component, Clone, Copy)]
pub(super) struct PoiShape {
    kind: PoiKind,
    look: PoiLook,
    variety: u64,
}

/// How far above the ground a poi's flat base lies, so the ground doesn't
/// show through it, and how far its model stands, so the base doesn't show
/// through what lies flat in the model (a farm's fields).
const BASE_LIFT: f32 = 0.2;
const MODEL_LIFT: f32 = BASE_LIFT * 2.0;

/// A model and its outline.
type ModelMeshes = (Handle<Mesh>, Handle<Mesh>);

/// The models' meshes, each made once, the two flat bases (`false`: round;
/// `true`: octagonal, for towns), and their one material.
#[derive(Resource, Default)]
pub(super) struct PoiMeshes {
    meshes: HashMap<(PoiKind, PoiLook, u64), ModelMeshes>,
    bases: HashMap<bool, Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
    outline_material: Option<Handle<StandardMaterial>>,
}

/// The black shell round a poi's model, shown while the mouse is over it.
#[derive(Component)]
pub(super) struct PoiOutline {
    owner: Entity,
}

/// The poi entities that exist, by their position.
#[derive(Resource, Default)]
pub(super) struct SpawnedPois {
    entities: BTreeMap<(i64, i64), Entity>,
    /// Whether `entities` matches the player's position: false when the
    /// view has just become current.
    ready: bool,
}

/// The name of a poi's picture on the maps, for `minimap`'s store of them.
pub fn picture_name(kind: PoiKind, look: PoiLook) -> String {
    format!("poi:{kind:?}:{look:?}")
}

/// A poi's picture on the maps: its model drawn at an isometric angle.
pub fn picture(kind: PoiKind, look: PoiLook, size: u32) -> Vec<u8> {
    poi_model::build(kind, look, 0).picture(size)
}

/// When the player reaches a new grid point, the server has sent more of the
/// world, or the view has just become current: spawns the pois now in range and despawns those that left it.
pub(super) fn spawn_nearby(
    mut commands: Commands,
    world: Res<CurrentWorld>,
    player: Single<Ref<MapPosition>, With<Player>>,
    mut spawned: ResMut<SpawnedPois>,
) {
    if spawned.ready && !player.is_changed() && !world.is_changed() {
        return;
    }
    spawned.ready = true;
    let near = world.pois.in_area(
        (player.x - RANGE, player.y - RANGE),
        (player.x + RANGE, player.y + RANGE),
    );

    let wanted: BTreeMap<(i64, i64), SeenPoi> = near.into_iter().map(|f| ((f.x, f.y), f)).collect();
    spawned.entities.retain(|position, entity| {
        let keep = wanted.contains_key(position);
        if !keep {
            commands.entity(*entity).despawn();
        }
        keep
    });
    for (position, poi) in wanted {
        spawned.entities.entry(position).or_insert_with(|| {
            let at = MapPosition { x: poi.x, y: poi.y };
            commands
                .spawn((
                    // Clicking it travels there and enters its site.
                    Portal::Poi(poi),
                    Clickable,
                    at,
                    SmoothPosition::from(at),
                    Token {
                        no_base: true,
                        ..default()
                    },
                    PoiShape {
                        kind: poi.kind,
                        look: poi.look,
                        variety: (poi.x ^ poi.y).rem_euclid(VARIETIES as i64) as u64,
                    },
                ))
                .id()
        });
    }
}

/// Gives each new poi's token view its model, as a child so it goes where the
/// view goes and is hidden with it.
pub(super) fn add_models(
    mut commands: Commands,
    mut store: ResMut<PoiMeshes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    views: Query<(Entity, &TokenView), Added<TokenView>>,
    shapes: Query<&PoiShape>,
) {
    for (view, token_view) in &views {
        let Ok(shape) = shapes.get(token_view.owner) else {
            continue;
        };
        let store = &mut *store;
        let (mesh, outline_mesh) = store
            .meshes
            .entry((shape.kind, shape.look, shape.variety))
            .or_insert_with(|| {
                let model = poi_model::build(shape.kind, shape.look, shape.variety);
                (meshes.add(model.mesh()), meshes.add(model.outline_mesh()))
            })
            .clone();
        // White, so what shows is the mesh's own colours.
        let material = store
            .material
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    perceptual_roughness: 0.9,
                    ..default()
                })
            })
            .clone();
        let town = shape.kind.is_town();
        let base_mesh = store
            .bases
            .entry(town)
            .or_insert_with(|| meshes.add(poi_model::base_mesh(town)))
            .clone();
        let base = commands
            .spawn((
                Mesh3d(base_mesh),
                MeshMaterial3d(material.clone()),
                NotShadowCaster,
                // The view stands half a token's thickness above the ground.
                Transform::from_xyz(0.0, BASE_LIFT - TOKEN_HALF_THICKNESS, 0.0),
            ))
            .id();
        let model = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(0.0, MODEL_LIFT - TOKEN_HALF_THICKNESS, 0.0),
            ))
            .id();
        // Drawn from inside, so only what stands out past the model shows.
        let outline_material = store
            .outline_material
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::BLACK,
                    unlit: true,
                    cull_mode: Some(Face::Front),
                    ..default()
                })
            })
            .clone();
        let outline = commands
            .spawn((
                PoiOutline {
                    owner: token_view.owner,
                },
                Mesh3d(outline_mesh),
                MeshMaterial3d(outline_material),
                NotShadowCaster,
                Transform::from_xyz(0.0, MODEL_LIFT - TOKEN_HALF_THICKNESS, 0.0),
                Visibility::Hidden,
            ))
            .id();
        commands.entity(view).add_children(&[base, model, outline]);
    }
}

/// Shows the outline of the poi the mouse is over.
pub(super) fn outline_hovered(
    hovered: Res<HoveredToken>,
    mut outlines: Query<(&PoiOutline, &mut Visibility)>,
) {
    for (outline, mut visibility) in &mut outlines {
        let shown = if hovered.0 == Some(outline.owner) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        visibility.set_if_neq(shown);
    }
}

/// Leaving the world map: pois only exist while it is on screen.
pub(super) fn despawn_all(mut commands: Commands, mut spawned: ResMut<SpawnedPois>) {
    spawned.ready = false;
    for entity in std::mem::take(&mut spawned.entities).into_values() {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pois_are_shown_only_as_far_as_the_server_sends() {
        const { assert!(RANGE <= shared::protocol::WORLD_MAP_SIGHT) };
    }
}
