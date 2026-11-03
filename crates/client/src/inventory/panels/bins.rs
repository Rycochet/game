//! Showing the bin and the ground to drop on.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::Rotation;

use super::super::hold::Target;
use super::super::{Carried, Holding};
use super::{BinIcon, BinNode, BinSlot, DropIcon, DropNode, ItemCell};
use crate::item_art::look_of;
use crate::widgets::{
    ALLOWED, ITEM_EDGE, ITEM_LABEL_FONT, PICTURE_PIXELS_PER_CELL, Pressable, REFUSED, item_colour,
    item_label, item_picture, picture_in_square,
};

/// Gives the bin its picture, shows what is in it, and whether dropping something
/// on it would throw that away for good.
#[allow(clippy::type_complexity)]
pub(in crate::inventory) fn show_bins(
    icon: Res<BinIcon>,
    carried: Res<Carried>,
    holding: Res<Holding>,
    new_bins: Query<Entity, Added<BinSlot>>,
    mut bins: Query<(Entity, &mut BackgroundColor), With<BinSlot>>,
    mut commands: Commands,
    mut drawn: Local<Option<Id>>,
) {
    for bin in &new_bins {
        commands
            .entity(bin)
            .insert((ImageNode::new(icon.0.clone()), BackgroundColor(Color::NONE)));
    }
    let held = holding.held.map(|held| held.item.id);
    let inside = carried.0.bin.as_ref().filter(|item| Some(item.id) != held);
    let now = inside.map(|item| item.id);
    let redraw = *drawn != now || !new_bins.is_empty();
    *drawn = now;
    let tint = match (holding.aim, carried.0.bin.is_some()) {
        (Some(Target::Bin), true) => REFUSED,
        (Some(Target::Bin), false) => ALLOWED,
        _ => Color::NONE,
    };
    for (bin, mut background) in &mut bins {
        background.set_if_neq(BackgroundColor(tint));
        if !redraw {
            continue;
        }
        commands.entity(bin).despawn_related::<Children>();
        let Some(item) = inside else {
            continue;
        };
        // Any shape shows as one cell here.
        let cell = ItemCell {
            id: item.id,
            rotation: Rotation::None,
            grip: item.shape().cells().next().unwrap_or((0, 0)),
        };
        let kind = item.kind;
        let picture = look_of(item);
        commands.entity(bin).with_children(|bin| {
            let colour = if picture.is_some() {
                Color::NONE
            } else {
                item_colour(kind)
            };
            let mut slot = bin.spawn((
                BinNode,
                cell,
                Pressable,
                Node {
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(colour),
            ));
            match picture {
                Some(look) => {
                    let node = picture_in_square(&look);
                    let pixels = look.longest() as u32 * PICTURE_PIXELS_PER_CELL;
                    slot.with_child(item_picture(look, pixels, node, Rotation::None));
                }
                None => {
                    slot.with_child((
                        BinNode,
                        cell,
                        Text::new(item_label(kind)),
                        ITEM_LABEL_FONT.font(),
                        TextColor(ITEM_EDGE),
                    ));
                }
            }
        });
    }
}

/// Gives the Drop icon its picture, and shows what dropping something on it would do.
pub(in crate::inventory) fn show_drop(
    icon: Res<DropIcon>,
    holding: Res<Holding>,
    new_icons: Query<Entity, Added<DropNode>>,
    mut icons: Query<&mut BackgroundColor, With<DropNode>>,
    mut commands: Commands,
) {
    for entity in &new_icons {
        commands
            .entity(entity)
            .insert((ImageNode::new(icon.0.clone()), BackgroundColor(Color::NONE)));
    }
    let tint = match holding.aim {
        Some(Target::Drop { allowed: true }) => ALLOWED,
        Some(Target::Drop { allowed: false }) => REFUSED,
        _ => Color::NONE,
    };
    for mut background in &mut icons {
        background.set_if_neq(BackgroundColor(tint));
    }
}
