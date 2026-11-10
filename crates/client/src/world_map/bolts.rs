//! Bolts of lightning on the world map, striking where the server says, if
//! inside the window: from the clouds down to the drawn ground.

use bevy::prelude::*;
use shared::wilderness::SCALE;

use super::rivers::KnownRivers;
use super::terrain::{HeightCache, ViewSquare, shown_height_at};
use super::{WorldMapScene, view_square};
use crate::common::{BoltStruck, CurrentWorld, GameClock, MapPosition, Preferences};
use crate::lightning::spawn_bolt;
use crate::player::Player;
use crate::surface_style::SurfaceStyle;
use crate::view3d::{OrbitCamera, ViewCamera};

/// How wide a bolt's main stroke is, in grid units: a few pixels at the
/// starting zoom, as wide as a broad river.
const BOLT_WIDTH: f32 = 1.5;

/// The least a bolt is tall, where the land comes near the clouds.
const LEAST_HEIGHT: f32 = 30.0;

/// Draws each bolt struck, if inside the window.
#[allow(clippy::too_many_arguments)]
pub(super) fn strike(
    mut struck: MessageReader<BoltStruck>,
    mut commands: Commands,
    (world, clock, style, known_rivers): (
        Res<CurrentWorld>,
        Res<GameClock>,
        Res<SurfaceStyle>,
        Res<KnownRivers>,
    ),
    cache: Option<ResMut<HeightCache>>,
    player: Single<&MapPosition, With<Player>>,
    orbit: Single<&OrbitCamera, With<ViewCamera>>,
    preferences: Res<Preferences>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(mut cache) = cache else {
        struck.clear();
        return;
    };
    let square: ViewSquare = view_square(&orbit, preferences.graphics.view_distance);
    let width = world.surface.width() as f32;
    for BoltStruck(strike) in struck.read() {
        // Cells to grid points: a point's middle cell is the point itself.
        let to_point = |cell: i64| (cell - SCALE / 2) as f32 / SCALE as f32;
        // East or west of the player, the nearer way round the planet.
        let east = (to_point(strike.x) - player.x as f32).rem_euclid(width);
        let east = if east > width / 2.0 {
            east - width
        } else {
            east
        };
        let south = to_point(strike.y) - player.y as f32;
        if !square.contains(Vec2::new(east, south)) {
            continue;
        }
        let (x, z) = (player.x as f32 + east, player.y as f32 + south);
        let ground = shown_height_at(
            &mut cache,
            &world.surface,
            &world.climate,
            &world.vegetation,
            &known_rivers,
            &style,
            world.water_level(&clock),
            (f64::from(x), f64::from(z)),
        );
        let height = (super::sky::CLOUD_HEIGHT - ground).max(LEAST_HEIGHT);
        let at = Vec3::new(x, ground, z);
        let bolt = spawn_bolt(
            &mut commands,
            &mut meshes,
            &mut materials,
            strike.seed,
            at,
            height,
            BOLT_WIDTH,
        );
        commands.entity(bolt).insert(WorldMapScene);
    }
}
