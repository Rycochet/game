//! Full-screen panels: each is a layer, darkening everything outside it, with
//! any layer above it smaller, so what is outside both is darkened twice. Only
//! what is on the top layer can be used.

use bevy::prelude::*;

use super::theme::DARKENING;
use super::{PANEL_EDGE, Pressable, panel, panel_title};
use crate::dom::Opaque;

/// Above every moveable panel, however many times they have been brought to the
/// front. A layer's own parts may go a little above its number.
const FIRST_LAYER_Z: i32 = 1_000_000;
const Z_PER_LAYER: i32 = 10;

/// How much of the window the first layer takes, and how much less each one
/// above it.
const DIALOG_MARGIN: f32 = 16.0;
const DIALOG_BUTTON_GAP: f32 = 8.0;
const FIRST_SHARE: f32 = 0.9;
const SHARE_PER_LAYER: f32 = 0.15;

/// A layer's darkening, which while it shows stops anything under it being used.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct Darkening;

/// The `GlobalZIndex` of the layer `depth` above the first, for its darkening.
pub fn layer_z(depth: i32) -> i32 {
    FIRST_LAYER_Z + depth * Z_PER_LAYER
}

/// How much of the window's width and height this layer's panel takes.
pub fn layer_share(depth: i32) -> f32 {
    FIRST_SHARE - SHARE_PER_LAYER * depth as f32
}

/// A darkening across the window, for a panel that places itself (a dialog's,
/// a `dark` panel's in `dom`). Add a way to take presses on it (`Pressable`,
/// `dom::OwnPointer`), and hide it to hide it.
pub fn darkening(depth: i32) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(DARKENING),
        GlobalZIndex(layer_z(depth)),
        Darkening,
    )
}

/// A full-screen panel, hidden, as layer `depth`, centred. `content` lays out
/// its contents; its size and edge are set here. Returns the layer, to show and
/// hide (a press outside the panel is a `ButtonPressed` for it), and the panel,
/// to put the contents in.
pub fn spawn_fullscreen_layer(
    commands: &mut Commands,
    depth: i32,
    content: Node,
) -> (Entity, Entity) {
    let share = percent(layer_share(depth) * 100.0);
    spawn_sized_layer(commands, depth, share, share, content)
}

/// As `spawn_fullscreen_layer`, with a panel of this size rather than its depth's
/// (`Val::Auto` fits its contents).
pub fn spawn_sized_layer(
    commands: &mut Commands,
    depth: i32,
    width: Val,
    height: Val,
    content: Node,
) -> (Entity, Entity) {
    let layer = commands
        .spawn((darkening(depth), Pressable, Visibility::Hidden))
        .id();
    commands.entity(layer).insert(Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    });
    let panel_entity = commands.spawn(panel()).id();
    commands.entity(panel_entity).insert((
        Opaque,
        Node {
            position_type: PositionType::Relative,
            width,
            height,
            border: UiRect::all(px(PANEL_EDGE)),
            ..content
        },
    ));
    commands.entity(layer).add_child(panel_entity);
    (layer, panel_entity)
}

/// The parts of a dialog.
pub struct Dialog {
    /// To show and hide; a press outside the panel is a `ButtonPressed` for it.
    pub layer: Entity,
    /// To put what it says in.
    pub content: Entity,
    /// To put its buttons in, along the bottom on the right.
    pub buttons: Entity,
}

/// A hidden panel asking something that needs an answer now, as layer `depth`.
pub fn spawn_dialog(commands: &mut Commands, depth: i32, title: &str) -> Dialog {
    let (layer, panel) = spawn_sized_layer(
        commands,
        depth,
        Val::Auto,
        Val::Auto,
        Node {
            padding: UiRect::all(px(DIALOG_MARGIN)),
            flex_direction: FlexDirection::Column,
            row_gap: px(DIALOG_MARGIN),
            ..default()
        },
    );
    let title = commands.spawn(panel_title(title)).id();
    let content = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .id();
    let buttons = commands
        .spawn(Node {
            justify_content: JustifyContent::End,
            column_gap: px(DIALOG_BUTTON_GAP),
            ..default()
        })
        .id();
    commands
        .entity(panel)
        .add_children(&[title, content, buttons]);
    Dialog {
        layer,
        content,
        buttons,
    }
}

#[cfg(test)]
mod tests;
