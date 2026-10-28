//! The mesh view: the surface as a lit 3D terrain mesh, rendered by the GPU.
//! See `docs/context/client/mesh_view.md`.
//!
//! Like the slice view, it shows a fixed square window of terrain around the
//! player with a flat front face. The window is one mesh that moves with the
//! player; each step reuses cached heights and only samples the new edge.

mod terrain;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::light_consts;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::view::Tonemapping;

use crate::common::{CurrentWorld, GameClock, MapView, ViewStats, Weather};
use crate::player::{MapPosition, Player};
use crate::surface_style::SurfaceStyle;
use terrain::HeightCache;

/// World height units per 3D unit of height. 1.0 keeps the slice view's
/// proportions.
pub const HEIGHT_SCALE: f32 = 1.0;

/// Grid units from the player to each edge of the window (400×400, as the
/// slice view).
pub const HALF_SIZE: i64 = 200;

/// Vertical field of view. Narrow (a "long lens" from far away) flattens the
/// perspective, so the map's sides stay closer to vertical.
const CAMERA_FOV_DEGREES: f32 = 20.0;

/// Camera distance from the point it looks at; with the narrow field of view
/// this frames the whole 400×400 window.
const CAMERA_DISTANCE: f32 = 1100.0;

/// Camera angle above the horizon, looking north.
const CAMERA_PITCH_DEGREES: f32 = 34.0;

/// The camera looks this far south of the player, so the front edge and its
/// face fit on screen.
const LOOK_SOUTH: f32 = 40.0;

/// How much of the slice view's slope shading is baked into the terrain
/// colours (1.0 = the same as the slice view).
pub const SLOPE_SHADING: f64 = 1.0;

/// A harsh, spotlight-like sun with little ambient fill, so slopes facing away
/// go properly dark. Tuned so overall brightness is close to the slice view.
const SUN_LUX: f32 = light_consts::lux::FULL_DAYLIGHT * 0.2;
const AMBIENT_BRIGHTNESS: f32 = 60.0;

pub struct MeshViewPlugin;

impl Plugin for MeshViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MapView::Mesh), spawn_view)
            .add_systems(OnExit(MapView::Mesh), |mut commands: Commands| {
                commands.remove_resource::<HeightCache>();
            })
            .add_systems(Update, follow_player.run_if(in_state(MapView::Mesh)));
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
) {
    // Everything starts at the origin; `follow_player` moves it to the player
    // (who may not exist yet if the game starts in this view).
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
        DespawnOnExit(MapView::Mesh),
    ));

    // Low sun from the east, matching the slice view's shading.
    commands.spawn((
        DirectionalLight {
            illuminance: SUN_LUX,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(1.0, 0.6, 0.3).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(MapView::Mesh),
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
        DespawnOnExit(MapView::Mesh),
    ));

    commands.spawn((
        FrontFace,
        FollowsPlayer,
        // Placeholder until the first update builds the real face.
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        // Unlit, so the colours match the slice view exactly.
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        })),
        NoFrustumCulling,
        Transform::default(),
        DespawnOnExit(MapView::Mesh),
    ));

    commands.init_resource::<HeightCache>();
}

/// When the player moves (or the hour changes), moves everything to the new
/// position and rebuilds the terrain heights and front face.
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
    mut followers: Query<&mut Transform, (With<FollowsPlayer>, Without<ViewCamera>)>,
    mut camera: Single<&mut Transform, With<ViewCamera>>,
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

    // The camera looks at sea level, so it doesn't bob over hills.
    let target = ground.with_y(world.params.sea_level as f32 * HEIGHT_SCALE) + Vec3::Z * LOOK_SOUTH;
    **camera = Transform::from_translation(target + camera_offset()).looking_at(target, Vec3::Y);

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
    stats.summary = format!("Mesh view: update {ms:.1} ms");
}

/// Camera position relative to the point it looks at: south and above.
fn camera_offset() -> Vec3 {
    let pitch = CAMERA_PITCH_DEGREES.to_radians();
    Vec3::new(0.0, pitch.sin(), pitch.cos()) * CAMERA_DISTANCE
}
