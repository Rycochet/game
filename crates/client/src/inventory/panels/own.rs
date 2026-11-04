//! Opening the inventory, and forgetting grids whose panels closed.

use bevy::prelude::*;
use shared::item::Holder;

use super::super::give::NearbyPanel;
use super::super::{Carried, OpenCharacter};
use super::grids::spawn_cells;
use super::shown::shown_size;
use super::{GridNode, InventoryButton, OpenGrids};
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
/// longer carried, and draws a grid again, where it is, when its size changes: the
/// inventory's when the body changes, a growing bag's as it fills or empties.
pub(in crate::inventory) fn forget_closed(
    carried: Res<Carried>,
    mut open: ResMut<OpenGrids>,
    nodes: Query<(), With<Node>>,
    grids: Query<(Entity, &GridNode, &ChildOf)>,
    mut commands: Commands,
) {
    open.open.retain_mut(|(holder, panel, size)| {
        if !nodes.contains(*panel) {
            return false;
        }
        let Some(now) = shown_size(&carried.0, *holder) else {
            commands.entity(*panel).despawn();
            return false;
        };
        if now != *size {
            *size = now;
            let grid = grids.iter().find(|(_, grid, _)| grid.holder == *holder);
            if let Some((grid, _, parent)) = grid {
                commands.entity(grid).despawn();
                let holder = *holder;
                commands
                    .entity(parent.parent())
                    .with_children(|parent| spawn_cells(parent, holder, now));
            }
        }
        true
    });
}
