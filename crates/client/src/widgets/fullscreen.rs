//! Full-screen panels: each is a layer, darkening everything outside it, with
//! any layer above it smaller, so what is outside both is darkened twice. Only
//! what is on the top layer can be used.

use bevy::prelude::*;

use super::moveable::PanelPart;
use super::{Pressable, panel, raised_border, stops_presses};

/// Above every moveable panel, however many times they have been brought to the
/// front. A layer's own parts may go a little above its number.
const FIRST_LAYER_Z: i32 = 1_000_000;
const Z_PER_LAYER: i32 = 10;

/// How much of the window the first layer takes, and how much less each one
/// above it.
const FIRST_SHARE: f32 = 0.9;
const SHARE_PER_LAYER: f32 = 0.15;

/// The width of a layer's edge.
pub const LAYER_EDGE: f32 = 3.0;

const DARKENING: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);

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

/// A darkening across the window, for a panel that places itself. Add a way to
/// take presses on it (`Pressable`, `leaves_presses`), and hide it to hide it.
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

/// Put this on a node for presses on it, and on what is in front of it with no
/// press taking of its own, to reach no widget under it and be left for whoever
/// owns it to read from the mouse. `entity` is the node.
pub fn leaves_presses(entity: Entity) -> impl Bundle {
    PanelPart::LeftForOwner(entity)
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
    let share = layer_share(depth) * 100.0;
    let panel_entity = commands.spawn((panel(), raised_border())).id();
    commands.entity(panel_entity).insert((
        stops_presses(panel_entity),
        Node {
            position_type: PositionType::Relative,
            width: percent(share),
            height: percent(share),
            border: UiRect::all(px(LAYER_EDGE)),
            ..content
        },
    ));
    commands.entity(layer).add_child(panel_entity);
    (layer, panel_entity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_layer_is_smaller_and_in_front_of_the_last() {
        assert!(layer_share(1) < layer_share(0));
        assert!((layer_share(0) - 0.9).abs() < 0.001);
        assert!(layer_z(1) > layer_z(0));
    }
}
