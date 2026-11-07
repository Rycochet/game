//! Calling off the player's ability while it winds up: Esc, or moving (owner,
//! 2026-11-07). Once it has landed there is nothing to call off, and Esc opens
//! the menu as usual.

use bevy::prelude::*;
use shared::combat::CANCEL_PENALTY;
use shared::protocol::ClientMessage;

use crate::common::{CancelledEnergy, OwnAbility, SkillsPageOpen};
use crate::connection::ToServer;
use crate::debug_menu::DebugMenu;
use crate::minimap::FullMap;
use crate::player::{TakeStep, TravelTo};
use crate::system_menu::SystemMenu;

/// Calls it off: the server is asked, the player's energy here comes back but
/// for the penalty, and the bar's wait is over then too.
fn cancel(
    now: f64,
    own: &mut OwnAbility,
    to_server: &mut MessageWriter<ToServer>,
    cancelled: &mut MessageWriter<CancelledEnergy>,
) {
    to_server.write(ToServer(ClientMessage::CancelAbility));
    cancelled.write(CancelledEnergy(CANCEL_PENALTY));
    own.lands = now;
    own.ready = own.ready.min(now);
}

/// Esc while winding up calls it off instead of opening the menu (unless
/// something Esc closes is open). Before the system menu takes input.
#[allow(clippy::too_many_arguments)]
pub(super) fn cancel_by_escape(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    time: Res<Time>,
    menu: Res<SystemMenu>,
    debug_menu: Res<DebugMenu>,
    (full_map, skills): (Res<FullMap>, Res<SkillsPageOpen>),
    mut own: ResMut<OwnAbility>,
    mut to_server: MessageWriter<ToServer>,
    mut cancelled: MessageWriter<CancelledEnergy>,
) {
    let now = time.elapsed_secs_f64();
    let open = menu.is_open() || debug_menu.open || full_map.open || skills.0;
    if open || !own.is_winding_up(now) || !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    keys.clear_just_pressed(KeyCode::Escape);
    cancel(now, &mut own, &mut to_server, &mut cancelled);
}

/// A step or a journey asked for while winding up calls it off, so the
/// player can move away.
pub(super) fn cancel_by_moving(
    mut steps: MessageReader<TakeStep>,
    mut journeys: MessageReader<TravelTo>,
    time: Res<Time>,
    mut own: ResMut<OwnAbility>,
    mut to_server: MessageWriter<ToServer>,
    mut cancelled: MessageWriter<CancelledEnergy>,
) {
    let moving = steps.read().count() + journeys.read().count() > 0;
    let now = time.elapsed_secs_f64();
    if moving && own.is_winding_up(now) {
        cancel(now, &mut own, &mut to_server, &mut cancelled);
    }
}
