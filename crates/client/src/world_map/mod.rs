//! The world map: the surface as a lit 3D terrain mesh, rendered by the GPU.
//! See `docs/context/client/world_map.md`.
//!
//! Like the prototype, it shows a fixed square window of terrain around the
//! player with a flat front face. The window is one mesh that moves with the
//! player; each step reuses cached heights and only samples the new edge.

mod marker;
mod picking;
mod terrain;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::light_consts;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::view::{DebandDither, Tonemapping};
use bevy::window::PrimaryWindow;

use crate::common::{CurrentWorld, GameClock, ViewStats, Weather};
use crate::player::{MapPosition, Player, PlayerSystems, SmoothPosition, TravelTo};
use crate::surface_style::SurfaceStyle;
use terrain::HeightCache;

/// World height units per 3D unit of height. 1.0 keeps the prototype's
/// proportions.
pub const HEIGHT_SCALE: f32 = 1.0;

/// Grid units from the player to each edge of the window (400×400, as the
/// prototype).
pub const HALF_SIZE: i64 = 200;

/// Vertical field of view. Narrow (a "long lens" from far away) flattens the
/// perspective, so the map's sides stay closer to vertical.
pub const CAMERA_FOV_DEGREES: f32 = 20.0;

/// Camera distance from the point it looks at; with the narrow field of view
/// this frames the whole 400×400 window.
pub const CAMERA_DISTANCE: f32 = 1100.0;

/// Camera angle above the horizon, looking north.
const CAMERA_PITCH_DEGREES: f32 = 34.0;

/// The camera looks this far south of the player, so the front edge and its
/// face fit on screen.
const LOOK_SOUTH: f32 = 40.0;

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
        app.add_systems(Startup, spawn_view)
            .add_systems(Update, pick_on_click.in_set(PlayerSystems::Input))
            .add_systems(
                Update,
                (follow_player, place_camera_and_marker)
                    .chain()
                    .after(PlayerSystems::Travel),
            );
    }
}

#[derive(Component)]
struct ViewCamera;

/// Entities that sit at the player's position: terrain and front face.
#[derive(Component)]
struct FollowsPlayer;

#[derive(Component)]
struct Terrain;

#[derive(Component)]
struct FrontFace;

fn spawn_view(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    // Everything starts at the origin; the systems below move it to the player.
    commands.spawn((
        ViewCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: CAMERA_FOV_DEGREES.to_radians(),
            // The camera is far away, so draw well beyond the default range.
            far: CAMERA_DISTANCE * 3.0,
            ..default()
        }),
        Transform::from_translation(camera_offset()).looking_at(Vec3::ZERO, Vec3::Y),
        // The default tonemapping desaturates colours (pastel); show the
        // surface colours as authored instead.
        Tonemapping::None,
        // Dithering only works with tonemapping.
        DebandDither::Disabled,
    ));

    // Low sun from the east, matching the prototype's shading.
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
        Mesh3d(meshes.add(terrain::new_terrain_mesh())),
        // Colours come from the mesh's vertex colours.
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        })),
        // Heights change as the player moves, so the initial bounds go stale.
        NoFrustumCulling,
        Transform::default(),
    ));

    commands.spawn((
        FrontFace,
        FollowsPlayer,
        // Placeholder until the first update builds the real face.
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

    marker::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        window.physical_height() as f32,
    );

    commands.init_resource::<HeightCache>();
}

/// When the player reaches a new grid point (or the hour, weather or style
/// changes), moves the terrain there and rebuilds its heights and front face.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn follow_player(
    player: Single<Ref<MapPosition>, With<Player>>,
    world: Res<CurrentWorld>,
    clock: Res<GameClock>,
    weather: Res<Weather>,
    style: Res<SurfaceStyle>,
    cache: Option<ResMut<HeightCache>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut stats: ResMut<ViewStats>,
    mut followers: Query<&mut Transform, With<FollowsPlayer>>,
    terrain: Single<&Mesh3d, With<Terrain>>,
    front: Single<&Mesh3d, With<FrontFace>>,
) {
    // The cache is inserted by `spawn_view`'s commands, so it can be missing
    // on the very first frame.
    let Some(mut cache) = cache else {
        return;
    };
    let changed =
        player.is_changed() || clock.is_changed() || weather.is_changed() || style.is_changed();
    if !(changed || cache.is_added()) {
        return;
    }

    let start = Instant::now();
    let centre = (player.x, player.y);
    let water_level = world.params.water_level(clock.hour as f64);
    let ground = Vec3::new(player.x as f32, 0.0, player.y as f32);

    for mut transform in &mut followers {
        transform.translation = ground;
    }

    if let Some(mut mesh) = meshes.get_mut(&terrain.0) {
        terrain::update_terrain(
            &mut mesh,
            &mut cache,
            &world.surface,
            &style,
            water_level,
            weather.snow_line,
            centre,
        );
    }
    let front_mesh = terrain::build_front(
        &mut cache,
        &world.surface,
        &style,
        water_level,
        weather.snow_line,
        centre,
    );
    if let Some(mut mesh) = meshes.get_mut(&front.0) {
        *mesh = front_mesh;
    }

    let ms = start.elapsed().as_secs_f32() * 1000.0;
    stats.summary = format!("World map: update {ms:.1} ms");
}

/// Every frame: the camera and marker follow the player's exact position, so
/// travelling looks smooth (the terrain itself moves a whole step at a time).
#[allow(clippy::type_complexity)]
fn place_camera_and_marker(
    player: Single<&SmoothPosition, With<Player>>,
    world: Res<CurrentWorld>,
    clock: Res<GameClock>,
    style: Res<SurfaceStyle>,
    cache: Option<ResMut<HeightCache>>,
    mut camera: Single<&mut Transform, (With<ViewCamera>, Without<marker::Marker>)>,
    mut marker: Single<&mut Transform, With<marker::Marker>>,
) {
    let Some(mut cache) = cache else {
        return;
    };
    let (x, y) = (player.x as f32, player.y as f32);

    // The camera looks at sea level, so it doesn't bob over hills.
    let target = Vec3::new(
        x,
        world.params.sea_level as f32 * HEIGHT_SCALE,
        y + LOOK_SOUTH,
    );
    **camera = Transform::from_translation(target + camera_offset()).looking_at(target, Vec3::Y);

    let water_level = world.params.water_level(clock.hour as f64);
    // Rest on the highest ground under the marker, so it never sinks in.
    let ground = terrain::highest_shown_height(
        &mut cache,
        &world.surface,
        &style,
        water_level,
        (player.x, player.y),
        (marker::DIAMETER / 2.0) as f64,
    );
    marker.translation = Vec3::new(x, ground + marker::HALF_THICKNESS, y);
}

/// A left click on the terrain sends the player travelling there.
#[allow(clippy::too_many_arguments)]
fn pick_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
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
    let (camera, camera_transform) = *camera;
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    let water_level = world.params.water_level(clock.hour as f64);
    if let Some((x, y)) = picking::pick(
        ray,
        (player.x, player.y),
        &mut cache,
        &world.surface,
        &style,
        water_level,
    ) {
        travel.write(TravelTo { x, y });
    }
}

/// Camera position relative to the point it looks at: south and above.
fn camera_offset() -> Vec3 {
    let pitch = CAMERA_PITCH_DEGREES.to_radians();
    Vec3::new(0.0, pitch.sin(), pitch.cos()) * CAMERA_DISTANCE
}
