//! The hover box's parts.

use bevy::prelude::*;

use crate::widgets::spawn_tooltip_with_picture;

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

/// The staff mark before a marked player's name (`common::StaffMark`).
#[derive(Component)]
pub(super) struct HoverMark;

/// The staff mark's picture, and its colour: gold.
const MARK: &str = "icons/lorc/laurel-crown.png";
const MARK_COLOUR: Color = Color::srgb(0.95, 0.75, 0.25);

pub(super) fn spawn_box(mut commands: Commands, assets: Res<AssetServer>) {
    let mark = assets.load(MARK);
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
        .with_children(|holder| {
            spawn_tooltip_with_picture(holder, HoverText, HoverMark, mark, MARK_COLOUR);
        });
}
