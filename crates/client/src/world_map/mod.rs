//! The world map: the surface as a lit 3D terrain mesh, rendered by the GPU.
//! See `docs/context/client/world_map.md`.
//!
//! Like the prototype, it shows a square window of terrain around the player
//! with flat earth faces on its edges. Right-drag turns and tilts the view
//! around the player and the wheel zooms; the window stays lined up with the
//! camera, so the world turns inside it. Home returns to the starting view.

mod camera;
mod picking;
mod terrain;
mod token;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::light_consts;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::view::{DebandDither, Tonemapping};
use bevy::window::PrimaryWindow;

use crate::common::{CurrentWorld, GameClock, ViewFacing, ViewStats, Weather};
use crate::player::{MapPosition, Player, PlayerSystems, SmoothPosition, TravelTo};
use crate::surface_style::SurfaceStyle;
pub use camera::OrbitCamera;
use terrain::{HeightCache, TerrainGrid};
use token::{IconAnchor, TokenAssets, TokenView};

/// World height units per 3D unit of height. 1.0 keeps the prototype's
/// proportions.
pub const HEIGHT_SCALE: f32 = 1.0;

/// Grid units from the player to each edge of the window (400×400, as the
/// prototype).
pub const HALF_SIZE: i64 = 200;

/// Vertical field of view. Narrow (a "long lens" from far away) flattens the
/// perspective, so the map's sides stay closer to vertical.
pub const CAMERA_FOV_DEGREES: f32 = 20.0;

/// Starting camera distance from the point it looks at; with the narrow field
/// of view this frames the whole 400×400 window.
pub const CAMERA_DISTANCE: f32 = 1100.0;

/// Starting camera angle above the horizon.
pub const CAMERA_PITCH_DEGREES: f32 = 34.0;

/// In the starting view, the camera looks this far from the player towards
/// itself (south at first), so the near edge and its face fit on screen.
const LOOK_AHEAD: f32 = 40.0;

/// How much of the prototype's slope shading is baked into the terrain
/// colours (1.0 = the same as the prototype).
pub const SLOPE_SHADING: f64 = 1.0;

/// A harsh, spotlight-like sun with little ambient fill, so slopes facing away
/// go properly dark. Tuned so overall brightness is close to the prototype.
const SUN_LUX: f32 = light_consts::lux::FULL_DAYLIGHT * 0.2;
const AMBIENT_BRIGHTNESS: f32 = 60.0;

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainGrid>()
            .add_systems(Startup, spawn_view)
            .add_systems(
                Update,
                (camera::control_with_mouse, pick_on_click)
                    .chain()
                    .in_set(PlayerSystems::Input),
            )
            .add_systems(
                Update,
                (
                    update_terrain,
                    token::resize_tokens,
                    token::sync_tokens,
                    token::sync_icons,
                    place_camera_and_tokens,
                )
                    .chain()
                    .after(PlayerSystems::Travel),
            );
    }
}

#[derive(Component)]
struct ViewCamera;

/// Entities that sit at the player's position: terrain and edge faces.
#[derive(Component)]
struct FollowsPlayer;

#[derive(Component)]
struct Terrain;

#[derive(Component)]
struct EdgeFaces;

fn spawn_view(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    // Everything starts at the origin; the systems below move it to the player.
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
            // The camera is far away, so draw well beyond the default range,
            // even when zoomed right out.
            far: orbit.max_distance() * 2.0,
            ..default()
        }),
        orbit.transform(Vec3::ZERO),
        // The default tonemapping desaturates colours (pastel); show the
        // surface colours as authored instead.
        Tonemapping::None,
        // Dithering only works with tonemapping.
        DebandDither::Disabled,
    ));

    // Low sun from the east, matching the prototype's shading. It stays in the
    // east when the view turns, like the slope shading baked into the terrain.
    commands.spawn((
        DirectionalLight {
            illuminance: SUN_LUX,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(1.0, 0.6, 0.3).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(GlobalAmbientLight {
        brightness: AMBIENT_BRIGHTNESS,
        ..default()
    });

    commands.spawn((
        Terrain,
        FollowsPlayer,
        // Placeholder until the first update cuts the real window.
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        // Colours come from the mesh's vertex colours.
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        })),
        // The mesh is replaced as the player moves, so its bounds go stale.
        NoFrustumCulling,
        Transform::default(),
    ));

    commands.spawn((
        EdgeFaces,
        FollowsPlayer,
        // Placeholder until the first update builds the real faces.
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        // Unlit, so the colours match the prototype exactly.
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        })),
        NoFrustumCulling,
        Transform::default(),
    ));

    commands.insert_resource(TokenAssets::new(
        &mut meshes,
        &mut materials,
        window.physical_height() as f32,
        orbit.distance,
    ));

    commands.init_resource::<HeightCache>();
}

