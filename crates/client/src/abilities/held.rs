//! The ability being dragged, drawn at the pointer above everything, and the keys
//! 1 to 0 (or whatever Settings has) using the bar's slots.

use bevy::prelude::*;
use shared::ability::BAR_SLOTS;

use super::drag::Drag;
use super::icons::icon_path;
use super::use_ability::UseSlot;
use crate::common::{Action, Preferences};
use crate::widgets::{ABILITY, Floating, HELD_ITEM_Z, Shade, attribute_colour};

/// The picture following the pointer while an ability is dragged.
#[derive(Component)]
pub(super) struct HeldPicture;

pub(super) fn spawn_held(mut commands: Commands) {
    commands.spawn((
        HeldPicture,
        Floating,
        ImageNode::default(),
        BackgroundColor::default(),
        Node {
            position_type: PositionType::Absolute,
            width: px(ABILITY),
            height: px(ABILITY),
            padding: UiRect::all(px(4.0)),
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        },
        GlobalZIndex(HELD_ITEM_Z),
        Visibility::Hidden,
    ));
}

/// Shows what is being dragged, centred on the pointer.
pub(super) fn show_held(
    drag: Res<Drag>,
    assets: Res<AssetServer>,
    mut picture: Single<
        (
            &mut ImageNode,
            &mut BackgroundColor,
            &mut Node,
            &mut Visibility,
        ),
        With<HeldPicture>,
    >,
) {
    let (image, background, node, visibility) = &mut *picture;
    let Some(held) = drag.held else {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    };
    visibility.set_if_neq(Visibility::Inherited);
    let icon = assets.load(icon_path(held.ability));
    if image.image != icon {
        image.image = icon;
    }
    // Over the colour of what it leans on, as on the page and the bar.
    let ability = held.ability;
    let colour = attribute_colour(ability.leans(), Shade::of_ability(!ability.is_active()));
    background.set_if_neq(BackgroundColor(colour));
    node.left = px(held.at.x - ABILITY / 2.0);
    node.top = px(held.at.y - ABILITY / 2.0);
}

/// A slot's key uses it: nothing yet, until combat.
pub(super) fn use_by_key(
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    mut used: MessageWriter<UseSlot>,
) {
    for slot in 0..BAR_SLOTS as u8 {
        if preferences
            .controls
            .just_pressed(&keys, Action::UseSlot(slot))
        {
            used.write(UseSlot(slot));
        }
    }
}
