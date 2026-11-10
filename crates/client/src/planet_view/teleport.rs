//! Going to a World Gate from the view, by clicking a gate's ring, or opening
//! the debug menu's Teleport to World with this planet's seed by the bar's
//! Teleport to World.... Only on the planet it was opened for, as it is: the
//! bar's seed is a terrain seed, which can't be turned back into the world's
//! own seed for the server, and another generator would place the gates
//! elsewhere than the server has them.

use bevy::prelude::*;
use shared::protocol::ClientMessage;

use super::close::close;
use super::showing::{PlanetView, Showing};
use crate::common::{DebugTools, TeleportToWorldAsked};
use crate::connection::ToServer;
use crate::widgets::{ButtonPressed, ButtonState, HoverTip};
use shared::role::DebugTool;

/// Why the view's gates can't be gone to now, if they can't.
pub(super) fn why_not(showing: &Showing, dev_tools: bool) -> Option<&'static str> {
    if !dev_tools {
        Some("you may not use Teleport to World")
    } else if showing.choice.seed != showing.home.seed {
        Some("only the planet it was opened for")
    } else if showing.choice.generator != showing.home.generator {
        Some("not this planet's own generator")
    } else {
        None
    }
}

/// Asks the server for World Gate `gate` (from 0) of the world on show, and
/// closes the view.
pub(super) fn go_to_gate(
    gate: u8,
    view: &mut PlanetView,
    to_server: &mut MessageWriter<ToServer>,
    commands: &mut Commands,
) {
    let seed = view.showing.as_ref().and_then(|showing| showing.world);
    to_server.write(ToServer(ClientMessage::DevGoToWorld {
        seed,
        gate: Some(gate),
    }));
    close(view, commands);
}

/// Greys Teleport to World... where the planet on show isn't a world's,
/// saying why when pointed at; pressed, it closes the view and opens the
/// debug menu's panel with this world's seed.
pub(super) fn teleport_button(
    mut view: ResMut<PlanetView>,
    tools: Res<DebugTools>,
    mut pressed: MessageReader<ButtonPressed>,
    mut buttons: Query<(&mut ButtonState, &mut HoverTip, &mut Node)>,
    mut asked: MessageWriter<TeleportToWorldAsked>,
    mut commands: Commands,
) {
    let Some(showing) = view.showing.as_ref() else {
        return;
    };
    let teleport = showing.bar.teleport;
    let permitted = tools.may(DebugTool::TeleportToWorld);
    let why = why_not(showing, permitted);
    if let Ok((mut state, mut tip, mut node)) = buttons.get_mut(teleport) {
        // Hidden without permission (owner), as in the debug menu.
        let display = if permitted {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
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
        asked.write(TeleportToWorldAsked {
            seed: showing.world,
        });
        close(&mut view, &mut commands);
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