/// When the player reaches a new grid point, the hour, weather or style change,
/// or the view turns: moves the terrain to the player, works out any grid
/// points the window now needs, and cuts the window (lined up with the camera)
/// and its edge faces out of the grid.
#[allow(clippy::too_many_arguments)]
fn update_terrain(
    player: Single<Ref<MapPosition>, With<Player>>,
    orbit: Single<&OrbitCamera, With<ViewCamera>>,
    world: Res<CurrentWorld>,
    clock: Res<GameClock>,
    weather: Res<Weather>,
    style: Res<SurfaceStyle>,
    cache: Option<ResMut<HeightCache>>,
    mut grid: ResMut<TerrainGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut stats: ResMut<ViewStats>,
    mut followers: Query<&mut Transform, With<FollowsPlayer>>,
    terrain: Single<&Mesh3d, With<Terrain>>,
    faces: Single<&Mesh3d, With<EdgeFaces>>,
    // `Local` keeps a value between runs of this system: the yaw last cut.
    mut cut_yaw: Local<Option<f32>>,
) {
    // The cache is inserted by `spawn_view`'s commands, so it can be missing
    // on the very first frame.
    let Some(mut cache) = cache else {
        return;
    };
    // A new hour, weather or style changes how every point is drawn.
    let redraw_all =
        clock.is_changed() || weather.is_changed() || style.is_changed() || cache.is_added();
    let turned = *cut_yaw != Some(orbit.yaw);
    if !(player.is_changed() || redraw_all || turned) {
        return;
    }
    *cut_yaw = Some(orbit.yaw);

    let start = Instant::now();
    let centre = (player.x, player.y);
    let water_level = world.params.water_level(clock.hour as f64);
    let axes = orbit.ground_axes();
    for mut transform in &mut followers {
        transform.translation = Vec3::new(player.x as f32, 0.0, player.y as f32);
    }
    grid.fill(
        &mut cache,
        &world.surface,
        &style,
        water_level,
        weather.snow_line,
        centre,
        axes,
        redraw_all,
    );
    let fill_ms = start.elapsed().as_secs_f32() * 1000.0;

    let start = Instant::now();
    if let Some(mut mesh) = meshes.get_mut(&terrain.0) {
        *mesh = terrain::build_window(&grid, axes);
    }
    let faces_mesh = terrain::build_edges(
        &grid,
        &mut cache,
        &world.surface,
        &style,
        water_level,
        weather.snow_line,
        centre,
        axes,
    );
    if let Some(mut mesh) = meshes.get_mut(&faces.0) {
        *mesh = faces_mesh;
    }
    let cut_ms = start.elapsed().as_secs_f32() * 1000.0;
    stats.summary = format!("World map: fill {fill_ms:.1} ms, cut window {cut_ms:.1} ms");
}

/// Every frame: the camera follows the player's exact position, so travelling
/// looks smooth (the terrain itself moves a whole step at a time). Tokens sit
/// at their owners' exact positions, with icon anchors facing the camera. Also
/// tells other modules which way the view is turned (`ViewFacing`).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn place_camera_and_tokens(
    player: Single<&SmoothPosition, With<Player>>,
    world: Res<CurrentWorld>,
    clock: Res<GameClock>,
    style: Res<SurfaceStyle>,
    cache: Option<ResMut<HeightCache>>,
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
    let Some(mut cache) = cache else {
        return;
    };
    let (orbit, mut camera) = camera.into_inner();

    // The camera looks at sea level, so it doesn't bob over hills.
    let target = Vec3::new(
        player.x as f32,
        world.params.sea_level as f32 * HEIGHT_SCALE,
        player.y as f32,
    );
    *camera = orbit.transform(target);
    // Only write when it changes, so change detection stays meaningful.
    facing.set_if_neq(ViewFacing(orbit.quarter_turns()));

    let water_level = world.params.water_level(clock.hour as f64);
    for (view, mut transform) in &mut views {
        let Ok(position) = positions.get(view.owner) else {
            continue;
        };
        // Rest on the highest ground under the token, so it never sinks in.
        let ground = terrain::highest_shown_height(
            &mut cache,
            &world.surface,
            &style,
            water_level,
            (position.x, position.y),
            (token::DIAMETER / 2.0) as f64,
        );
        transform.translation = Vec3::new(
            position.x as f32,
            ground + token::HALF_THICKNESS,
            position.y as f32,
        );
    }

    // Token views never rotate, so turning each anchor to the camera's
    // rotation makes it face the screen; it sits above the token on screen,
    // whatever the camera's angle.
    for mut anchor in &mut anchors {
        anchor.rotation = camera.rotation;
        anchor.translation = assets.icon_offset(camera.rotation);
    }
}

/// A left click on the terrain sends the player travelling there.
#[allow(clippy::too_many_arguments)]
fn pick_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform, &OrbitCamera), With<ViewCamera>>,
    player: Single<&MapPosition, With<Player>>,
    world: Res<CurrentWorld>,
    clock: Res<GameClock>,
    style: Res<SurfaceStyle>,
    cache: Option<ResMut<HeightCache>>,
    mut travel: MessageWriter<TravelTo>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let (Some(cursor), Some(mut cache)) = (window.cursor_position(), cache) else {
        return;
    };
    let (camera, camera_transform, orbit) = *camera;
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    let water_level = world.params.water_level(clock.hour as f64);
    if let Some((x, y)) = picking::pick(
        ray,
        (player.x, player.y),
        orbit.ground_axes(),
        &mut cache,
        &world.surface,
        &style,
        water_level,
    ) {
        travel.write(TravelTo { x, y });
    }
}
