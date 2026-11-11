//! How an ability looks: its icon, outlined in black, in a frame half again as big
//! as an item's cell, named on hovering, over the colour of the attributes it leans
//! on (`attribute_colour`). Square for active abilities, round for passive ones;
//! greyed, with a dull red frame, when it can't be used now.

use bevy::prelude::*;

use super::{CELL, HoverTip, ITEM_EDGE};

/// An ability's frame, across.
pub const ABILITY: f32 = CELL * 1.5;
/// Its frame's edge.
const EDGE: f32 = 2.0;
/// Between the frame and the icon.
const INSET: f32 = 4.0;
/// Behind an ability that can't be used now.
const UNUSABLE_BACKGROUND: Color = Color::srgb(0.16, 0.15, 0.14);
const UNUSABLE_EDGE: Color = Color::srgb(0.45, 0.16, 0.12);
const UNUSABLE_ICON: Color = Color::srgb(0.45, 0.45, 0.45);

/// What an ability shows.
#[derive(Clone, Debug, PartialEq)]
pub struct AbilityLook {
    /// Its icon, white on a transparent background.
    pub icon: Handle<Image>,
    /// What it is called, shown on hovering.
    pub name: String,
    /// Always at work rather than done when asked: drawn round.
    pub passive: bool,
    /// Whether it can be done now.
    pub usable: bool,
    /// Behind the icon: `attribute_colour` of what it leans on.
    pub colour: Color,
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
            BackgroundColor(if look.usable {
                look.colour
            } else {
                UNUSABLE_BACKGROUND
            }),
            BorderColor::from(edge),
            HoverTip(look.name),
        ))
        .with_children(|frame| spawn_outlined_icon(frame, look.icon, icon))
        .id()
}

/// How far round the icon its black outline reaches, in logical pixels.
const OUTLINE: f32 = 1.0;

/// The icon filling its parent, in `colour`, with a black outline: the icon in
/// black nudged each way behind it, as outlined words are drawn. Abilities and the
/// Skills page's nodes.
pub fn spawn_outlined_icon(parent: &mut ChildSpawnerCommands, icon: Handle<Image>, colour: Color) {
    parent
        .spawn(Node {
            width: percent(100.0),
            height: percent(100.0),
            ..default()
        })
        .with_children(|holder| {
            for dx in [-OUTLINE, 0.0, OUTLINE] {
                for dy in [-OUTLINE, 0.0, OUTLINE] {
                    if (dx, dy) == (0.0, 0.0) {
                        continue;
                    }
                    holder.spawn((
                        ImageNode::new(icon.clone()).with_color(Color::BLACK),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(dx),
                            top: px(dy),
                            width: percent(100.0),
                            height: percent(100.0),
                            ..default()
                        },
                    ));
                }
            }
            holder.spawn((
                ImageNode::new(icon).with_color(colour),
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
            ));
        });
}
