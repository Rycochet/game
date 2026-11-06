//! How an ability looks: its icon in a frame half again as big as an item's cell,
//! named on hovering. Square for active abilities, round for passive ones; greyed,
//! with a dull red frame, when it can't be used now.

use bevy::prelude::*;

use super::{CELL, HoverTip, ITEM_EDGE};

/// An ability's frame, across.
pub const ABILITY: f32 = CELL * 1.5;
/// Its frame's edge.
const EDGE: f32 = 2.0;
/// Between the frame and the icon.
const INSET: f32 = 4.0;
const BACKGROUND: Color = Color::srgb(0.16, 0.12, 0.08);
const UNUSABLE_EDGE: Color = Color::srgb(0.45, 0.16, 0.12);
const UNUSABLE_ICON: Color = Color::srgb(0.45, 0.45, 0.45);

/// What an ability shows.
#[derive(Clone, Debug)]
pub struct AbilityLook {
    /// Its icon, white on a transparent background.
    pub icon: Handle<Image>,
    /// What it is called, shown on hovering.
    pub name: String,
    /// Always at work rather than done when asked: drawn round.
    pub passive: bool,
    /// Whether it can be done now.
    pub usable: bool,
}

/// An ability, `ABILITY` across. Returns its frame.
pub fn spawn_ability(parent: &mut ChildSpawnerCommands, look: AbilityLook) -> Entity {
    let edge = if look.usable {
        ITEM_EDGE
    } else {
        UNUSABLE_EDGE
    };
    let icon = if look.usable {
        Color::WHITE
    } else {
        UNUSABLE_ICON
    };
    let radius = if look.passive {
        BorderRadius::MAX
    } else {
        BorderRadius::all(px(4.0))
    };
    parent
        .spawn((
            Node {
                width: px(ABILITY),
                height: px(ABILITY),
                border: UiRect::all(px(EDGE)),
                padding: UiRect::all(px(INSET)),
                border_radius: radius,
                ..default()
            },
            BackgroundColor(BACKGROUND),
            BorderColor::from(edge),
            HoverTip(look.name),
        ))
        .with_children(|frame| {
            frame.spawn((
                ImageNode::new(look.icon).with_color(icon),
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
            ));
        })
        .id()
}
