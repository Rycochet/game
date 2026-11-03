//! Clicks on cells and doors.

use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::protocol::ClientMessage;

use super::Shown;
use crate::common::{ClickMode, CurrentDungeon, HoveredBag, PointerPosition};
use crate::connection::ToServer;
use crate::player::TravelTo;
use crate::view3d::{ClickedToken, ViewCamera};

/// A left click on a seen (or remembered) walkable cell sends the player
/// travelling there (in
/// a straight line for now: walls don't stop travel until there is
/// pathfinding). Clicks on clickable tokens are left to `site`; so is using
/// stairs, by arriving on their cell. In Teleport Mode it asks the server to teleport
/// the player there, and in Change Cell Mode it changes the cell (floor, wall,
/// door, window, and round again) to try walls out (dev tools).
#[allow(clippy::too_many_arguments)]
pub(super) fn pick_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mode: Res<ClickMode>,
    clicked: Res<ClickedToken>,
    pointer: Res<PointerPosition>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    dungeon: Res<CurrentDungeon>,
    shown: Res<Shown>,
    bag: Res<HoveredBag>,
    mut travel: MessageWriter<TravelTo>,
    mut door_clicked: MessageWriter<DoorClicked>,
    mut to_server: MessageWriter<ToServer>,
) {
    if !mouse.just_pressed(MouseButton::Left) || clicked.0.is_some() {
        return;
    }
    // Left to `ground`.
    if bag.0.is_some() && *mode == ClickMode::Normal {
        return;
    }
    let Some(cursor) = pointer.0 else {
        return;
    };
    let (camera, camera_transform) = *camera;
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    if *mode == ClickMode::ChangeCell {
        if let Some((x, y)) = super::picking::floor_cell(ray) {
            let cell = match dungeon.level.cell(x, y) {
                Cell::Floor => Cell::Wall,
                Cell::Wall => Cell::Door,
                Cell::Door => Cell::Window,
                _ => Cell::Floor,
            };
            to_server.write(ToServer(ClientMessage::DevSetCell { x, y, cell }));
        }
        return;
    }
    let door = super::picking::door_clicked(ray, &dungeon.buildings, &shown.solid_faces);
    if let Some(building) = door
        && *mode != ClickMode::Teleport
    {
        door_clicked.write(DoorClicked { building });
        return;
    }
    let front = door.and_then(|id| dungeon.buildings.iter().find(|b| b.id == id));
    let front = front.map(|b| (b.door.0 + b.faces.0, b.door.1 + b.faces.1));
    let picked = front.or_else(|| {
        super::picking::pick(ray, &dungeon.level, &dungeon.remembered, &shown.solid_faces)
    });
    if let Some((x, y)) = picked {
        // Dev tools: Teleport Mode jumps there instead (through walls), if the
        // server allows it.
        if *mode == ClickMode::Teleport {
            to_server.write(ToServer(ClientMessage::DevTeleport { x, y }));
        } else {
            travel.write(TravelTo { x, y });
        }
    }
}

/// A town building's door was clicked: for `shop`, which decides what that means.
#[derive(Message, Clone, Copy, Debug)]
pub struct DoorClicked {
    /// Its `BuildingLook::id`.
    pub building: u16,
}
