//! Holding things: picking up with the mouse, turning, following the pointer and
//! letting go, in three steps: `press`, `aim` and `let_go`.

mod aim;
mod let_go;
mod press;

use aim::aim_held;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex, UiGlobalTransform};
use let_go::let_go_of;
use press::take_presses;
use shared::wear::{Fit, WearRefused};

use super::body::BodyNode;
use super::give::GiveRow;
use super::panels::{BinNode, DropNode, ItemCell, OpenGrids};
use super::split::{PickRow, Split, SplitCell};
use super::{CLICK_PIXELS, CLICK_SECONDS, Carried, Hand, Holding};
use crate::common::{
    DroppedOnText, GameView, GaveItem, HeldOverGifts, HeldOverText, HeldStock, Preferences,
    SpentEnergy, StockCell, TakesGifts, TakesItemText, TookItem, TouchGesture, UiPointer,
};
use crate::connection::ToServer;
use crate::widgets::{ButtonPressed, Floating, RaisePanel};

/// Whether a press and release were a click rather than a drag.
pub(super) fn is_click(from: Vec2, to: Vec2, seconds: f64) -> bool {
    seconds < CLICK_SECONDS && from.distance(to) < CLICK_PIXELS
}

/// Whether it is worth asking to wear something: the server may know of magic this
/// client doesn't, which would make it fit.
pub(super) fn worth_wearing(check: Result<Fit, WearRefused>) -> bool {
    check.err().is_none_or(WearRefused::worth_trying)
}

/// What the pointer may be over: grids, and the bin, the ground, the body and
/// people to give to.
#[derive(SystemParam)]
#[allow(clippy::type_complexity)]
pub(super) struct Under<'w, 's> {
    grids: Query<
        'w,
        's,
        (
            &'static super::panels::GridNode,
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static InheritedVisibility,
            &'static ComputedStackIndex,
        ),
    >,
    nodes: Query<
        'w,
        's,
        (
            &'static ComputedStackIndex,
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static InheritedVisibility,
            Has<BinNode>,
            Has<DropNode>,
            Has<BodyNode>,
            Option<&'static GiveRow>,
            Option<&'static TakesGifts>,
            Option<&'static TakesItemText>,
            Option<&'static CalculatedClip>,
        ),
        Without<Floating>,
    >,
    /// How big stacks may get.
    stacks: Res<'w, super::Stacks>,
    god: Res<'w, crate::common::GodMode>,
}

/// What letting go of something tells the server and the rest of the client.
#[derive(SystemParam)]
pub(super) struct Outbox<'w> {
    to_server: MessageWriter<'w, ToServer>,
    gave: MessageWriter<'w, GaveItem>,
    took: MessageWriter<'w, TookItem>,
    spent: MessageWriter<'w, SpentEnergy>,
    stacks: Res<'w, super::Stacks>,
    on_text: (ResMut<'w, HeldOverText>, MessageWriter<'w, DroppedOnText>),
    /// Why a drop on the clothes was refused, and the rule it goes by.
    wear: (
        ResMut<'w, super::wear_note::WearNote>,
        Res<'w, crate::common::GodMode>,
        Res<'w, Time<Real>>,
    ),
}

/// Everything about holding: picking up with the mouse, turning, following the
/// pointer and letting go. In `PreUpdate`, so what it takes reaches nothing else.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn hold_items(
    mut pressed: MessageReader<ButtonPressed>,
    cells: (
        Query<&ItemCell>,
        Query<&StockCell>,
        Query<&SplitCell>,
        Query<&PickRow>,
    ),
    under: Under,
    (pointer, ui_scale): (Res<UiPointer>, Res<UiScale>),
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut gesture: ResMut<TouchGesture>,
    mut holding: ResMut<Holding>,
    mut carried: ResMut<Carried>,
    mut open: ResMut<OpenGrids>,
    mut commands: Commands,
    mut raise: MessageWriter<RaisePanel>,
    mut out: Outbox,
    (mut over_gifts, mut held_stock, view, mut keys, mut split, preferences): (
        ResMut<HeldOverGifts>,
        ResMut<HeldStock>,
        Res<State<GameView>>,
        ResMut<ButtonInput<KeyCode>>,
        ResMut<Split>,
        Res<Preferences>,
    ),
    time: Res<Time<Real>>,
    (mut last_press, mut was_down): (Local<Option<(shared::id::Id, f64)>>, Local<bool>),
) {
    let now = time.elapsed_secs_f64();
    // Part of a stack lifted out (see `split`) goes back once nothing holds it.
    if holding.held.is_none() {
        holding.group = None;
        if carried.bypass_change_detection().0.settle_parts() {
            carried.set_changed();
        }
    }
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    // Widgets may have taken `just_pressed`.
    let new_press = mouse.pressed(MouseButton::Left) && !*was_down;
    *was_down = mouse.pressed(MouseButton::Left);
    let done = take_presses(
        &mut pressed,
        (&cells.0, &cells.1, &cells.2, &cells.3),
        &mut split,
        &mut mouse,
        &mut holding,
        &carried,
        &mut open,
        &mut commands,
        &mut raise,
        &mut last_press,
        (now, new_press, gesture.tap, pointer.0, shift),
    );
    if done {
        return;
    }
    // Holding part of a stack: it lies on its own while held, even after the server
    // has said what is carried again.
    if let Some(held) = holding.held
        && holding.stock.is_none()
    {
        match &holding.group {
            Some((stack, items)) => {
                let lifted = carried.0.find(held.item.id).is_some_and(|item| {
                    item.count() as usize == items.len()
                        && item.members().all(|one| items.contains(&one.id))
                });
                if !lifted {
                    carried.0.lift_group(*stack, items);
                }
            }
            None if carried.0.find(held.item.id).is_none() => {
                carried.0.lift_part(held.item.id);
            }
            None => {}
        }
    }
    let Some((mut held, aim, at, finger)) = aim_held(
        &mut holding,
        &carried,
        &under,
        (&pointer, &ui_scale, &view),
        &mut mouse,
        &mut gesture,
        (&mut keys, &preferences.controls),
        (&mut over_gifts, &mut held_stock, &mut out.on_text.0),
    ) else {
        return;
    };

    let released = !mouse.pressed(MouseButton::Left);
    if held.by == Hand::Mouse
        && released
        && let (Some((from, when)), Some(at)) = (holding.picked, at)
        // In the window's pixels, so a click is the same whatever the size.
        && is_click(from * ui_scale.0, at * ui_scale.0, now - when)
    {
        held.by = Hand::Clicked;
    }
    let let_go = match held.by {
        Hand::Mouse => released,
        Hand::Clicked => new_press,
        Hand::Finger => finger.is_none(),
        Hand::Debug => false,
    };
    if !let_go {
        holding.held = Some(held);
        holding.aim = Some(aim);
        return;
    }
    holding.held = None;
    holding.aim = None;
    if let Some(on) = out.on_text.0.0.take() {
        // Written in where it is let go; it goes back where it was.
        out.on_text.1.write(DroppedOnText {
            on,
            item: held.item.id,
        });
        holding.stock = None;
        held_stock.set_if_neq(HeldStock(None));
        return;
    }
    let_go_of(
        held,
        aim,
        &mut holding,
        &mut carried,
        (&over_gifts, &mut held_stock),
        &mut out,
    );
}
