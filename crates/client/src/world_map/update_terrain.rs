//! Building the window of terrain again as the player moves, and which way light falls.

use bevy::platform::time::Instant;
use bevy::prelude::*;
use shared::celestial::brightest;

use super::rivers::KnownRivers;
use super::terrain::{HeightCache, TerrainGrid, ViewSquare};
use super::{EdgeFaces, FollowsPlayer, HALF_SIZE, MAX_HALF, StripMeshes, Terrain};
use crate::common::{CurrentWorld, GameClock, MapPosition, ViewStats, Weather};
use crate::player::Player;
use crate::surface_style::SurfaceStyle;
use crate::view3d::{OrbitCamera, ViewCamera};

/// When the player reaches a new grid point, the hour, weather or style change,
/// or the view turns: moves the terrain to the player, works out any grid
/// points the window now needs, and cuts the window (lined up with the camera)
/// and its edge faces out of the grid.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_terrain(
    player: Single<Ref<MapPosition>, With<Player>>,
    orbit: Single<&OrbitCamera, With<ViewCamera>>,
    world: Res<CurrentWorld>,
    clock: Res<GameClock>,
    weather: Res<Weather>,
    style: Res<SurfaceStyle>,
    cache: Option<ResMut<HeightCache>>,
    mut grid: ResMut<TerrainGrid>,
    mut known_rivers: ResMut<KnownRivers>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut stats: ResMut<ViewStats>,
    mut followers: Query<&mut Transform, With<FollowsPlayer>>,
    terrain: Single<&Mesh3d, With<Terrain>>,
    faces: Single<&Mesh3d, With<EdgeFaces>>,
    // Together, as Bevy limits how many things a system can ask for.
    (river_strips, river_banks, road_strips): StripMeshes,
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
    // More of the world, which may have pois the woods make way for.
    if !(player.is_changed() || redraw_all || reframed || world.is_changed()) {
        return;
    }
    *cut = Some((orbit.yaw, orbit.distance));

    let start = Instant::now();
    let centre = (player.x, player.y);
    let water_level = world.params.water_level(clock.hour as f64);
    let square = view_square(&orbit);
    let light_from = light_from(&world, clock.hour as f64);
    for mut transform in &mut followers {
        transform.translation = Vec3::new(player.x as f32, 0.0, player.y as f32);
    }
    // First, as woods are cleared round them.
    let clearings_moved = known_rivers.update(
        &world.rivers,
        &world.climate,
        &world.surface,
        &world.pois,
        &world.roads,
        centre,
    );
    grid.fill(
        &mut cache,
        &world.surface,
        &world.climate,
        &world.vegetation,
        &known_rivers,
        &style,
        water_level,
        weather.snow_line,
        light_from,
        centre,
        square,
        redraw_all || clearings_moved,
    );
    let fill_ms = start.elapsed().as_secs_f32() * 1000.0;

    let start = Instant::now();
    if let Some(mut mesh) = meshes.get_mut(&terrain.0) {
        *mesh = super::terrain::build_window(&grid, square);
    }
    let faces_mesh = super::terrain::build_edges(
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
    let (river_mesh, bank_mesh) = super::terrain::build_rivers(
        &grid,
        &known_rivers,
        world.surface.width(),
        &style,
        water_level,
        weather.snow_line,
        square,
    );
    if let Some(mut mesh) = meshes.get_mut(&river_strips.0) {
        *mesh = river_mesh;
    }
    if let Some(mut mesh) = meshes.get_mut(&river_banks.0) {
        *mesh = bank_mesh;
    }
    let road_mesh = super::terrain::build_roads(
        &grid,
        &world.roads,
        &known_rivers,
        world.surface.width(),
        square,
    );
    if let Some(mut mesh) = meshes.get_mut(&road_strips.0) {
        *mesh = road_mesh;
    }
    let cut_ms = start.elapsed().as_secs_f32() * 1000.0;
    stats.summary = format!("World map: fill {fill_ms:.1} ms, cut window {cut_ms:.1} ms");
}

/// Which way along the ground the brightest light is at an hour, `(east, south)`,
/// for the slope shading baked into the terrain: length 1. It moves once an hour,
/// when the terrain is redrawn anyway.
pub(super) fn light_from(world: &CurrentWorld, hours: f64) -> (f64, f64) {
    let bodies = world.celestial.at(hours);
    let [east, _, south] = bodies[brightest(&bodies)].direction;
    let length = (east * east + south * south).sqrt();
    if length < 1e-3 {
        // Straight overhead.
        (1.0, 0.0)
    } else {
        (east / length, south / length)
    }
}

/// The window of terrain on show: lined up with the camera, and smaller when
/// zoomed in, by as much as the camera moved in, so it keeps its size on
/// screen and zooming shows less of the world rather than a bigger map.
pub(super) fn view_square(orbit: &OrbitCamera) -> ViewSquare {
    let (right, towards) = orbit.ground_axes();
    ViewSquare {
        right,
        towards,
        half: window_half(orbit.zoom()),
    }
}

/// Grid units from the player to each side of the window at a camera zoom
/// (`OrbitCamera::zoom`): `HALF_SIZE` in the starting view, never more than
/// `MAX_HALF`.
pub fn window_half(zoom: f32) -> f32 {
    (HALF_SIZE as f32 * zoom).min(MAX_HALF)
}
