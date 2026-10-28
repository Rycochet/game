//! The world map: the surface as a lit 3D terrain mesh, rendered by the GPU.
//! See `docs/context/client/world_map.md`.
//!
//! Like the prototype, it shows a square window of terrain around the player
//! with flat earth faces on its edges. Right-drag turns and tilts the view
//! around the player and the wheel zooms; the window stays lined up with the
//! camera, so the world turns inside it. Home returns to the starting view.
//! The camera and tokens come from `view3d`; this module draws the terrain
//! while `GameView::WorldMap` is current.

mod features;
mod picking;
mod terrain;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::light_consts;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::common::{
    CurrentWorld, GameClock, GameView, MapPosition, SmoothPosition, ViewStats, Weather,
};
use crate::player::{Player, PlayerSystems, TravelTo};
use crate::surface_style::SurfaceStyle;
use crate::view3d::{
    OrbitCamera, TOKEN_DIAMETER, TOKEN_HALF_THICKNESS, TokenView, View3dSystems, ViewCamera,
    ViewGrid,
};
use features::SpawnedFeatures;
use terrain::{HeightCache, TerrainGrid, ViewSquare};

/// World height units per 3D unit of height. 1.0 keeps the prototype's
/// proportions.
pub const HEIGHT_SCALE: f32 = 1.0;

/// Grid units from the player to each edge of the window (400×400, as the
/// prototype).
pub const HALF_SIZE: i64 = 200;

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
        let in_view = in_state(GameView::WorldMap);
        app.init_resource::<TerrainGrid>()
            .init_resource::<SpawnedFeatures>()
            .add_systems(Startup, spawn_view)
            .add_systems(OnEnter(GameView::WorldMap), show_scene)
            .add_systems(
                OnExit(GameView::WorldMap),
                (hide_scene, features::despawn_all),
            )
            .add_systems(
                Update,
                pick_on_click
                    .in_set(PlayerSystems::Input)
                    .run_if(in_view.clone()),
            )
            .add_systems(
                Update,
                (update_terrain, features::spawn_nearby)
                    .after(PlayerSystems::Travel)
                    .before(View3dSystems::Place)
                    .run_if(in_view.clone()),
            )
            .add_systems(
                Update,
                (rest_tokens_on_terrain, hide_tokens_outside_window)
                    .after(View3dSystems::Place)
                    .run_if(in_view),
            );
    }
}

/// Everything this view draws (terrain, edge faces, sun), hidden while
/// another view is current.
#[derive(Component)]
struct WorldMapScene;

/// Entities that sit at the player's position: terrain and edge faces.
#[derive(Component)]
struct FollowsPlayer;

#[derive(Component)]
struct Terrain;

#[derive(Component)]
struct EdgeFaces;

