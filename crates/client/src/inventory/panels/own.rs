//! Opening the inventory, and forgetting grids whose panels closed.

use bevy::prelude::*;
use shared::item::Holder;

use super::super::give::NearbyPanel;
use super::super::{Carried, OpenCharacter};
use super::grids::spawn_cells;
use super::shown::shown_size;
use super::{GridNode, InventoryButton, OpenGrids, Page};
use crate::common::{Action, GameView, Preferences};
use crate::widgets::{ButtonPressed, ChooseTab, RaisePanel};

/// The keys for the Character panel's tabs (C, I, B).
const TAB_KEYS: [(Action, Page); 3] = [
    (Action::Character, Page::Character),
    (Action::Inventory, Page::Inventory),
    (Action::Abilities, Page::Abilities),
];

/// The button opens or closes the Character panel, on the tab last shown. A tab's key
/// opens it on that tab, or shows that tab, or closes it if that tab is on show.
/// `OpenCharacter` opens it on its Inventory tab.
#[allow(clippy::too_many_arguments)]
pub(in crate::inventory) fn toggle_inventory(
    mut pressed: MessageReader<ButtonPressed>,
    mut asked: MessageReader<OpenCharacter>,
    button: Query<(), With<InventoryButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    view: Res<State<GameView>>,
    carried: Res<Carried>,
    mut open: ResMut<OpenGrids>,
    mut nearby: ResMut<NearbyPanel>,
    nodes: Query<(), With<Node>>,
    mut commands: Commands,
    mut raise: MessageWriter<RaisePanel>,
    mut choose: MessageWriter<ChooseTab>,
) {
    let by_button = pressed.read().any(|press| button.contains(press.0));
    let by_key = TAB_KEYS
        .iter()
        .find(|(action, _)| preferences.controls.just_pressed(&keys, *action))
        .map(|&(_, page)| page);
    let by_asking = asked.read().count() > 0;
    if *view.get() == GameView::Connecting {
        return;
    }
    let want = match (by_button, by_key, by_asking) {
        (true, ..) => None,
        (false, Some(page), _) => Some(page),
        (false, None, true) => Some(Page::Inventory),
        (false, None, false) => return,
    };
    let is_open = open.inventory_open();
    let close = match want {
        None => is_open,
        // Asking never closes it.
        Some(page) => is_open && open.page == page && !by_asking,
    };
    if close {
        // Things are given by dragging them from there.
        nearby.close(&mut commands, &nodes);
        open.toggle_inventory(&carried.0, &mut commands, &mut raise);
        return;
    }
    if let Some(page) = want {
        open.page = page;
        if let Some(pages) = open.pages {
            pages.show(page, &mut commands);
            choose.write(ChooseTab(pages.row, page as usize));
        }
    }
    if !is_open {
        nearby.open(&mut commands, &mut raise, &nodes);
    }
    open.open(Holder::Inventory, &carried.0, &mut commands, &mut raise);
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
    let mut closed_character = false;
    open.open.retain_mut(|(holder, panel, size)| {
        if !nodes.contains(*panel) {
            closed_character |= *holder == Holder::Inventory;
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
    if closed_character {
        open.pages = None;
    }
}
