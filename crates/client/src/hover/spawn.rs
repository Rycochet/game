//! The hover box's parts.

use bevy::prelude::*;

use crate::widgets::spawn_tooltip;

/// How far above the bottom of the window the box sits, or above whatever the
/// interface keeps there (`common::BottomHud`), in interface pixels.
pub(super) const LIFT: f32 = 12.0;

/// The lowest the box goes, in interface pixels.
pub(super) const LOWEST: f32 = 32.0;

/// Holds the box, centred across the window.
#[derive(Component)]
pub(super) struct HoverBox;

#[derive(Component)]
pub(super) struct HoverText;

pub(super) fn spawn_box(mut commands: Commands) {
    commands
        .spawn((
            HoverBox,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(LOWEST),
                left: px(0),
                right: px(0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(10),
            Visibility::Hidden,
        ))
        .with_children(|holder| spawn_tooltip(holder, HoverText, ""));
}