/// Spawns the scene, hidden unless this is the current view.
fn spawn_view(
    mut commands: Commands,
    view: Res<State<GameView>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let visibility = if *view.get() == GameView::WorldMap {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    // Everything starts at the origin; `update_terrain` moves it to the player.
    // Low sun from the east, matching the prototype's shading. It stays in the
    // east when the view turns, like the slope shading baked into the terrain.
    commands.spawn((
        WorldMapScene,
        DirectionalLight {
            illuminance: SUN_LUX,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(1.0, 0.6, 0.3).looking_at(Vec3::ZERO, Vec3::Y),
        visibility,
    ));

    commands.spawn((
        WorldMapScene,
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
        visibility,
    ));

    commands.spawn((
        WorldMapScene,
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
        visibility,
    ));

    commands.init_resource::<HeightCache>();
}

/// Becoming the current view: shows the scene, sets the sun's ambient fill,
/// and tells `view3d` the grid is 1 unit per point with the camera looking at
/// sea level, so it doesn't bob over hills.
fn show_scene(
    mut commands: Commands,
    world: Res<CurrentWorld>,
    mut scene: Query<&mut Visibility, With<WorldMapScene>>,
) {
    for mut visibility in &mut scene {
        *visibility = Visibility::Inherited;
    }
    commands.insert_resource(GlobalAmbientLight {
        brightness: AMBIENT_BRIGHTNESS,
        ..default()
    });
    commands.insert_resource(ViewGrid {
        cell_size: 1.0,
        floor: world.params.sea_level as f32 * HEIGHT_SCALE,
    });
}

/// Leaving the view: hides the scene, and shows every token again (other
/// views draw all of them).
fn hide_scene(
    mut scene: Query<&mut Visibility, (With<WorldMapScene>, Without<TokenView>)>,
    mut tokens: Query<&mut Visibility, With<TokenView>>,
) {
    for mut visibility in &mut scene {
        *visibility = Visibility::Hidden;
    }
    for mut visibility in &mut tokens {
        *visibility = Visibility::Inherited;
    }
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
    // `Local` keeps a value between runs of this system: the square last cut.
    mut cut: Local<Option<(f32, f32)>>,
) {
    // The cache is inserted by `spawn_view`'s commands, so it can be missing
    // on the very first frame.
    let Some(mut cache) = cache else {
        return;
    };
    // A new hour, weather or style changes how every point is drawn.
    let redraw_all =
        clock.is_changed() || weather.is_changed() || style.is_changed() || cache.is_added();
    // Turning or zooming changes the square.
    let reframed = *cut != Some((orbit.yaw, orbit.distance));
    if !(player.is_changed() || redraw_all || reframed) {
        return;
    }
    *cut = Some((orbit.yaw, orbit.distance));

    let start = Instant::now();
    let centre = (player.x, player.y);
    let water_level = world.params.water_level(clock.hour as f64);
    let square = view_square(&orbit);
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
        square,
        redraw_all,
    );
    let fill_ms = start.elapsed().as_secs_f32() * 1000.0;

    let start = Instant::now();
    if let Some(mut mesh) = meshes.get_mut(&terrain.0) {
        *mesh = terrain::build_window(&grid, square);
    }
    let faces_mesh = terrain::build_edges(
        &grid,
        &mut cache,
        &world.surface,
        &style,
        water_level,
        weather.snow_line,
        centre,
        square,
    );
    if let Some(mut mesh) = meshes.get_mut(&faces.0) {
        *mesh = faces_mesh;
    }
    let cut_ms = start.elapsed().as_secs_f32() * 1000.0;
    stats.summary = format!("World map: fill {fill_ms:.1} ms, cut window {cut_ms:.1} ms");
}

/// Every frame, after `view3d` has placed the tokens: lifts each one onto the
/// highest ground under it, so it never sinks into a slope.
fn rest_tokens_on_terrain(
    world: Res<CurrentWorld>,
    clock: Res<GameClock>,
    style: Res<SurfaceStyle>,
    cache: Option<ResMut<HeightCache>>,
    positions: Query<&SmoothPosition>,
    mut views: Query<(&TokenView, &mut Transform)>,
) {
    let Some(mut cache) = cache else {
        return;
    };
    let water_level = world.params.water_level(clock.hour as f64);
    for (view, mut transform) in &mut views {
        let Ok(position) = positions.get(view.owner) else {
            continue;
        };
        let ground = terrain::highest_shown_height(
            &mut cache,
            &world.surface,
            &style,
            water_level,
            (position.x, position.y),
            (TOKEN_DIAMETER / 2.0) as f64,
        );
        transform.translation.y = ground + TOKEN_HALF_THICKNESS;
    }
}

/// Every frame: hides tokens that aren't wholly inside the terrain window,
/// so none hang over its edges or float beyond them.
fn hide_tokens_outside_window(
    player: Single<&MapPosition, With<Player>>,
    orbit: Single<&OrbitCamera, With<ViewCamera>>,
    positions: Query<&SmoothPosition>,
    mut views: Query<(&TokenView, &mut Visibility)>,
) {
    let square = view_square(&orbit);
    let inner = ViewSquare {
        half: square.half - TOKEN_DIAMETER / 2.0,
        ..square
    };
    for (view, mut visibility) in &mut views {
        let Ok(position) = positions.get(view.owner) else {
            continue;
        };
        // Relative to the window's centre, the player's grid point.
        let offset = Vec2::new(
            (position.x - player.x as f64) as f32,
            (position.y - player.y as f64) as f32,
        );
        visibility.set_if_neq(if inner.contains(offset) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
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
        view_square(orbit),
        &mut cache,
        &world.surface,
        &style,
        water_level,
    ) {
        travel.write(TravelTo { x, y });
    }
}

/// The window of terrain on show: lined up with the camera, and smaller when
/// zoomed in, by as much as the camera moved in, so it keeps its size on
/// screen and zooming shows less of the world rather than a bigger map.
fn view_square(orbit: &OrbitCamera) -> ViewSquare {
    let (right, towards) = orbit.ground_axes();
    ViewSquare {
        right,
        towards,
        half: HALF_SIZE as f32 * orbit.zoom(),
    }
}
