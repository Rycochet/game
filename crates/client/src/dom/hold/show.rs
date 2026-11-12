//! Drawing what is held, above everything else.

use bevy::prelude::*;

use super::{HeldObject, HeldWhat};
use crate::dom::Inert;
use crate::widgets::{ABILITY, CELL, HELD_ITEM_Z, ItemLook, spawn_item_cells};

/// The node what is held is drawn in.
#[derive(Component)]
pub(super) struct HeldNode;

pub(super) fn spawn_held(mut commands: Commands) {
    let node = Node {
        position_type: PositionType::Absolute,
        ..default()
    };
    let above = GlobalZIndex(HELD_ITEM_Z);
    // Inert: what is held is never what the pointer is on.
    commands.spawn((HeldNode, Inert, node, above, Visibility::Hidden));
}

/// The held thing: an item at its true size, with the cell it was picked up
/// by under the pointer, or an ability with its middle there.
pub(super) fn show_held(
    held: Res<HeldObject>,
    node: Single<(Entity, &mut Node, &mut Visibility), With<HeldNode>>,
    mut commands: Commands,
    mut drawn: Local<Option<HeldWhat>>,
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
    if drawn.as_ref() != Some(&thing.what) {
        *drawn = Some(thing.what.clone());
        commands.entity(entity).despawn_related::<Children>();
        commands
            .entity(entity)
            .with_children(|parent| draw(parent, &thing.what));
    }
    // From the pointer to its top left corner.
    let grip = match &thing.what {
        HeldWhat::Item(item) => {
            Vec2::new(item.grip.0 as f32 + 0.5, item.grip.1 as f32 + 0.5) * CELL
        }
        HeldWhat::Ability(_) => Vec2::splat(ABILITY / 2.0),
    };
    if let Some(at) = held.at {
        let corner = at - grip;
        node.left = px(corner.x);
        node.top = px(corner.y);
    }
}

fn draw(parent: &mut ChildSpawnerCommands, what: &HeldWhat) {
    match what {
        HeldWhat::Item(held) => {
            let look = ItemLook::of(&held.item).turned(held.rotation);
            let shape = held.item.shape().turned(held.rotation);
            spawn_item_cells(parent, look, shape, (0.0, 0.0), false, |_, _| ());
        }
        HeldWhat::Ability(thing) => {
            // Its icon over the colour of what it leans on, as where it
            // came from.
            let node = Node {
                width: px(ABILITY),
                height: px(ABILITY),
                padding: UiRect::all(px(4.0)),
                border_radius: BorderRadius::all(px(4.0)),
                ..default()
            };
            let icon = ImageNode::new(thing.look.icon.clone());
            parent.spawn((icon, BackgroundColor(thing.look.colour), node));
        }
    }
}
