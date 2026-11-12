//! Picking things up with a finger: a press held on an item.

use bevy::prelude::*;
use shared::wear::Remover;

use super::hold::Held;
use super::panels::ItemCell;
use super::{Carried, Hand, Holding};
use crate::common::{GodMode, StockCell, TouchGesture, in_interface};
use crate::dom::{Finder, HeldObject};

/// The item cell in front of everything else, if one is: the nearest to
/// what is in front (`chain`: `dom::Hit`'s, or `dom::Finder`'s for a finger).
pub(super) fn item_cell_in_front(chain: &[Entity], cells: &Query<&ItemCell>) -> Option<ItemCell> {
    chain
        .iter()
        .find_map(|&entity| cells.get(entity).ok().copied())
}

/// A finger staying still on an item picks it up, once for each contact: a tap shows
/// its name, or opens it, and a finger sliding straight away is scrolling.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn pick_up_by_finger(
    mut gesture: ResMut<TouchGesture>,
    (finder, cells, stock): (Finder, Query<&ItemCell>, Query<&StockCell>),
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
    let chain = finder.chain_at(start);
    if let Some(cell) = item_cell_in_front(&chain, &cells)
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
    if let Some(cell) = chain.iter().find_map(|&entity| stock.get(entity).ok()) {
        let held = Held {
            id: cell.item.id,
            rotation: cell.rotation,
            grip: cell.grip,
        };
        holding.pick_up(&mut object, &cell.item, held, Hand::Finger, None);
        holding.stock = Some((cell.with, cell.item.clone()));
    }
}
