//! The dungeon (and building) view: a grid of floor and wall cells in 3D,
//! seen through `view3d`'s camera with the same tokens as the world map. See
//! `docs/context/client/dungeon_map.md`.
//!
//! Only what the player (or anything sharing its sight) can see is drawn:
//! floors, stairs, and thin walls with only the faces someone sees (`walls`).
//! A seen side facing away from the camera is drawn half see-through. What
//! was seen before but is out of sight now is drawn dim, grey and half
//! see-through. Walls and floors use tile pictures chosen by the kind of place
//! (`theme`). Light is dull everywhere, and brighter near light sources, which
//! walls block. Left-click a seen walkable cell to travel there.

mod daylight;
mod field;
mod herbs;
pub use field::CROP_STAGES;
pub use herbs::HERB_STAGES;
mod gate_picking;
mod holes;
mod liquid;
mod mesh;
mod oasis;
mod picking;
mod rain;
mod sight_lines;
mod stairs;
mod theme;
mod town;
mod walls;

pub use theme::TOWN_ATLAS;
pub use town::door_picture;

use bevy::prelude::*;
use daylight::{OutsideLight, TownLights};
use mesh::Surface;
use sight_lines::SurfaceMaterial;

use crate::common::GameView;
use crate::player::PlayerSystems;
use crate::view3d::{TOKEN_DIAMETER, View3dSystems};

// Split into files (`cargo xtask lint` keeps files small).
mod click;
mod growth;
mod hover;
mod lights;
mod parts;
mod pictures;
mod plain_pictures;
mod showing;
mod spawn_scene;
mod update_meshes;
mod wild_ground;

pub use click::DoorClicked;
use click::pick_on_click;
use growth::{update_crops, update_herbs};
use lights::sync_lights;
pub use parts::Shown;
use parts::{
    CropPart, DungeonScene, HerbPart, MemoryPart, SeeThroughPart, SolidPart, StairsThroughPart,
    SurfaceMaterials, TokenLight,
};
use pictures::{apply_theme, colour_ground, grey_pictures};
use showing::{hide_scene, hide_unseen_tokens, show_scene};
use spawn_scene::{load_tile, spawn_scene};
use update_meshes::update_meshes;

/// Size of a cell in 3D units: about the size of a token's picture.
pub const CELL_SIZE: f32 = TOKEN_DIAMETER * 1.5;

/// How tall walls stand.
const WALL_HEIGHT: f32 = CELL_SIZE;

/// The dull light everywhere, as a fraction of full light.
const AMBIENT_LIGHT: f32 = 0.1;

/// Opacity of wall sides facing away from the camera.
const SEE_THROUGH_ALPHA: f32 = 0.5;

/// Opacity of what is remembered but out of sight.
const MEMORY_ALPHA: f32 = 0.5;

/// Draws a wall's fade across the floor after the floor, before the walls.
const BLEND_DEPTH_BIAS: f32 = -0.5;

/// Tokens are lit by real lights (the tiles' light is baked into their
/// colours): a point light this high above each light source, and a dim
/// ambient fill so tokens in the dark are still visible.
const TOKEN_LIGHT_HEIGHT: f32 = CELL_SIZE;
const TOKEN_LIGHT_LUMENS: f32 = 1.0e8;
const TOKEN_AMBIENT_BRIGHTNESS: f32 = 150.0;

pub struct DungeonMapPlugin;

impl Plugin for DungeonMapPlugin {
    fn build(&self, app: &mut App) {
        let in_view = in_state(GameView::Dungeon);
        app.add_message::<DoorClicked>()
            .init_resource::<Shown>()
            .init_resource::<OutsideLight>()
            .init_resource::<TownLights>()
            .init_resource::<wild_ground::GroundTint>()
            .add_plugins(MaterialPlugin::<rain::RainMaterial>::default())
            .add_plugins(MaterialPlugin::<SurfaceMaterial>::default())
            .add_systems(Startup, spawn_scene)
            .add_systems(
                OnEnter(GameView::Dungeon),
                (show_scene, sight_lines::set_mask_active::<true>),
            )
            .add_systems(
                OnExit(GameView::Dungeon),
                (hide_scene, sight_lines::set_mask_active::<false>),
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
                    apply_theme,
                    colour_ground,
                    daylight::update_outside,
                    daylight::update_town_lights,
                    daylight::tint_outside,
                    daylight::token_ambient,
                    sight_lines::update_outlines,
                    sight_lines::place_mask,
                    wild_ground::work_out_tint,
                    update_meshes,
                    update_crops,
                    update_herbs,
                    hide_unseen_tokens,
                    sync_lights,
                    rain::update,
                )
                    .chain()
                    .after(PlayerSystems::Travel)
                    .after(View3dSystems::Place)
                    .run_if(in_view),
            )
            .add_systems(
                Update,
                (grey_pictures, plain_pictures::plain_pictures).chain(),
            )
            .add_systems(
                Update,
                hover::name_hovered_place
                    .after(PlayerSystems::Travel)
                    .after(View3dSystems::Place),
            );
    }
}
