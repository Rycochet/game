//! The 3D view shared by the world map and the dungeon view: the orbiting
//! camera and the entity tokens. See `docs/context/client/view3d.md`.
//!
//! The current view (`common::GameView`) draws its own scene and sets
//! `ViewGrid`, which says how big a grid cell is and where the floor is; this
//! module keeps the camera on the player and the tokens on their owners.

mod camera;
mod token;

use bevy::prelude::*;
use bevy::render::view::{DebandDither, Tonemapping};
use bevy::window::PrimaryWindow;

use crate::common::ViewFacing;
use crate::player::{Player, PlayerSystems, SmoothPosition};
pub use camera::OrbitCamera;
pub use token::{DIAMETER as TOKEN_DIAMETER, HALF_THICKNESS as TOKEN_HALF_THICKNESS, TokenView};
use token::{IconAnchor, TokenAssets};

/// Vertical field of view. Narrow (a "long lens" from far away) flattens the
/// perspective, so the map's sides stay closer to vertical.
pub const CAMERA_FOV_DEGREES: f32 = 20.0;

/// Starting camera distance from the point it looks at; with the narrow field
/// of view this frames the world map's whole 400×400 window.
pub const CAMERA_DISTANCE: f32 = 1100.0;

/// Starting camera angle above the horizon.
pub const CAMERA_PITCH_DEGREES: f32 = 34.0;

/// In the starting view, the camera looks this far from the player towards
/// itself (south at first), so the near edge and its face fit on screen.
const LOOK_AHEAD: f32 = 40.0;

pub struct View3dPlugin;

impl Plugin for View3dPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewGrid>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                camera::control_with_mouse.in_set(PlayerSystems::Input),
            )
            .add_systems(
                Update,
                (
                    token::resize_tokens,
                    token::sync_tokens,
                    token::sync_icons,
                    place_camera_and_tokens,
                )
                    .chain()
                    .in_set(View3dSystems::Place)
                    .after(PlayerSystems::Travel),
            );
    }
}

/// `Place` puts the camera and tokens in position each frame. A view that
/// adjusts token heights (the world map rests them on the terrain) runs
/// `.after(View3dSystems::Place)`.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum View3dSystems {
    Place,
}

/// Marks the one camera every 3D view draws through.
#[derive(Component)]
pub struct ViewCamera;

/// How the current view lays out its grid in 3D, set by the view when it
/// becomes current. Grid position `(x, y)` is drawn at
/// `(x × cell_size, floor, y × cell_size)`; the camera looks at the floor and
/// tokens stand on it.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct ViewGrid {
    pub cell_size: f32,
    pub floor: f32,
}

impl Default for ViewGrid {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            floor: 0.0,
        }
    }
}

fn spawn_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    // Starts at the origin; `place_camera_and_tokens` moves it to the player.
    let orbit = OrbitCamera::new(
        CAMERA_PITCH_DEGREES.to_radians(),
        CAMERA_DISTANCE,
        LOOK_AHEAD,
    );
    commands.spawn((
        ViewCamera,
        orbit,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: CAMERA_FOV_DEGREES.to_radians(),
            // The camera is far away, so draw well beyond the default range.
            far: CAMERA_DISTANCE * 2.0,
            ..default()
        }),
        orbit.transform(Vec3::ZERO),
        // The default tonemapping desaturates colours (pastel); show the
        // colours as authored instead.
        Tonemapping::None,
        // Dithering only works with tonemapping.
        DebandDither::Disabled,
    ));

    commands.insert_resource(TokenAssets::new(
        &mut meshes,
        &mut materials,
        window.physical_height() as f32,
        orbit.distance,
    ));
}

/// Every frame: the camera follows the player's exact position, so travelling
/// looks smooth. Tokens stand on the floor at their owners' exact positions,
/// with icon anchors facing the camera. Also tells other modules which way the
/// view is turned (`ViewFacing`).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn place_camera_and_tokens(
    player: Single<&SmoothPosition, With<Player>>,
    grid: Res<ViewGrid>,
    assets: Res<TokenAssets>,
    positions: Query<&SmoothPosition>,
    mut facing: ResMut<ViewFacing>,
    camera: Single<
        (&OrbitCamera, &mut Transform),
        (With<ViewCamera>, Without<TokenView>, Without<IconAnchor>),
    >,
    mut views: Query<(&TokenView, &mut Transform), Without<IconAnchor>>,
    mut anchors: Query<&mut Transform, (With<IconAnchor>, Without<ViewCamera>, Without<TokenView>)>,
) {
    let (orbit, mut camera) = camera.into_inner();
    let size = grid.cell_size;

    let target = Vec3::new(player.x as f32 * size, grid.floor, player.y as f32 * size);
    *camera = orbit.transform(target);
    // Only write when it changes, so change detection stays meaningful.
    facing.set_if_neq(ViewFacing(orbit.quarter_turns()));

    for (view, mut transform) in &mut views {
        let Ok(position) = positions.get(view.owner) else {
            continue;
        };
        transform.translation = Vec3::new(
            position.x as f32 * size,
            grid.floor + token::HALF_THICKNESS,
            position.y as f32 * size,
        );
    }

    // Token views never rotate, so turning each anchor to the camera's
    // rotation makes it face the screen; it sits above the token on screen,
    // whatever the camera's angle.
    for mut anchor in &mut anchors {
        anchor.rotation = camera.rotation;
        anchor.translation = assets.icon_offset(camera.rotation, orbit.pitch, orbit.home_pitch());
    }
}
