//! Moving abilities about by mouse or finger (owner's design): dragged from the
//! Abilities page onto a slot puts it there; from one slot onto another swaps them;
//! from a slot onto the page takes it off the bar; anywhere else nothing changes.
//! A click or tap on a slot uses it; a right-click or a finger held still on one
//! turns its automatic use on or off.
//!
//! The dragging itself (following the pointer or finger, being drawn, being let
//! go) is `dom`'s, as for items (`HeldObject`): `drag_abilities` picks up before
//! it, and `drop_abilities` says what a drop would do, and does it, after.

use bevy::prelude::*;
use shared::ability::{Ability, BarChange};

use super::own_bar::OwnBar;
use super::page::look;
use super::spots::{Spot, Spots};
use super::use_ability::UseSlot;
use crate::common::{TouchGesture, UiPointer, in_interface};
use crate::connection::ToServer;
use crate::dom::{AbilityThing, Dropped, Hand, HeldObject, HeldThing, HeldWhat};
use crate::inventory::Carried;

/// How far the mouse moves with its button down before a press is a drag.
const DRAG_PIXELS: f32 = 6.0;

/// Where a dragged ability came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum From {
    Page,
    Slot(u8),
}

/// Where the ability being dragged came from, and a mouse press that may become
/// a drag.
#[derive(Resource, Default)]
pub(super) struct Drag {
    from: Option<From>,
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

/// Mouse and finger presses on abilities, before widgets and the map see them:
/// using a slot, turning its automatic use on or off, and picking an ability up
/// once it is dragged.
#[allow(clippy::too_many_arguments)]
pub(super) fn drag_abilities(
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    pointer: Res<UiPointer>,
    gesture: Res<TouchGesture>,
    ui_scale: Res<UiScale>,
    spots: Spots,
    mut bar: ResMut<OwnBar>,
    carried: Res<Carried>,
    assets: Res<AssetServer>,
    (mut drag, mut object): (ResMut<Drag>, ResMut<HeldObject>),
    mut to_server: MessageWriter<ToServer>,
    mut used: MessageWriter<UseSlot>,
) {
    let kind = carried.0.creature;
    let spot = pointer.0.and_then(|at| spots.at(at));
    // Something already held (an item, say) isn't for the bar.
    let free = object.thing().is_none();
    if mouse.just_pressed(MouseButton::Right)
        && free
        && let Some(Spot::Slot(slot)) = spot
    {
        mouse.clear_just_pressed(MouseButton::Right);
        bar.change(BarChange::ToggleAuto { slot }, kind, &mut to_server);
    }
    if mouse.just_pressed(MouseButton::Left)
        && free
        && let Some((ability, from)) = pick(spot, &bar)
    {
        mouse.clear_just_pressed(MouseButton::Left);
        drag.pressed = pointer.0.map(|at| (ability, from, at));
    }
    let mut pick_up = |ability: Ability, from: From, by: Hand, drag: &mut Drag| {
        let thing = AbilityThing {
            ability,
            look: look(ability, kind, &assets),
        };
        let what = HeldWhat::Ability(thing);
        // (No place and time picked up: letting the button go drops it.)
        object.pick_up(HeldThing { what, by }, None);
        drag.from = Some(from);
    };
    if let (Some((ability, from, start)), Some(at)) = (drag.pressed, pointer.0)
        && mouse.pressed(MouseButton::Left)
        && at.distance(start) > DRAG_PIXELS
    {
        drag.pressed = None;
        pick_up(ability, from, Hand::Mouse, &mut drag);
        return;
    }
    // A finger: dragging from an ability picks it up; held still on a slot, it
    // turns automatic use on or off, once.
    if gesture.first_finger.is_none() {
        drag.held_still = None;
    }
    if free && gesture.drag != Vec2::ZERO {
        let start = in_interface(gesture.drag_start, &ui_scale);
        if let Some((ability, from)) = pick(spots.at(start), &bar) {
            pick_up(ability, from, Hand::Finger, &mut drag);
            return;
        }
    }
    if let Some(still) = gesture.still_at.map(|at| in_interface(at, &ui_scale))
        && free
        && drag.held_still != Some(still)
        && let Some(Spot::Slot(slot)) = spots.at(still)
    {
        drag.held_still = Some(still);
        bar.change(BarChange::ToggleAuto { slot }, kind, &mut to_server);
    }
    if free && mouse.just_released(MouseButton::Left) {
        if let Some((_, From::Slot(slot), _)) = drag.pressed {
            used.write(UseSlot(slot));
        }
        drag.pressed = None;
    }
}

/// What letting go of the ability dragged would do, and doing it when `dom`
/// says it was let go.
pub(super) fn drop_abilities(
    object: Res<HeldObject>,
    mut dropped: MessageReader<Dropped>,
    spots: Spots,
    mut bar: ResMut<OwnBar>,
    carried: Res<Carried>,
    (mut drag, mut aim): (ResMut<Drag>, ResMut<Aim>),
    mut to_server: MessageWriter<ToServer>,
) {
    let kind = carried.0.creature;
    let held = object.thing().and_then(|thing| thing.ability());
    let aimed = held.zip(drag.from).and_then(|(thing, from)| {
        let over = object.over.front.and_then(|front| spots.of(front));
        let change = drop_change(thing.ability, from, over)?;
        match over {
            Some(Spot::Slot(slot)) => Some((slot, bar.0.clone().apply(change, kind).is_ok())),
            _ => None,
        }
    });
    aim.set_if_neq(Aim(aimed));
    for drop in dropped.read() {
        let (Some(thing), Some(from)) = (drop.thing.ability(), drag.from.take()) else {
            continue;
        };
        let over = drop.over.front.and_then(|front| spots.of(front));
        if let Some(change) = drop_change(thing.ability, from, over) {
            bar.change(change, kind, &mut to_server);
        }
    }
    if held.is_none() {
        // Let go, or put back with Esc.
        drag.from = None;
    }
}
