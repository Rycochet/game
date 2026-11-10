//! What Teleport to World's buttons do, greying them where they can't, and
//! opening it with a seed when View Planet asks.

use bevy::prelude::*;
use shared::protocol::ClientMessage;

use super::super::actions::DebugAction;
use super::teleport_to_world::{TeleportButton, TeleportForm, WorldSeedBox, seed_in};
use super::{GenerateForm, OpenPanels, new_seed};
use crate::common::{DebugTools, TeleportToWorldAsked};
use crate::connection::ToServer;
use crate::planet_view::ViewPlanet;
use crate::widgets::{ButtonPressed, ButtonState, RaisePanel, TextBox};
use shared::role::DebugTool;

/// View Planet's Teleport to World...: the panel, with that world's seed in
/// it (blank for the player's own), the gate back at Any.
pub(in crate::debug_menu) fn open_teleport_asked(
    mut asked: MessageReader<TeleportToWorldAsked>,
    mut form: ResMut<TeleportForm>,
    mut open: ResMut<OpenPanels>,
    mut commands: Commands,
    mut raise: MessageWriter<RaisePanel>,
    nodes: Query<(), With<Node>>,
) {
    let Some(&TeleportToWorldAsked { seed }) = asked.read().last() else {
        return;
    };
    open.open(
        DebugAction::TeleportToWorld,
        &mut commands,
        &mut raise,
        &nodes,
    );
    form.gate = None;
    form.seed_to_show = Some(seed.map_or(String::new(), |seed| seed.to_string()));
}

/// What the pressed button does: a random seed, choosing a gate, or asking to
/// go, to the Hall, or to see the planet.
#[allow(clippy::too_many_arguments)]
pub(in crate::debug_menu) fn press_teleport_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&TeleportButton>,
    mut seed_box: Query<&mut TextBox, With<WorldSeedBox>>,
    mut form: ResMut<TeleportForm>,
    // Its count of seeds made keeps random ones apart.
    mut generate: ResMut<GenerateForm>,
    time: Res<Time<Real>>,
    mut to_server: MessageWriter<ToServer>,
    mut view_planet: MessageWriter<ViewPlanet>,
) {
    for press in pressed.read() {
        let (Ok(&button), Ok(mut seed_box)) = (buttons.get(press.0), seed_box.single_mut()) else {
            continue;
        };
        let seed = seed_in(&seed_box.text);
        let gate = form.gate;
        match (button, seed) {
            (TeleportButton::RandomSeed, _) => {
                generate.seeds_made += 1;
                let new = new_seed(time.elapsed().as_nanos() as u64, generate.seeds_made);
                seed_box.text = new.to_string();
            }
            (TeleportButton::Gate(gate), _) => form.gate = gate,
            (TeleportButton::Go, Ok(seed)) => {
                to_server.write(ToServer(ClientMessage::DevGoToWorld { seed, gate }));
            }
            (TeleportButton::Hall, Ok(seed)) => {
                to_server.write(ToServer(ClientMessage::DevToHall { seed, gate }));
            }
            (TeleportButton::ViewPlanet, Ok(world)) => {
                view_planet.write(ViewPlanet { world });
            }
            // A seed that isn't one: its buttons are greyed.
            (_, Err(())) => {}
        }
    }
}

/// Puts a seed View Planet asked for in the box, marks the gate chosen, and
/// greys what can't be done: anything with a seed that isn't a number, and Go
/// with nowhere new to go (this world, Any gate). View Planet... is hidden
/// without its own permission, the rest without Teleport to World's.
pub(in crate::debug_menu) fn update_teleport_buttons(
    tools: Res<DebugTools>,
    mut form: ResMut<TeleportForm>,
    mut seed_box: Query<&mut TextBox, With<WorldSeedBox>>,
    mut buttons: Query<(&TeleportButton, &mut ButtonState, &mut Node)>,
) {
    let teleport = tools.may(DebugTool::TeleportToWorld);
    let Ok(mut text_box) = seed_box.single_mut() else {
        return;
    };
    if let Some(seed) = form.seed_to_show.take() {
        text_box.text = seed;
    }
    let seed = seed_in(&text_box.text);
    if text_box.error != seed.is_err() {
        text_box.error = seed.is_err();
    }
    let gate = form.gate;
    for (&button, mut state, mut node) in &mut buttons {
        let permitted = match button {
            TeleportButton::ViewPlanet => tools.may(DebugTool::ViewPlanet),
            _ => teleport,
        };
        super::show_if(&mut node, permitted);
        let can = match button {
            TeleportButton::RandomSeed | TeleportButton::Gate(_) => true,
            TeleportButton::ViewPlanet | TeleportButton::Hall => seed.is_ok(),
            TeleportButton::Go => seed.is_ok_and(|seed| seed.is_some() || gate.is_some()),
        };
        let chosen = button == TeleportButton::Gate(gate);
        state.set_if_neq(match (can, chosen) {
            (true, true) => ButtonState::Selected,
            (true, false) => ButtonState::Normal,
            (false, true) => ButtonState::DisabledSelected,
            (false, false) => ButtonState::Disabled,
        });
    }
}
