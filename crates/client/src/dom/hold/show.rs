//! Drawing what is held, above everything else.

use bevy::prelude::*;
use shared::item::{Item, Rotation};

use super::HeldObject;
use crate::widgets::{CELL, Floating, HELD_ITEM_Z, ItemLook, spawn_item_cells};

/// The node what is held is drawn in.
#[derive(Component)]
pub(super) struct HeldNode;

pub(super) fn spawn_held(mut commands: Commands) {
    let node = Node {
        position_type: PositionType::Absolute,
        ..default()
    };
    let above = GlobalZIndex(HELD_ITEM_Z);
    commands.spawn((HeldNode, Floating, node, above, Visibility::Hidden));
}

/// The held thing, at its true size, with the cell it was picked up by under
/// the pointer.
pub(super) fn show_held(
    held: Res<HeldObject>,
    node: Single<(Entity, &mut Node, &mut Visibility), With<HeldNode>>,
    mut commands: Commands,
    mut drawn: Local<Option<(Item, Rotation)>>,
) {
    let (entity, mut node, mut visibility) = node.into_inner();
    let Some(thing) = held.thing() else {
        visibility.set_if_neq(Visibility::Hidden);
        if drawn.take().is_some() {
            commands.entity(entity).despawn_related::<Children>();
        }
        return;
    };
    visibility.set_if_neq(Visibility::Inherited);
    let same = drawn
        .as_ref()
        .is_some_and(|(item, rotation)| (item, *rotation) == (&thing.item, thing.rotation));
    if !same {
        *drawn = Some((thing.item.clone(), thing.rotation));
        commands.entity(entity).despawn_related::<Children>();
        let look = ItemLook::of(&thing.item).turned(thing.rotation);
        let shape = thing.item.shape().turned(thing.rotation);
        commands.entity(entity).with_children(|parent| {
            spawn_item_cells(parent, look, shape, (0.0, 0.0), false, |_, _| Floating);
        });
    }
    let grip = Vec2::new(thing.grip.0 as f32 + 0.5, thing.grip.1 as f32 + 0.5) * CELL;
    if let Some(at) = held.at {
        let corner = at - grip;
        node.left = px(corner.x);
        node.top = px(corner.y);
    }
}
