//! Moving abilities about by mouse or finger (owner's design): dragged from the
//! Abilities page onto a slot puts it there; from one slot onto another swaps them;
//! from a slot onto the page takes it off the bar; anywhere else nothing changes.
//! A click or tap on a slot uses it (nothing yet: combat comes later); a right-click
//! or a finger held still on one turns its automatic use on or off.

use bevy::prelude::*;
use shared::ability::{Ability, BarChange};

use super::own_bar::OwnBar;
use super::spots::{Spot, Spots};
use crate::common::{TouchGesture, UiPointer, in_interface};
use crate::connection::ToServer;
use crate::inventory::Carried;

/// How far the mouse moves with its button down before a press is a drag.
const DRAG_PIXELS: f32 = 6.0;

/// Where a dragged ability came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum From {
    Page,
    Slot(u8),
}

/// What is being dragged, and where it is.
#[derive(Clone, Copy, Debug)]
pub(super) struct Dragged {
    pub ability: Ability,
    pub from: From,
    pub at: Vec2,
    by_finger: bool,
}

/// The ability being dragged, if any, and a mouse press that may become a drag.
#[derive(Resource, Default)]
pub(super) struct Drag {
    pub held: Option<Dragged>,
    pressed: Option<(Ability, From, Vec2)>,
    /// Where the finger last stood a still press on, so it acts once.
    held_still: Option<Vec2>,
}

/// The slot something dragged is over, and whether it could go there.
#[derive(Resource, Default, Clone, Copy, PartialEq)]
pub(super) struct Aim(pub Option<(u8, bool)>);

/// What can be picked up at a spot.
fn pick(spot: Option<Spot>, bar: &OwnBar) -> Option<(Ability, From)> {
    match spot? {
        Spot::Listed(ability) => Some((ability, From::Page)),
        Spot::Slot(slot) => bar.0.slots[slot as usize].map(|held| (held.ability, From::Slot(slot))),
        Spot::Page => None,
    }
}

/// What dropping `dragged` on `spot` asks for.
pub(super) fn drop_change(ability: Ability, from: From, spot: Option<Spot>) -> Option<BarChange> {
    match (from, spot?) {
        (From::Page, Spot::Slot(slot)) => Some(BarChange::Place { slot, ability }),
        (From::Slot(a), Spot::Slot(b)) if a != b => Some(BarChange::Swap { a, b }),
        (From::Slot(slot), Spot::Page | Spot::Listed(_)) => Some(BarChange::Clear { slot }),
        _ => None,
    }
}

/// Mouse and finger presses on abilities, before widgets and the map see them.
#[allow(clippy::too_many_arguments)]
pub(super) fn drag_abilities(
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    pointer: Res<UiPointer>,
    mut gesture: ResMut<TouchGesture>,
    ui_scale: Res<UiScale>,
    spots: Spots,
    mut bar: ResMut<OwnBar>,
    carried: Res<Carried>,
    mut drag: ResMut<Drag>,
    mut aim: ResMut<Aim>,
    mut to_server: MessageWriter<ToServer>,
) {
    let kind = carried.0.creature;
    let spot = pointer.0.and_then(|at| spots.at(at));
    if mouse.just_pressed(MouseButton::Right)
        && let Some(Spot::Slot(slot)) = spot
    {
        mouse.clear_just_pressed(MouseButton::Right);
        bar.change(BarChange::ToggleAuto { slot }, kind, &mut to_server);
    }
    if mouse.just_pressed(MouseButton::Left)
        && let Some((ability, from)) = pick(spot, &bar)
    {
        mouse.clear_just_pressed(MouseButton::Left);
        drag.pressed = pointer.0.map(|at| (ability, from, at));
    }
    if let (Some((ability, from, start)), Some(at)) = (drag.pressed, pointer.0)
        && mouse.pressed(MouseButton::Left)
        && at.distance(start) > DRAG_PIXELS
    {
        drag.pressed = None;
        drag.held = Some(Dragged {
            ability,
            from,
            at,
            by_finger: false,
        });
    }
    // A finger: dragging from an ability picks it up; held still on a slot, it
    // turns automatic use on or off, once.
    if gesture.first_finger.is_none() {
        drag.held_still = None;
    }
    if drag.held.is_none() && gesture.drag != Vec2::ZERO {
        let start = in_interface(gesture.drag_start, &ui_scale);
        if let Some((ability, from)) = pick(spots.at(start), &bar) {
            drag.held = Some(Dragged {
                ability,
                from,
                at: start,
                by_finger: true,
            });
        }
    }
    if let Some(still) = gesture.still_at.map(|at| in_interface(at, &ui_scale))
        && drag.held.is_none()
        && drag.held_still != Some(still)
        && let Some(Spot::Slot(slot)) = spots.at(still)
    {
        drag.held_still = Some(still);
        bar.change(BarChange::ToggleAuto { slot }, kind, &mut to_server);
    }
    let Some(mut held) = drag.held else {
        aim.set_if_neq(Aim(None));
        if mouse.just_released(MouseButton::Left) {
            if let Some((_, From::Slot(slot), _)) = drag.pressed {
                debug!("Using action bar slot {slot}: nothing yet, until combat");
            }
            drag.pressed = None;
        }
        return;
    };
    let still_down = if held.by_finger {
        // Moving what it holds, not scrolling or moving what is under it.
        gesture.drag = Vec2::ZERO;
        let finger = gesture.first_finger.map(|at| in_interface(at, &ui_scale));
        if let Some(at) = finger {
            held.at = at;
        }
        finger.is_some()
    } else {
        if let Some(at) = pointer.0 {
            held.at = at;
        }
        mouse.pressed(MouseButton::Left)
    };
    let over = spots.at(held.at);
    let change = drop_change(held.ability, held.from, over);
    let aimed = match (over, change) {
        (Some(Spot::Slot(slot)), Some(change)) => {
            Some((slot, bar.0.clone().apply(change, kind).is_ok()))
        }
        _ => None,
    };
    aim.set_if_neq(Aim(aimed));
    if still_down {
        drag.held = Some(held);
        return;
    }
    drag.held = None;
    aim.set_if_neq(Aim(None));
    if let Some(change) = change {
        bar.change(change, kind, &mut to_server);
    }
}
