//! The 3D view shared by the world map and the dungeon view: the orbiting
//! camera and the entity tokens. See `docs/context/client/view3d.md`.
//!
//! The current view (`common::GameView`) draws its own scene and sets
//! `ViewGrid`, which says how big a grid cell is and where the floor is; this
//! module keeps the camera on the player and the tokens on their owners.

mod anti_aliasing;
mod bands;
mod camera;
mod clearance;
mod render_scale;
mod token;

use bevy::prelude::*;
use bevy::render::view::{DebandDither, Tonemapping};
use bevy::window::PrimaryWindow;

use crate::common::{MapPosition, Preferences, SmoothPosition, ViewFacing, WildCellHeights};
use crate::player::{Player, PlayerSystems};
pub use camera::OrbitCamera;
pub use token::{
    ClickedToken, DIAMETER as TOKEN_DIAMETER, HALF_THICKNESS as TOKEN_HALF_THICKNESS, HoveredToken,
    TokenFlash, TokenView,
};
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
            .init_resource::<ClickedToken>()
            .init_resource::<HoveredToken>()
            .configure_sets(Update, View3dSystems::Click.in_set(PlayerSystems::Input))
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    (
                        camera::control_with_mouse,
                        camera::control_with_touch,
                        camera::control_with_keys,
                    )
                        .in_set(PlayerSystems::Input),
                    token::find_token_under_cursor.in_set(View3dSystems::Click),
                    anti_aliasing::apply_anti_aliasing,
                    render_scale::apply_render_scale,
                ),
            )
            .add_systems(
                Update,
                (
                    token::resize_tokens,
                    token::sync_tokens,
                    token::sync_icons,
                    token::flash_tokens,
                    place_camera_and_tokens,
                    bands::drift_bands,
                )
                    .chain()
                    .in_set(View3dSystems::Place)
                    .after(PlayerSystems::Travel),
            );
    }
}

/// `Click` (part of `PlayerSystems::Input`) finds the clicked token
/// (`ClickedToken`); systems handling clicks run after it. `Place` puts the
/// camera and tokens in position each frame. A view that adjusts token
/// heights (the world map rests them on the terrain) runs
/// `.after(View3dSystems::Place)`.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum View3dSystems {
    Click,
    Place,
}

/// Marks the one camera every 3D view draws through.
#[derive(Component)]
pub struct ViewCamera;

/// How the current view lays out its grid in 3D, set by the view when it
/// becomes current. Grid position `(x, y)` is drawn at
/// `(x × cell_size, floor, y × cell_size)`; the camera looks at the floor and
/// tokens stand on it. `raised`: the wilderness's cells are raised by their
/// heights (`common::WildCellHeights`, none on other levels), and so are the
/// camera and tokens.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct ViewGrid {
    pub cell_size: f32,
    pub floor: f32,
    pub raised: bool,
}

impl ViewGrid {
    /// The floor's height at grid position `(x, y)`, in world units.
    pub fn floor_at(&self, heights: &WildCellHeights, x: f32, y: f32) -> f32 {
        match self.raised {
            true => self.floor + heights.drawn_between(x, y) * self.cell_size,
            false => self.floor,
        }
    }
}

impl Default for ViewGrid {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            floor: 0.0,
            raised: false,
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
            // The camera is far away, so draw well beyond the default range:
            // in the Hall of Worlds it looks 70 tiles along, further out.
            far: CAMERA_DISTANCE * 3.0,
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
    player: Single<(Option<&MapPosition>, &SmoothPosition), With<Player>>,
    preferences: Res<Preferences>,
    grid: Res<ViewGrid>,
    heights: Res<WildCellHeights>,
    positions: Query<(Option<&MapPosition>, &SmoothPosition)>,
    mut facing: ResMut<ViewFacing>,
    time: Res<Time>,
    camera: Single<
        (&mut OrbitCamera, &mut Transform),
        (With<ViewCamera>, Without<TokenView>, Without<IconAnchor>),
    >,
    mut views: Query<(&TokenView, &mut Transform), Without<IconAnchor>>,
    mut anchors: Query<(&IconAnchor, &mut Transform), (Without<ViewCamera>, Without<TokenView>)>,
) {
    let (mut orbit, mut camera) = camera.into_inner();
    let size = grid.cell_size;
    // Reduce Motion: on the cell, without gliding between.
    let snap = preferences.video.reduce_motion;
    let shown_at = |(cell, smooth): (Option<&MapPosition>, &SmoothPosition)| match cell {
        Some(cell) if snap => (cell.x as f32, cell.y as f32),
        _ => (smooth.x as f32, smooth.y as f32),
    };

    let (x, y) = shown_at(*player);
    // On raised ground the camera keeps the player's cell where it is on
    // screen, and the land moves under them.
    let target = Vec3::new(x * size, grid.floor_at(&heights, x, y), y * size);
    // Tilted up as far as a clear line to them over that land needs.
    let wanted = match (grid.raised, &heights.heights) {
        (true, Some(known)) => {
            let ground = |cx: i64, cy: i64| {
                let floor = grid.floor + heights.drawn(cx, cy) * size;
                known.covers(cx, cy).then_some(floor)
            };
            clearance::clearing_raise(&orbit, target, size, ground)
        }
        _ => 0.0,
    };
    let raise = clearance::eased(orbit.raise(), wanted, time.delta_secs(), snap);
    orbit.set_raise(raise);
    *camera = orbit.transform(target);
    // Only write when it changes, so change detection stays meaningful.
    facing.set_if_neq(ViewFacing(orbit.quarter_turns()));

    for (view, mut transform) in &mut views {
        let Ok(position) = positions.get(view.owner) else {
            continue;
        };
        let (x, y) = shown_at(position);
        let floor = grid.floor_at(&heights, x, y);
        transform.translation = Vec3::new(x * size, floor + token::HALF_THICKNESS, y * size);
    }

    // Token views never rotate, so turning each anchor about the vertical
    // towards the camera makes it face the screen while standing upright.
    // Scaling the anchor scales its icon by the owner's `TokenSize`.
    let rotation = TokenAssets::icon_rotation(camera.rotation);
    for (anchor, mut transform) in &mut anchors {
        let scale = anchor.scale();
        transform.rotation = rotation;
        transform.scale = Vec3::splat(scale);
        transform.translation = TokenAssets::icon_offset(scale);
    }
}
