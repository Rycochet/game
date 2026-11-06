//! Clicks on the terrain.

use bevy::prelude::*;
use shared::protocol::ClientMessage;

use super::rivers::KnownRivers;
use super::terrain::HeightCache;
use super::view_square;
use crate::common::{
    ClickMode, CurrentWorld, GameClock, MapPosition, PointerPosition, Preferences,
};
use crate::connection::ToServer;
use crate::player::{Player, TravelTo};
use crate::surface_style::SurfaceStyle;
use crate::view3d::{ClickedToken, OrbitCamera, ViewCamera};

/// A left click on the terrain sends the player travelling there (unless it
/// was on a clickable token, such as a poi: `site` handles those); in Teleport
/// Mode it asks the server to teleport them (dev tools).
#[allow(clippy::too_many_arguments)]
pub(super) fn pick_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mode: Res<ClickMode>,
    clicked: Res<ClickedToken>,
    pointer: Res<PointerPosition>,
    camera: Single<(&Camera, &GlobalTransform, &OrbitCamera), With<ViewCamera>>,
    player: Single<&MapPosition, With<Player>>,
    world: Res<CurrentWorld>,
    clock: Res<GameClock>,
    style: Res<SurfaceStyle>,
    known_rivers: Res<KnownRivers>,
    cache: Option<ResMut<HeightCache>>,
    mut travel: MessageWriter<TravelTo>,
    mut to_server: MessageWriter<ToServer>,
    preferences: Res<Preferences>,
) {
    if !mouse.just_pressed(MouseButton::Left) || clicked.0.is_some() {
        return;
    }
    let (Some(cursor), Some(mut cache)) = (pointer.0, cache) else {
        return;
    };
    let (camera, camera_transform, orbit) = *camera;
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    let water_level = world.water_level(&clock);
    if let Some((x, y)) = super::picking::pick(
        ray,
        (player.x, player.y),
        view_square(orbit, preferences.video.view_distance),
        &mut cache,
        &world.surface,
        &style,
        water_level,
        |cache: &mut HeightCache, x: i64, z: i64| {
            super::terrain::canopy_lift(
                cache,
                &world.surface,
                &world.climate,
                &world.vegetation,
                &known_rivers,
                &style,
                water_level,
                (x, z),
            )
        },
    ) {
        // Dev tools: Teleport Mode jumps there instead (through walls), if the
        // server allows it.
        if *mode == ClickMode::Teleport {
            to_server.write(ToServer(ClientMessage::DevTeleport { x, y }));
        } else {
            travel.write(TravelTo { x, y });
        }
    }
}
