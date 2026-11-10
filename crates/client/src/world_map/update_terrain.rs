//! Building the window of terrain again as the player moves, and which way light falls.

use bevy::platform::time::Instant;
use bevy::prelude::*;
use shared::celestial::brightest;

use super::rivers::KnownRivers;
use super::terrain::{HeightCache, TerrainGrid, ViewSquare};
use super::{EdgeFaces, FollowsPlayer, HALF_SIZE, MAX_HALF, StripMeshes};
use crate::common::{CurrentWorld, GameClock, MapPosition, Preferences, ViewDistance, ViewStats};
use crate::player::Player;
use crate::surface_style::SurfaceStyle;
use crate::view3d::{OrbitCamera, ViewCamera};

/// The window last cut: its turn and size, and whether lit at noon.
type LastCut = (f32, f32, bool);

/// When the player reaches a new grid point, the hour or style change,
/// or the view turns: moves the edge faces to the player, works out any grid
/// points the window's chunks now need (`chunks::update_chunks` builds them),
/// and cuts the edge faces, rivers and roads to the window.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_terrain(
    player: Single<Ref<MapPosition>, With<Player>>,
    orbit: Single<&OrbitCamera, With<ViewCamera>>,
    world: Res<CurrentWorld>,
    clock: Res<GameClock>,
    style: Res<SurfaceStyle>,
    cache: Option<ResMut<HeightCache>>,
    mut grid: ResMut<TerrainGrid>,
    mut known_rivers: ResMut<KnownRivers>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut stats: ResMut<ViewStats>,
    mut followers: Query<&mut Transform, With<FollowsPlayer>>,
    faces: Single<&Mesh3d, With<EdgeFaces>>,
    // Together, as Bevy limits how many things a system can ask for.
    (river_strips, river_banks, road_strips): StripMeshes,
    // `Local` keeps a value between runs of this system: the square last cut
    // (its turn and size), and whether lit at noon (Midday Sun).
    (preferences, mut cut): (Res<Preferences>, Local<Option<LastCut>>),
) {
    // The cache is inserted by `spawn_view`'s commands, so it can be missing
    // on the very first frame.
    let Some(mut cache) = cache else {
        return;
    };
    let graphics = &preferences.graphics;
    let square = view_square(&orbit, graphics.view_distance);
    // Turning or zooming changes the square.
    let reframed = cut.is_none_or(|(yaw, half, _)| (yaw, half) != (orbit.yaw, square.half));
    let relit = cut.is_some_and(|(.., noon)| noon != graphics.midday_sun);
    // A new hour (for the tide), style or light changes how every
    // point is drawn.
    let redraw_all = clock.is_changed() || style.is_changed() || cache.is_added() || relit;
    // More of the world, which may have pois the woods make way for.
    if !(player.is_changed() || redraw_all || reframed || world.is_changed()) {
        return;
    }
    *cut = Some((orbit.yaw, square.half, graphics.midday_sun));

    let start = Instant::now();
    let centre = (player.x, player.y);
    let water_level = world.water_level(&clock);
    let hour = super::map_sky::shading_hour(graphics.midday_sun, clock.hour as f64);
    let light_from = light_from(&world, hour);
    for mut transform in &mut followers {
        transform.translation = Vec3::new(player.x as f32, 0.0, player.y as f32);
    }
    // First, as woods are cleared round them.
    let clearings_moved = known_rivers.update(
        &world.rivers,
        &world.climate,
        &world.surface,
        &world.pois,
        &world.gates,
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
        world.surface.floor(),
        light_from,
        centre,
        // Whole chunks, as `chunks::update_chunks` builds them.
        &super::terrain::chunks_needed(square, centre)
            .iter()
            .map(|&chunk| super::terrain::chunks_area(&[chunk]))
            .collect::<Vec<_>>(),
        redraw_all || clearings_moved,
    );
    let fill_ms = start.elapsed().as_secs_f32() * 1000.0;

    let start = Instant::now();
    let faces_mesh = super::terrain::build_edges(
        &grid,
        &mut cache,
        &world.surface,
        &world.climate,
        &style,
        water_level,
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
    stats.summary = format!("World map: fill {fill_ms:.1} ms, cut edges {cut_ms:.1} ms");
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
pub(super) fn view_square(orbit: &OrbitCamera, distance: ViewDistance) -> ViewSquare {
    let (right, towards) = orbit.ground_axes();
    ViewSquare {
        right,
        towards,
        half: drawn_half(orbit.zoom(), distance),
    }
}

/// How far the window's sides are from the player with the View Distance
/// setting: Normal as `window_half`, Near three quarters of that, Far all the
/// window there can be, even zoomed in.
pub(super) fn drawn_half(zoom: f32, distance: ViewDistance) -> f32 {
    match distance {
        ViewDistance::Near => window_half(zoom) * NEAR_SHARE,
        ViewDistance::Normal => window_half(zoom),
        ViewDistance::Far => MAX_HALF,
    }
}

/// How much of the usual window Near draws.
const NEAR_SHARE: f32 = 0.75;

/// Grid units from the player to each side of the window at a camera zoom
/// (`OrbitCamera::zoom`): `HALF_SIZE` in the starting view, never more than
/// `MAX_HALF`.
pub fn window_half(zoom: f32) -> f32 {
    (HALF_SIZE as f32 * zoom).min(MAX_HALF)
}
