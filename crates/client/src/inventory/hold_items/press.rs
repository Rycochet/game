//! Taking this frame's presses on items: a double click on a bag opens it, and
//! a press on anything else picks it up.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::{Holder, Inventory};

use super::super::hold::Held;
use super::super::panels::{ItemCell, OpenGrids};
use super::super::split::{PickRow, Split, SplitCell, Splitting};
use super::super::{Carried, DOUBLE_PRESS_SECONDS, Hand, HeldBy, Holding};
use crate::common::StockCell;
use crate::dom::Dom;
use crate::dom::HeldObject;
use crate::widgets::ButtonPressed;

/// Takes the presses. Whether that was all there is to do this frame.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn take_presses(
    pressed: &mut MessageReader<ButtonPressed>,
    (cells, stock_cells, split_cells, pick_rows): (
        &Query<&ItemCell>,
        &Query<&StockCell>,
        &Query<&SplitCell>,
        &Query<&PickRow>,
    ),
    split: &mut ResMut<Split>,
    mouse: &mut ResMut<ButtonInput<MouseButton>>,
    (holding, object): (&mut ResMut<Holding>, &mut ResMut<HeldObject>),
    carried: &Carried,
    open: &mut ResMut<OpenGrids>,
    commands: &mut Commands,
    dom: &mut Dom,
    last_press: &mut Option<(shared::id::Id, f64)>,
    (now, new_press, tapped, at_press, shift): (f64, bool, bool, Option<Vec2>, bool),
) -> bool {
    let clicked_held = HeldBy::of(object)
        .filter(|held| held.by == Hand::Clicked)
        .map(|held| held.item.id);
    if clicked_held.is_some() && new_press {
        // For putting it down, not for what is under it.
        mouse.clear_just_pressed(MouseButton::Left);
    }
    if let Some(id) = clicked_held.filter(|_| new_press)
        && last_press
            .is_some_and(|(pressed, when)| pressed == id && now - when < DOUBLE_PRESS_SECONDS)
        && carried
            .0
            .find(id)
            .is_some_and(|item| item.kind.def().container.is_some())
        && !carried.0.in_bin(id)
    {
        // A double click on a bag: the first lifted it, and it goes back.
        holding.put_back(object);
        open.open(Holder::Inside(id), &carried.0, commands, dom);
        pressed.clear();
        return true;
    }
    let picked = at_press.map(|at| (at, now));
    for press in pressed.read() {
        if clicked_held.is_some() {
            continue;
        }
        if let Ok(PickRow(items)) = pick_rows.get(press.0) {
            // A row of a mixed stack's box: those items together, or all of it.
            if let Some(splitting) = split.0.take()
                && let Some((_, placed)) = carried.0.placed(splitting.stack)
            {
                let id = items.as_ref().map_or(splitting.stack, |items| items[0]);
                holding.group = items.clone().map(|items| (splitting.stack, items));
                let held = Held {
                    id,
                    rotation: placed.rotation,
                    grip: placed.shape().cells().next().unwrap_or((0, 0)),
                };
                // (As the stack until what is lifted from it is known.)
                holding.pick_up(object, &placed.item, held, Hand::Mouse, picked);
            }
            continue;
        }
        if let Ok(cell) = split_cells.get(press.0) {
            // The stack in the split box: that many from its top, picked up as anything
            // is, a tap or quick click holding it until the next.
            if let Some(splitting) = split.0.take()
                && let Some(part) =
                    part_to_hold(&carried.0, holding, splitting.stack, splitting.count)
                && let Some((_, placed)) = carried.0.placed(splitting.stack)
            {
                let held = Held {
                    id: part,
                    rotation: placed.rotation,
                    grip: cell.grip,
                };
                // (As the stack until what is lifted from it is known.)
                holding.pick_up(object, &placed.item, held, Hand::Mouse, picked);
            }
            continue;
        }
        if let Ok(cell) = stock_cells.get(press.0) {
            if tapped {
                continue;
            }
            let held = Held {
                id: cell.item.id,
                rotation: cell.rotation,
                grip: cell.grip,
            };
            holding.pick_up(object, &cell.item, held, Hand::Mouse, picked);
            holding.stock = Some((cell.with, cell.item.clone()));
            continue;
        }
        let Ok(&cell) = cells.get(press.0) else {
            continue;
        };
        let again =
            last_press.is_some_and(|(id, when)| id == cell.id && now - when < DOUBLE_PRESS_SECONDS);
        *last_press = Some((cell.id, now));
        let is_bag = carried
            .0
            .find(cell.id)
            .is_some_and(|item| item.kind.def().container.is_some())
            && !carried.0.in_bin(cell.id);
        if again && is_bag {
            open.open(Holder::Inside(cell.id), &carried.0, commands, dom);
            continue;
        }
        let all = carried.0.find(cell.id).map_or(1, |item| item.count());
        let mixed = carried.0.find(cell.id).is_some_and(|item| !item.is_even());
        if (shift || tapped || mixed) && all > 1 && !carried.0.in_bin(cell.id) {
            // A stack to split: see `split`.
            if let Some(at) = at_press {
                split.0 = Some(Splitting::open(cell.id, all, at));
            }
            continue;
        }
        if tapped {
            continue;
        }
        let Some(item) = carried.0.find(cell.id) else {
            continue;
        };
        let held = Held {
            id: cell.id,
            rotation: cell.rotation,
            grip: cell.grip,
        };
        holding.pick_up(object, item, held, Hand::Mouse, picked);
    }
    false
}

/// The id of the top `count` of a carried stack, to hold. Part of a counted stack
/// is new: a stand-in names it until the server does (`PartOfStack`), noted in
/// `Holding::counted`.
fn part_to_hold(carried: &Inventory, holding: &mut Holding, stack: Id, count: u32) -> Option<Id> {
    if let Some(part) = carried.part_of(stack, count) {
        return Some(part);
    }
    if !carried.find(stack)?.is_counted() {
        return None;
    }
    holding.counted = Some((stack, count));
    Some(holding.stand_in())
}
