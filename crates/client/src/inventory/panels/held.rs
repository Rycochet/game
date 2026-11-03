//! Showing what is held, tooltips over items, and the pockets button.

use bevy::prelude::*;
use bevy::ui::{ComputedStackIndex, UiGlobalTransform};
use shared::id::Id;
use shared::item::Rotation;

use super::super::body::worn_notes;
use super::super::hold::Target;
use super::super::{Carried, Holding, item_cell_in_front};
use super::{HELD_NAME_ABOVE, HeldItem, ItemCell, OpenGrids, PocketsButton, TOOLTIP_OFFSET};
use crate::common::{GROUND, GameView, UiPointer};
use crate::widgets::{CELL, Floating, ItemLook, ItemTip, ItemTooltip, spawn_item_cells};

/// The held item, at its true size, with the cell it was picked up by under the
/// pointer.
#[allow(clippy::type_complexity)]
pub(in crate::inventory) fn show_held(
    carried: Res<Carried>,
    holding: Res<Holding>,
    held_item: Single<(Entity, &mut Node, &mut Visibility), With<HeldItem>>,
    mut commands: Commands,
    mut drawn: Local<Option<(Id, Rotation)>>,
) {
    let (entity, mut node, mut visibility) = held_item.into_inner();
    let held = holding.held.map(|held| held.item);
    let item = held.and_then(|_| holding.item(&carried.0));
    let (Some(held), Some(item)) = (held, item) else {
        visibility.set_if_neq(Visibility::Hidden);
        if drawn.take().is_some() {
            commands.entity(entity).despawn_related::<Children>();
        }
        return;
    };
    visibility.set_if_neq(Visibility::Inherited);
    let now = Some((held.id, held.rotation));
    if *drawn != now {
        *drawn = now;
        commands.entity(entity).despawn_related::<Children>();
        let look = ItemLook::of(item).turned(held.rotation);
        let shape = item.shape().turned(held.rotation);
        commands.entity(entity).with_children(|parent| {
            spawn_item_cells(parent, look, shape, (0.0, 0.0), false, |_, _| Floating);
        });
    }
    let grip = Vec2::new(held.grip.0 as f32 + 0.5, held.grip.1 as f32 + 0.5) * CELL;
    if let Some(at) = holding.at {
        let corner = at - grip;
        node.left = px(corner.x);
        node.top = px(corner.y);
    }
}

/// The name of what is under the pointer, or of what is held.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(in crate::inventory) fn show_tooltip(
    carried: Res<Carried>,
    holding: Res<Holding>,
    pointer: Res<UiPointer>,
    open: Res<OpenGrids>,
    nodes: Query<
        (
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Option<&ItemCell>,
        ),
        (With<Node>, Without<Floating>),
    >,
    mut tooltip: ResMut<ItemTooltip>,
    split: Res<super::super::split::Split>,
) {
    let shown = if let Some(held) = holding.held {
        Some(held.item.id)
    } else if open.open.is_empty() || split.0.is_some() {
        // The split box names the stack itself.
        None
    } else {
        pointer
            .0
            .and_then(|at| item_cell_in_front(at, nodes.iter()))
            .map(|cell| cell.id)
    };
    let name = shown.and_then(|id| {
        let item = match holding.held {
            Some(_) => holding.item(&carried.0)?,
            None => carried.0.find(id)?,
        };
        // A stack is named by its top, which is what shows.
        let item = item.top();
        let words = format!("{}{}", item.name(), worn_notes(&carried.0, id));
        Some((words, item.quality, item.set))
    });
    let (Some((name, quality, set)), Some(at)) = (name, pointer.0.or(holding.at)) else {
        return;
    };
    let words = match holding.aim {
        Some(Target::Someone(GROUND)) | Some(Target::Drop { allowed: true }) => {
            format!("Drop {name}")
        }
        Some(Target::Drop { allowed: false }) => format!("Can't drop {name}"),
        Some(Target::Someone(_)) => format!("Give {name}"),
        Some(Target::Bin) => match carried.0.bin.as_ref() {
            Some(old) => format!("Throw away {name}, destroying {}", old.name()),
            None => format!("Throw away {name}"),
        },
        _ => name,
    };
    let top = match holding.held {
        // Above what is held, so as not to hide any of it.
        Some(held) => at.y - (held.item.grip.1 as f32 + 0.5) * CELL - HELD_NAME_ABOVE,
        None => at.y + TOOLTIP_OFFSET.y,
    };
    tooltip.0 = Some(ItemTip {
        words,
        quality,
        set,
        at: Vec2::new(at.x + TOOLTIP_OFFSET.x, top),
    });
}

/// The button is hidden while connecting, as the warning button is.
pub(in crate::inventory) fn show_button(
    view: Res<State<GameView>>,
    mut button: Single<&mut Visibility, With<PocketsButton>>,
) {
    button.set_if_neq(if *view.get() == GameView::Connecting {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
}
