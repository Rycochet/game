//! The world map: the surface as a lit 3D terrain mesh, rendered by the GPU.
//! See `docs/context/client/world_map.md`.
//!
//! Like the prototype, it shows a square window of terrain around the player
//! with flat earth faces on its edges. Right-drag turns and tilts the view
//! around the player and the wheel zooms; the window stays lined up with the
//! camera, so the world turns inside it. Home returns to the starting view.
//! The camera and tokens come from `view3d`; this module draws the terrain
//! while `GameView::WorldMap` is current.

mod lights;
pub(crate) mod map_sky;
mod picking;
mod poi_model;
mod pois;
mod rivers;
mod roads;
pub mod sky;
mod terrain;

pub use gates::{gate_picture, gate_picture_name};
pub use pois::{picture as poi_picture, picture_name as poi_picture_name};

use bevy::prelude::*;
use pois::{PoiMeshes, SpawnedPois};
use rivers::KnownRivers;
use terrain::TerrainGrid;

use crate::common::GameView;
use crate::player::PlayerSystems;
use crate::view3d::View3dSystems;

// Split into files (`cargo xtask lint` keeps files small).
mod click;
mod forget;
mod gate_view;
mod gates;
mod parts;
mod showing;
mod spawn;
mod tokens;
mod update_terrain;

use click::pick_on_click;
use parts::{
    EdgeFaces, FollowsPlayer, RiverBanks, RiverStrips, RoadStrips, StripMeshes, Terrain,
    WorldMapScene,
};
use showing::{hide_scene, show_scene};
use spawn::spawn_view;
use tokens::{hide_tokens_outside_window, rest_tokens_on_terrain};
pub use update_terrain::window_half;
use update_terrain::{update_terrain, view_square};

/// World height units per 3D unit of height. 1.0 keeps the prototype's
/// proportions.
pub const HEIGHT_SCALE: f32 = 1.0;

/// Grid units from the player to each edge of the window (400×400, as the
/// prototype).
pub const HALF_SIZE: i64 = 200;

/// The furthest the window's sides may be from the player, which the terrain
/// and sky stores are sized for (checked when compiling). Zoomed out further
/// than this (only debug tools can), the window keeps this size and so gets
/// smaller on screen, rather than outgrowing what is worked out.
pub const MAX_HALF: f32 = HALF_SIZE as f32;

/// How much of the prototype's slope shading is baked into the terrain
/// colours (1.0 = the same as the prototype).
pub const SLOPE_SHADING: f64 = 1.0;

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        let in_view = in_state(GameView::WorldMap);
        app.init_resource::<TerrainGrid>()
            .init_resource::<SpawnedPois>()
            .init_resource::<gates::SpawnedGates>()
            .init_resource::<PoiMeshes>()
            .init_resource::<KnownRivers>()
            .init_resource::<map_sky::MapSky>()
            .add_plugins((
                MaterialPlugin::<sky::TerrainMaterial>::default(),
                MaterialPlugin::<sky::CloudMaterial>::default(),
                MaterialPlugin::<sky::PrecipitationMaterial>::default(),
            ))
            .add_systems(Startup, spawn_view)
            // Before anything draws the new world with the last one's caches.
            .add_systems(
                Update,
                (
                    pois::despawn_all,
                    gates::despawn_gates,
                    forget::forget_world,
                )
                    .before(update_terrain)
                    .run_if(on_message::<crate::common::WorldChanged>),
            )
            .add_systems(OnEnter(GameView::WorldMap), show_scene)
            .add_systems(
                OnExit(GameView::WorldMap),
                (hide_scene, pois::despawn_all, gates::despawn_gates),
            )
            .add_systems(
                Update,
                pick_on_click
                    .in_set(PlayerSystems::Input)
                    .after(View3dSystems::Click)
                    .run_if(in_view.clone()),
            )
            .add_systems(
                Update,
                (
                    map_sky::update.before(sky::update).before(lights::update),
                    update_terrain,
                    sky::update,
                    lights::update,
                    pois::spawn_nearby,
                    gates::spawn_gates,
                )
                    .after(PlayerSystems::Travel)
                    .before(View3dSystems::Place)
                    .run_if(in_view.clone()),
            )
            .add_systems(
                Update,
                (
                    pois::add_models,
                    gates::add_gate_models,
                    gate_view::face_camera,
                    pois::outline_hovered,
                    rest_tokens_on_terrain,
                    hide_tokens_outside_window,
                )
                    .after(View3dSystems::Place)
                    .run_if(in_view),
            );
    }
}
