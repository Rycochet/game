//! Opening the inventory, and forgetting grids whose panels closed.

use bevy::prelude::*;
use shared::item::Holder;

use super::super::give::NearbyPanel;
use super::super::{Carried, OpenCharacter};
use super::{InventoryButton, OpenGrids};
use crate::common::GameView;
use crate::widgets::{ButtonPressed, RaisePanel};

/// The button, `C` or `I` opens or closes the Character panel.
#[allow(clippy::too_many_arguments)]
pub(in crate::inventory) fn toggle_inventory(
    mut pressed: MessageReader<ButtonPressed>,
    mut asked: MessageReader<OpenCharacter>,
    button: Query<(), With<InventoryButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    view: Res<State<GameView>>,
    carried: Res<Carried>,
    mut open: ResMut<OpenGrids>,
    mut nearby: ResMut<NearbyPanel>,
    nodes: Query<(), With<Node>>,
    mut commands: Commands,
    mut raise: MessageWriter<RaisePanel>,
) {
    let by_button = pressed.read().any(|press| button.contains(press.0));
    let by_key = keys.any_just_pressed([KeyCode::KeyC, KeyCode::KeyI]);
    if (by_button || by_key) && *view.get() != GameView::Connecting {
        // Things are given by dragging them from there.
        if open.inventory_open() {
            nearby.close(&mut commands, &nodes);
        } else {
            nearby.open(&mut commands, &mut raise, &nodes);
        }
        open.toggle_inventory(&carried.0, &mut commands, &mut raise);
    } else if asked.read().count() > 0 {
        if !open.inventory_open() {
            nearby.open(&mut commands, &mut raise, &nodes);
        }
        open.open(Holder::Inventory, &carried.0, &mut commands, &mut raise);
    }
}

/// Forgets panels closed by their own close buttons, closes those of bags no
/// longer carried, and draws the inventory again if the body changed its size.
pub(in crate::inventory) fn forget_closed(
    carried: Res<Carried>,
    mut open: ResMut<OpenGrids>,
    nodes: Query<(), With<Node>>,
    mut commands: Commands,
    mut raise: MessageWriter<RaisePanel>,
) {
    let mut redraw_inventory = false;
    open.open.retain(|&(holder, panel, size)| {
        if !nodes.contains(panel) {
            return false;
        }
        let keep = carried.0.grid_size(holder) == Some(size);
        if !keep {
            commands.entity(panel).despawn();
            redraw_inventory |= holder == Holder::Inventory;
        }
        keep
    });
    if redraw_inventory {
        open.open(Holder::Inventory, &carried.0, &mut commands, &mut raise);
    }
}
