//! What is held now and where: turning it, following the pointer or finger, and
//! what letting go there would do.

use bevy::prelude::*;
use shared::item::StackLimits;

use super::super::hold::Target;
use super::super::{Carried, Hand, HeldBy, Holding};
use super::{Under, worth_wearing};
use crate::common::{
    GROUND, GameView, HeldGift, HeldOverGifts, HeldStock, TouchGesture, UiPointer, in_interface,
};
use crate::widgets::{CELL, logical_rect, shown_at};

/// The held thing, what letting go would do, where it is (in the interface's
/// pixels) and where a finger is. Nothing when nothing is held.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn aim_held(
    holding: &mut ResMut<Holding>,
    carried: &Carried,
    under: &Under,
    (pointer, ui_scale, view): (&UiPointer, &UiScale, &State<GameView>),
    mouse: &mut ResMut<ButtonInput<MouseButton>>,
    gesture: &mut ResMut<TouchGesture>,
    keys: &mut ResMut<ButtonInput<KeyCode>>,
    (over_gifts, held_stock): (&mut ResMut<HeldOverGifts>, &mut ResMut<HeldStock>),
) -> Option<(HeldBy, Target, Option<Vec2>, Option<Vec2>)> {
    let Under {
        grids,
        nodes,
        stacks,
    } = under;
    over_gifts.0 = None;
    let from_stock = holding.stock.as_ref().map(|(with, _)| *with);
    held_stock.set_if_neq(HeldStock(
        holding
            .held
            .filter(|_| from_stock.is_some())
            .map(|held| held.item.id),
    ));
    let Some(mut held) = holding.held else {
        holding.aim = None;
        holding.at = None;
        holding.stock = None;
        return None;
    };
    if holding.stock.is_none() && carried.0.find(held.item.id).is_none() {
        // The server says it has gone.
        holding.held = None;
        holding.aim = None;
        return None;
    }

    let turn = mouse.just_pressed(MouseButton::Right) || gesture.other_finger_tapped;
    // So the camera doesn't turn with the same press.
    mouse.reset(MouseButton::Right);
    if turn || keys.clear_just_pressed(KeyCode::KeyR) {
        held.item = match &holding.stock {
            Some((_, item)) => held.item.turned_as(item),
            None => held.item.turned(&carried.0),
        };
    }
    let finger = gesture.first_finger.map(|at| in_interface(at, ui_scale));
    if held.by == Hand::Finger {
        // Two fingers on the screen here are holding and turning, not moving the
        // camera.
        **gesture = TouchGesture::default();
    }
    let at = match held.by {
        Hand::Finger => finger.or(holding.at),
        Hand::Mouse | Hand::Clicked | Hand::Debug => pointer.0.or(holding.at),
    };
    holding.at = at;
    let over = at.and_then(|at| {
        grids
            .iter()
            .filter(|(_, node, transform, visible, _)| {
                visible.get() && node.contains_point(**transform, at / node.inverse_scale_factor)
            })
            .max_by_key(|(.., stack)| stack.0)
            .map(|(grid, node, transform, ..)| {
                let (corner, _) = logical_rect(node, transform);
                let cell = ((at - corner) / CELL).floor();
                (grid.holder, cell.x as i32, cell.y as i32)
            })
    });
    // Only what is in front counts: the bin destroys things.
    let (on_bin, on_drop, on_body, on_row, on_gifts) = at
        .and_then(|at| {
            nodes
                .iter()
                .filter(|(_, node, transform, visible, .., clip)| {
                    visible.get() && shown_at(at, node, transform, *clip)
                })
                .max_by_key(|(stack, ..)| stack.0)
                .map(|(.., is_bin, is_drop, is_body, row, gifts, _)| {
                    (
                        is_bin,
                        is_drop,
                        is_body,
                        row.map(|row| row.0),
                        gifts.map(|gifts| gifts.0),
                    )
                })
        })
        .unwrap_or_default();
    let on_row = on_row.or(on_gifts);
    // Something of theirs can only go back to them.
    let on_gifts = on_gifts.filter(|&to| from_stock.is_none_or(|from| from == to));
    over_gifts.0 = on_gifts.zip(at).map(|(to, at)| HeldGift {
        to,
        item: held.item.id,
        rotation: held.item.rotation,
        corner: at - (Vec2::new(held.item.grip.0 as f32, held.item.grip.1 as f32) + 0.5) * CELL,
    });
    let aim = if let Some((with, item)) = &holding.stock {
        let from_ground = *with == GROUND;
        if on_drop {
            // On the ground already, or someone else's.
            Target::Drop { allowed: false }
        } else if from_ground && on_bin {
            Target::Bin
        } else if from_ground && on_body {
            // Only the top of a stack goes on.
            Target::Body {
                allowed: worth_wearing(carried.0.check_wear_new(item.top())),
            }
        } else if on_bin || on_body || on_row.is_some() {
            Target::Nowhere
        } else {
            // Only what is on the ground can go on a stack, not someone's stock.
            let limits = if from_ground {
                stacks.0
            } else {
                StackLimits::NONE
            };
            super::super::hold::target_for_new(&carried.0, item, held.item, over, limits)
        }
    } else if on_drop {
        // Only sites have ground, and what is covered can't come off.
        Target::Drop {
            allowed: *view.get() == GameView::Dungeon
                && carried.0.covered_by(held.item.id).is_none(),
        }
    } else if on_bin {
        // In it already: nothing happens.
        if carried.0.in_bin(held.item.id) {
            Target::Nowhere
        } else {
            Target::Bin
        }
    } else if let Some(someone) = on_row {
        Target::Someone(someone)
    } else if on_body {
        // On it already: nothing happens, whatever it is over.
        if carried.0.is_worn(held.item.id) {
            Target::Nowhere
        } else {
            Target::Body {
                allowed: worth_wearing(carried.0.check_wear(held.item.id)),
            }
        }
    } else {
        super::super::hold::target(&carried.0, held.item, over, stacks.0)
    };
    Some((held, aim, at, finger))
}
