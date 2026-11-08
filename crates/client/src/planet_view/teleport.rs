//! Going to a World Gate from the view: by the bar's Teleport to Gate 1, or by
//! clicking a gate's ring. Only on the planet the player is on, as it is: the
//! bar's seed is a terrain seed, which can't be turned back into the world's
//! own seed for the server, and another generator would place the gates
//! elsewhere than the server has them.

use bevy::prelude::*;
use shared::protocol::ClientMessage;

use super::close::close;
use super::showing::{PlanetView, Showing};
use crate::common::{CurrentWorld, ServerDevTools};
use crate::connection::ToServer;
use crate::widgets::{ButtonPressed, ButtonState, HoverTip};

/// Why the view's gates can't be gone to now, if they can't.
pub(super) fn why_not(
    showing: &Showing,
    world: &CurrentWorld,
    dev_tools: bool,
) -> Option<&'static str> {
    if !dev_tools {
        Some("the server doesn't allow dev tools")
    } else if showing.choice.seed != world.params.seed {
        Some("only the planet you are on")
    } else if showing.choice.generator != world.params.generator {
        Some("not this planet's own generator")
    } else {
        None
    }
}

/// Asks the server for World Gate `gate` (from 0) of the world the player is
/// in, and closes the view.
pub(super) fn go_to_gate(
    gate: u8,
    view: &mut PlanetView,
    to_server: &mut MessageWriter<ToServer>,
    commands: &mut Commands,
) {
    to_server.write(ToServer(ClientMessage::DevGoToWorld {
        seed: None,
        gate: Some(gate),
    }));
    close(view, commands);
}

/// Greys Teleport to Gate 1 where it can't go, saying why when pointed at,
/// and goes when it is pressed.
#[allow(clippy::too_many_arguments)]
pub(super) fn teleport_button(
    mut view: ResMut<PlanetView>,
    world: Option<Res<CurrentWorld>>,
    dev_tools: Res<ServerDevTools>,
    mut pressed: MessageReader<ButtonPressed>,
    mut buttons: Query<(&mut ButtonState, &mut HoverTip)>,
    mut to_server: MessageWriter<ToServer>,
    mut commands: Commands,
) {
    let (Some(showing), Some(world)) = (view.showing.as_ref(), world) else {
        return;
    };
    let teleport = showing.bar.teleport;
    let why = why_not(showing, &world, dev_tools.0).or_else(|| {
        let none = showing.globe.planet.gates.gates.is_empty();
        none.then_some("this planet has no World Gates")
    });
    if let Ok((mut state, mut tip)) = buttons.get_mut(teleport) {
        state.set_if_neq(match why {
            Some(_) => ButtonState::Disabled,
            None => ButtonState::Normal,
        });
        let words = why.map_or(String::new(), |why| format!("Not now: {why}"));
        if tip.0 != words {
            tip.0 = words;
        }
    }
    if why.is_none() && pressed.read().any(|press| press.0 == teleport) {
        go_to_gate(0, &mut view, &mut to_server, &mut commands);
    }
}

/// A click or tap `take_input` noted, once the ring under it is known: on one
/// that can be gone to, it goes there; on one that can't, nothing; anywhere
/// else, the view closes.
pub(super) fn answer_click(
    mut view: ResMut<PlanetView>,
    mut to_server: MessageWriter<ToServer>,
    mut commands: Commands,
) {
    let Some(showing) = view.showing.as_mut() else {
        return;
    };
    if !std::mem::take(&mut showing.clicked) {
        return;
    }
    match showing.pointed {
        Some(gate) if gate.can_go => {
            go_to_gate(gate.number, &mut view, &mut to_server, &mut commands);
        }
        Some(_) => {}
        None => close(&mut view, &mut commands),
    }
}
