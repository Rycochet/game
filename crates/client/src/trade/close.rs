//! Closing panels: Esc, or walking out of reach.

use bevy::prelude::*;

use super::{OpenTrades, beside};
use crate::common::{GameView, MapPosition};
use crate::player::Player;

/// Esc closes the panel opened last rather than opening the system menu.
pub(super) fn close_on_escape(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut open: ResMut<OpenTrades>,
    nodes: Query<(), With<Node>>,
    mut commands: Commands,
) {
    open.0.retain(|trade| nodes.contains(trade.panel));
    if !open.0.is_empty()
        && keys.clear_just_pressed(KeyCode::Escape)
        && let Some(trade) = open.0.pop()
    {
        commands.entity(trade.panel).despawn();
    }
}

/// Walking out of reach or leaving the map closes a panel, and its close button
/// forgets it.
pub(super) fn close_when_left(
    view: Res<State<GameView>>,
    player: Single<&MapPosition, With<Player>>,
    nodes: Query<(), With<Node>>,
    mut open: ResMut<OpenTrades>,
    mut commands: Commands,
) {
    let on_map = *view.get() == GameView::Dungeon;
    open.0.retain(|trade| {
        if !nodes.contains(trade.panel) {
            return false;
        }
        let stays = on_map && beside(**player, trade.place);
        if !stays {
            commands.entity(trade.panel).despawn();
        }
        stays
    });
}
