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
mod grid;
mod light_clusters;
mod render_scale;
mod token;

use bevy::prelude::*;
use bevy::render::view::{DebandDither, Tonemapping};
use bevy::window::PrimaryWindow;

use crate::common::{DrawnGround, MapPosition, Preferences, SmoothPosition, ViewFacing, ViewTurn};
use crate::player::{Player, PlayerSystems};
pub use camera::OrbitCamera;
pub use grid::ViewGrid;
use token::{BaseTilt, IconAnchor, ModelAnchor, TokenAssets, TokenBase};
pub use token::{
    ClickedToken, DIAMETER as TOKEN_DIAMETER, HALF_THICKNESS as TOKEN_HALF_THICKNESS, HoveredToken,
    TokenFlash, TokenView, XRAY_WHITE, XrayMaterial,
};

/// Vertical field of view. Narrow (a "long lens" from far away) flattens the
/// perspective, so the map's sides stay closer to vertical.
pub const CAMERA_FOV_DEGREES: f32 = 20.0;

/// Starting camera distance from the point it looks at; with the narrow field
/// of view this frames the world map's whole 400×400 window.
pub const CAMERA_DISTANCE: f32 = 1100.0;

/// How far the camera draws: well beyond the default range, as it is far
/// away (in the Hall of Worlds it looks 70 tiles along, further out). Its
/// light slices end here too (`light_clusters`).
const DRAW_DISTANCE: f32 = CAMERA_DISTANCE * 3.0;

/// Starting camera angle above the horizon.
pub const CAMERA_PITCH_DEGREES: f32 = 34.0;

/// In the starting view, the camera looks this far from the player towards
/// itself (south at first), so the near edge and its face fit on screen.
const LOOK_AHEAD: f32 = 40.0;

pub struct View3dPlugin;

impl Plugin for View3dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<XrayMaterial>::default())
            .add_plugins(MaterialPlugin::<token::HullMaterial>::default())
            .add_plugins(MaterialPlugin::<crate::common::ModelMaterial>::default())
            .init_resource::<token::HullMaterials>()
            .init_resource::<ViewGrid>()
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
                    token::grow_hulls,
                    token::sync_tokens,
                    token::sync_icons,
                    token::sync_models,
                    token::flash_tokens,
                    place_camera_and_tokens,
                    token::turn_models,
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

fn spawn_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut xray: ResMut<Assets<XrayMaterial>>,
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
            far: DRAW_DISTANCE,
            ..default()
        }),
        light_clusters::light_clusters(DRAW_DISTANCE),
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
        &mut xray,
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
    heights: Res<DrawnGround>,
    positions: Query<(Option<&MapPosition>, &SmoothPosition)>,
    (mut facing, mut turn): (ResMut<ViewFacing>, ResMut<ViewTurn>),
    time: Res<Time>,
    camera: Single<
        (&mut OrbitCamera, &mut Transform),
        (
            With<ViewCamera>,
            Without<TokenView>,
            Without<IconAnchor>,
            Without<ModelAnchor>,
        ),
    >,
    mut views: Query<
        (&mut TokenView, &mut Transform),
        (
            Without<IconAnchor>,
            Without<ModelAnchor>,
            Without<TokenBase>,
        ),
    >,
    mut anchors: Query<
        (&IconAnchor, &mut Transform),
        (
            Without<ViewCamera>,
            Without<TokenView>,
            Without<ModelAnchor>,
            Without<TokenBase>,
        ),
    >,
    mut models: Query<
        (&ModelAnchor, &mut Transform),
        (Without<ViewCamera>, Without<TokenView>, Without<TokenBase>),
    >,
    mut bases: Query<
        &mut Transform,
        (
            With<TokenBase>,
            Without<ViewCamera>,
            Without<TokenView>,
            Without<IconAnchor>,
            Without<ModelAnchor>,
        ),
    >,
) {
    let (mut orbit, mut camera) = camera.into_inner();
    let size = grid.cell_size;
    // Reduce Motion: on the cell, without gliding between.
    let snap = preferences.graphics.reduce_motion;
    let shown_at = |(cell, smooth): (Option<&MapPosition>, &SmoothPosition)| match cell {
        Some(cell) if snap => (cell.x as f32, cell.y as f32),
        _ => (smooth.x as f32, smooth.y as f32),
    };

    let (x, y) = shown_at(*player);
    // On raised ground the camera keeps the player's cell where it is on
    // screen, and the land moves under them.
    let target = Vec3::new(x * size, grid.floor_at(&heights, x, y), y * size);
    // Tilted up as far as a clear line to them over that land needs.
    let wanted = match grid.raised && heights.slopes() {
        true => {
            let ground = |cx: i64, cy: i64| {
                let floor = grid.floor + heights.drawn(cx, cy) * size;
                heights.covers(cx, cy).then_some(floor)
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
    // The camera turning east (positive yaw) puts west at the top.
    turn.set_if_neq(ViewTurn(-orbit.yaw));

    // Token views never rotate, so turning each icon anchor about the
    // vertical towards the camera makes it face the screen while standing
    // upright. Scaling the anchor scales its icon by the owner's `TokenSize`.
    let rotation = TokenAssets::icon_rotation(camera.rotation);
    let half = token::HALF_THICKNESS;
    for (mut view, mut transform) in &mut views {
        let Ok(position) = positions.get(view.owner) else {
            continue;
        };
        let (x, y) = shown_at(position);
        // Each base lies on the ground under it, tilted where it slopes.
        let tilt = match grid.raised && heights.slopes() {
            true => BaseTilt::fit(view.radius, half, |dx, dz| {
                grid.floor_at(&heights, x + dx / size, y + dz / size)
            }),
            false => BaseTilt::flat(grid.floor_at(&heights, x, y)),
        };
        // The root at the base's middle; what stands on it stands on the
        // middle of its top, `up` from there.
        let up = tilt.up(half);
        transform.translation = Vec3::new(x * size, tilt.bottom, y * size) + up;
        view.tilt = tilt.rotation();
        if let Ok(mut base) = bases.get_mut(view.base) {
            base.rotation = view.tilt;
        }
        // The icon or model stays upright, its feet on the top's middle.
        if let Ok((anchor, mut transform)) = anchors.get_mut(view.anchor) {
            let scale = anchor.scale();
            transform.rotation = rotation;
            transform.scale = Vec3::splat(scale);
            transform.translation = TokenAssets::icon_offset(scale) + up - Vec3::Y * half;
        }
        // Turned by `turn_models`.
        if let Ok((_, mut transform)) = models.get_mut(view.model) {
            transform.translation = up;
            transform.scale = Vec3::splat(token::MODEL_SCALE);
        }
    }
}
