//! Doors in the player's way. Walking into a closed door asks the server to
//! open it (`DoorAsk::Open`) instead of stepping; a journey waits at one until
//! it is open, or ends there if the server refuses (`DoorRefused`), or if the
//! door itself was where it was going. The Close Doors key (L) closes the open
//! doors beside the player. Nothing knows whether a key fits a lock: the door
//! is tried.

use bevy::prelude::*;
use shared::door::door_cost;
use shared::protocol::{ClientMessage, DoorAsk, ServerMessage};

use super::{Player, Travel};
use crate::common::{Action, CurrentDungeon, GameView, Preferences, SpentEnergy};
use crate::connection::{FromServer, ToServer};

/// Something asked of doors, for the server: sent by `send_door_asks`, which
/// counts the time it takes as the server will.
#[derive(Message, Clone, Copy, Debug)]
pub(super) struct AskDoor(pub(super) DoorAsk);

/// A journey has come to the closed door at `door`: asks it open, once, and
/// ends there if the door was where it was going.
pub(super) fn wait_at_door(
    travel: &mut Travel,
    door: (i64, i64),
    asks: &mut MessageWriter<AskDoor>,
) {
    if travel.door_asked != Some(door) {
        travel.door_asked = Some(door);
        asks.write(AskDoor(DoorAsk::Open {
            x: door.0,
            y: door.1,
        }));
    }
    if travel.goal == door {
        travel.route.clear();
    }
}

/// Sends what was asked of doors, spending its time here as the server will.
pub(super) fn send_door_asks(
    mut asks: MessageReader<AskDoor>,
    mut to_server: MessageWriter<ToServer>,
    mut spent: MessageWriter<SpentEnergy>,
) {
    for &AskDoor(ask) in asks.read() {
        to_server.write(ToServer(ClientMessage::Door(ask)));
        spent.write(SpentEnergy(door_cost()));
    }
}

/// The Close Doors key (L): closes the open doors beside the player.
pub(super) fn close_doors(
    preferences: Res<Preferences>,
    keys: Res<ButtonInput<KeyCode>>,
    view: Res<State<GameView>>,
    mut asks: MessageWriter<AskDoor>,
) {
    if *view.get() == GameView::Dungeon
        && preferences.controls.just_pressed(&keys, Action::CloseDoors)
    {
        asks.write(AskDoor(DoorAsk::CloseNear));
    }
}

/// The server wouldn't open a door: a journey waiting at it ends there. A door
/// that opens lets the journey on, and may be asked again later.
pub(super) fn door_answers(
    mut from_server: MessageReader<FromServer>,
    dungeon: Res<CurrentDungeon>,
    travel: Single<&mut Travel, With<Player>>,
) {
    let mut travel = travel.into_inner();
    for FromServer(message) in from_server.read() {
        if let ServerMessage::DoorRefused { x, y } = *message
            && travel.door_asked == Some((x, y))
        {
            info!("The door at ({x}, {y}) stays shut");
            travel.route.clear();
            travel.door_asked = None;
        }
    }
    if let Some((x, y)) = travel.door_asked
        && !dungeon.level.is_shut(x, y)
    {
        travel.door_asked = None;
    }
}
