//! The parts that are always there: the inventory button, the held item, the bin and the ground.

use bevy::prelude::*;

use super::{
    BUTTON, BinIcon, CORNER_GAP, DropIcon, HeldItem, InventoryButton, bin_icon, drop_icon,
    person_icon,
};
use crate::widgets::{
    ButtonState, ButtonTooltip, Floating, HELD_ITEM_Z, Pressable, spawn_icon_button,
};

/// The button, and the held item, which stands above panels.
pub(in crate::inventory) fn spawn_fixed_parts(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let icon = images.add(person_icon());
    commands.insert_resource(BinIcon(images.add(bin_icon())));
    commands.insert_resource(DropIcon(images.add(drop_icon())));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(CORNER_GAP),
                left: px(CORNER_GAP),
                ..default()
            },
            // Above the maps, as the warning button is.
            GlobalZIndex(15),
        ))
        .with_children(|holder| {
            let node = Node {
                width: px(BUTTON),
                height: px(BUTTON),
                ..default()
            };
            spawn_icon_button(holder, ButtonState::Normal, icon, node).insert((
                InventoryButton,
                Pressable,
                ButtonTooltip("Character".into()),
            ));
        });
    commands.spawn((
        HeldItem,
        Floating,
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        GlobalZIndex(HELD_ITEM_Z),
        Visibility::Hidden,
    ));
}
