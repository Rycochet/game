//! Developer tools: going into a poi clicked on the full map.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use shared::protocol::ClientMessage;

use super::{FullMap, ICON_PIXELS, Room, hovered_poi, spot_on_full_map};
use crate::common::{ClickMode, CurrentWorld, GameView, SmoothPosition, UiPointer};
use crate::connection::ToServer;
use crate::player::Player;
use crate::view3d::OrbitCamera;

/// Developer tools: in Teleport Mode, a click on a poi on the full map teleports
/// there, enters it and closes the map. Elsewhere on the map, it is an ordinary
/// click.
#[allow(clippy::too_many_arguments)]
pub(super) fn go_into_clicked_poi(
    mode: Res<ClickMode>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    view: Res<State<GameView>>,
    camera: Single<&OrbitCamera>,
    player: Single<&SmoothPosition, With<Player>>,
    world: Option<Res<CurrentWorld>>,
    mut full: ResMut<FullMap>,
    mut to_server: MessageWriter<ToServer>,
    pointer: Res<UiPointer>,
) {
    let window = Room::of(&window, &ui_scale);
    let view = *view.get();
    let teleporting = *mode == ClickMode::Teleport;
    if !full.open
        || view != GameView::WorldMap
        || !teleporting
        || !mouse.just_pressed(MouseButton::Left)
    {
        return;
    }
    let centre = (player.x, player.y);
    let poi = pointer
        .0
        .and_then(|cursor| spot_on_full_map(&window, cursor, view, &camera, &full, centre))
        .and_then(|(spot, units_per_pixel)| {
            let reach = ICON_PIXELS * full.zoom / 2.0;
            hovered_poi(world.as_deref(), spot, reach, units_per_pixel)
        });
    let Some(poi) = poi else {
        return;
    };
    mouse.clear_just_pressed(MouseButton::Left);
    let (x, y) = (poi.x, poi.y);
    to_server.write(ToServer(ClientMessage::DevTeleport { x, y }));
    to_server.write(ToServer(ClientMessage::EnterSite { chunk: poi.chunk }));
    full.set_open(false);
}
