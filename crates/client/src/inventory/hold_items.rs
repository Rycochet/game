//! Holding things, as far as it is the inventory's: what a press picks up
//! (`press`), what letting go where it is would do (`aim`) and doing that
//! (`let_go`). Following the pointer, turning and when it is let go are `dom`'s
//! (`HeldObject`), which runs between `hold_items` and `aim_items`.

mod aim;
mod let_go;
mod press;

use aim::aim_at;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use let_go::let_go_of;
use press::take_presses;
use shared::wear::{Fit, WearRefused};

use super::body::BodyNode;
use super::give::GiveRow;
use super::panels::{BinNode, DropNode, GridOwner, ItemCell, OpenGrids};
use super::split::{PickRow, Split, SplitCell};
use super::{Carried, HeldBy, Holding};
use crate::common::{
    DroppedOnText, GameView, GaveItem, HeldOverGifts, HeldOverText, HeldStock, SpentEnergy,
    StockCell, TakesGifts, TakesItemText, TookItem, TouchGesture, UiPointer,
};
use crate::connection::ToServer;
use crate::dom::Dom;
use crate::dom::{Dropped, HeldObject};
use crate::widgets::ButtonPressed;

/// Whether it is worth asking to wear something: the server may know of magic this
/// client doesn't, which would make it fit.
pub(super) fn worth_wearing(check: Result<Fit, WearRefused>) -> bool {
    check.err().is_none_or(WearRefused::worth_trying)
}

/// What the thing in front may be: a grid, or the bin, the ground, the body, people
/// to give to and text to write an item into.
#[derive(SystemParam)]
#[allow(clippy::type_complexity)]
pub(super) struct Under<'w, 's> {
    grids: Query<'w, 's, &'static GridOwner>,
    marks: Query<
        'w,
        's,
        (
            Has<BinNode>,
            Has<DropNode>,
            Has<BodyNode>,
            Option<&'static GiveRow>,
            Option<&'static TakesGifts>,
            Option<&'static TakesItemText>,
        ),
    >,
    parents: Query<'w, 's, &'static ChildOf>,
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
    dressing: MessageWriter<'w, crate::common::Dressing>,
    stacks: Res<'w, super::Stacks>,
    on_text: (ResMut<'w, HeldOverText>, MessageWriter<'w, DroppedOnText>),
    /// Why a drop on the clothes was refused, and the rule it goes by.
    wear: (
        ResMut<'w, super::wear_note::WearNote>,
        Res<'w, crate::common::GodMode>,
        Res<'w, Time<Real>>,
    ),
}

/// Picking up with the mouse, and keeping what is held as it should be. In
/// `PreUpdate`, so what it takes reaches nothing else, before `dom` moves what
/// is held.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn hold_items(
    mut pressed: MessageReader<ButtonPressed>,
    cells: (
        Query<&ItemCell>,
        Query<&StockCell>,
        Query<&SplitCell>,
        Query<&PickRow>,
    ),
    pointer: Res<UiPointer>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    gesture: Res<TouchGesture>,
    (mut holding, mut object): (ResMut<Holding>, ResMut<HeldObject>),
    mut carried: ResMut<Carried>,
    mut open: ResMut<OpenGrids>,
    mut commands: Commands,
    mut dom: Dom,
    keys: Res<ButtonInput<KeyCode>>,
    mut split: ResMut<Split>,
    time: Res<Time<Real>>,
    (mut last_press, mut was_down): (Local<Option<(shared::id::Id, f64)>>, Local<bool>),
) {
    let now = time.elapsed_secs_f64();
    // Part of a stack lifted out (see `split`) goes back once nothing holds it.
    if HeldBy::of(&object).is_none() {
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
        (&mut holding, &mut object),
        &carried,
        &mut open,
        &mut commands,
        &mut dom,
        &mut last_press,
        (now, new_press, gesture.tap, pointer.0, shift),
    );
    if done {
        return;
    }
    let Some(held) = HeldBy::of(&object) else {
        return;
    };
    // Holding part of a stack: it lies on its own while held, even after the server
    // has said what is carried again.
    if holding.stock.is_none() {
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
    match holding.item(&carried.0) {
        // What is drawn following the pointer is the thing as it is now.
        Some(item) => object.keep_up(item),
        // The server says it has gone.
        None => holding.put_back(&mut object),
    }
}

/// What letting go of what is held would do, and doing it when `dom` says it
/// was let go. In `PreUpdate`, after `dom` has moved it.
#[allow(clippy::too_many_arguments)]
pub(super) fn aim_items(
    object: Res<HeldObject>,
    mut dropped: MessageReader<Dropped>,
    under: Under,
    view: Res<State<GameView>>,
    mut holding: ResMut<Holding>,
    mut carried: ResMut<Carried>,
    (mut over_gifts, mut held_stock): (ResMut<HeldOverGifts>, ResMut<HeldStock>),
    mut out: Outbox,
) {
    over_gifts.0 = None;
    out.on_text.0.set_if_neq(HeldOverText(None));
    // Noted for what draws by them.
    let held = HeldBy::of(&object);
    holding.held = held;
    holding.at = object.at;
    let from_stock = holding.stock.is_some();
    let stock = held.filter(|_| from_stock).map(|held| held.item.id);
    held_stock.set_if_neq(HeldStock(stock));
    // (An ability let go is `abilities`'.)
    let let_go = dropped.read().filter_map(|drop| {
        let held = HeldBy::from_thing(&drop.thing)?;
        Some((held, drop.at, drop.over))
    });
    let let_go = let_go.last();
    let now = held.map(|held| (held, object.at, object.over));
    let Some((subject, at, over)) = now.or(let_go) else {
        holding.aim = None;
        holding.stock = None;
        return;
    };
    let aim = aim_at(
        subject.item,
        (at, over),
        &holding,
        &carried,
        &under,
        &view,
        (&mut over_gifts, &mut out.on_text.0),
    );
    if held.is_some() {
        holding.aim = Some(aim);
        return;
    }
    holding.aim = None;
    if let Some(on) = out.on_text.0.0.take() {
        // Written in where it is let go; it goes back where it was.
        out.on_text.1.write(DroppedOnText {
            on,
            item: subject.item.id,
        });
        holding.stock = None;
        held_stock.set_if_neq(HeldStock(None));
        return;
    }
    let_go_of(
        subject,
        aim,
        &mut holding,
        &mut carried,
        (&over_gifts, &mut held_stock),
        &mut out,
    );
}
