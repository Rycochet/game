//! Picking things up with a finger: a press held on an item.

use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex, UiGlobalTransform};
use shared::wear::Remover;

use super::hold::Held;
use super::panels::ItemCell;
use super::{Carried, Hand, Holding};
use crate::common::{GodMode, StockCell, TouchGesture, in_interface};
use crate::dom::HeldObject;
use crate::widgets::{Floating, shown_at};

/// The item cell in front of everything else at this point, if one is.
pub(super) fn item_cell_in_front<'a>(
    at: Vec2,
    nodes: impl Iterator<
        Item = (
            &'a ComputedStackIndex,
            &'a ComputedNode,
            &'a UiGlobalTransform,
            &'a InheritedVisibility,
            Option<&'a ItemCell>,
        ),
    >,
) -> Option<ItemCell> {
    nodes
        .filter(|(_, node, transform, visible, _)| {
            visible.get() && node.contains_point(**transform, at / node.inverse_scale_factor)
        })
        .max_by_key(|(stack, ..)| stack.0)
        .and_then(|(.., cell)| cell.copied())
}

/// A finger staying still on an item picks it up, once for each contact: a tap shows
/// its name, or opens it, and a finger sliding straight away is scrolling.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn pick_up_by_finger(
    mut gesture: ResMut<TouchGesture>,
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
    stock: Query<
        (
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Option<&StockCell>,
            Option<&CalculatedClip>,
        ),
        (With<Node>, Without<Floating>),
    >,
    (mut holding, mut object): (ResMut<Holding>, ResMut<HeldObject>),
    carried: Res<Carried>,
    god: Res<GodMode>,
    ui_scale: Res<UiScale>,
    mut decided: Local<Option<Vec2>>,
    mut split: ResMut<super::split::Split>,
) {
    if gesture.first_finger.is_none() {
        *decided = None;
    }
    if object
        .thing()
        .is_some_and(|thing| thing.item().is_some() && thing.by == Hand::Finger)
    {
        // Moving what it holds, not scrolling what is under it.
        gesture.drag = Vec2::ZERO;
    }
    let Some(start) = gesture
        .still_at
        .map(|start| in_interface(start, &ui_scale))
        .filter(|&start| object.thing().is_none() && *decided != Some(start))
    else {
        return;
    };
    *decided = Some(start);
    if let Some(cell) = item_cell_in_front(start, nodes.iter())
        && (god.0 || carried.0.may_remove(cell.id, Remover::Wearer))
    {
        // A mixed stack is chosen from first: see `split`.
        if let Some(stack) = carried.0.find(cell.id).filter(|item| !item.is_even())
            && !carried.0.in_bin(cell.id)
        {
            split.0 = Some(super::split::Splitting::open(cell.id, stack.count(), start));
            return;
        }
        if let Some(item) = carried.0.find(cell.id) {
            let held = Held {
                id: cell.id,
                rotation: cell.rotation,
                grip: cell.grip,
            };
            holding.pick_up(&mut object, item, held, Hand::Finger, None);
        }
        return;
    }
    let front = stock
        .iter()
        .filter(|(_, node, transform, visible, _, clip)| {
            visible.get() && shown_at(start, node, transform, *clip)
        })
        .max_by_key(|(stack, ..)| stack.0);
    if let Some((.., Some(cell), _)) = front {
        let held = Held {
            id: cell.item.id,
            rotation: cell.rotation,
            grip: cell.grip,
        };
        holding.pick_up(&mut object, &cell.item, held, Hand::Finger, None);
        holding.stock = Some((cell.with, cell.item.clone()));
    }
}
