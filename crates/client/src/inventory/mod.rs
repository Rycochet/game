//! What the character carries: grids in moveable panels, and picking things up,
//! holding them and putting them down. See `docs/context/client/inventory.md`.

mod body;
mod give;
mod ground;
mod hold;
mod panels;

use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex, UiGlobalTransform};
use shared::energy::{dressing_cost, handling_cost};
use shared::item::{Holder, Inventory, Item};
use shared::protocol::{ClientMessage, ServerMessage};
use shared::wear::{Fit, Remover, WearRefused};

use crate::common::{
    GROUND, GameView, GaveItem, GodMode, HeldGift, HeldOverGifts, HeldStock, PointerSystems,
    SpentEnergy, StockCell, TakesGifts, TookItem, TouchGesture, UiPointer, in_interface,
};
use crate::connection::{FromServer, ToServer};
use crate::widgets::{
    ButtonPressed, CELL, Floating, RaisePanel, WidgetSystems, logical_rect, shown_at,
};
use body::BodyNode;
use give::{GiveRow, NearbyPanel};
use hold::{Held, Target};
use panels::{BinNode, DropNode, ItemCell, OpenGrids};

/// A press on an item this long after the last one on it opens it.
const DOUBLE_PRESS_SECONDS: f64 = 0.35;
/// A press and release this quick, this close together, is a click: what it picked up
/// stays held until the next.
const CLICK_SECONDS: f64 = 0.3;
const CLICK_PIXELS: f32 = 8.0;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Carried>()
            .init_resource::<Holding>()
            .init_resource::<OpenGrids>()
            .init_resource::<NearbyPanel>()
            .init_resource::<ground::Ground>()
            .init_resource::<HeldOverGifts>()
            .init_resource::<HeldStock>()
            .add_message::<TookItem>()
            .add_message::<OpenCharacter>()
            .add_message::<GaveItem>()
            .add_systems(Startup, panels::spawn_fixed_parts)
            .add_systems(
                PreUpdate,
                (
                    // Before widgets stop a finger's drag on a panel, and before the
                    // menus see Esc.
                    (pick_up_by_finger, cancel_held)
                        .before(WidgetSystems::Presses)
                        .after(PointerSystems),
                    hold_items.after(WidgetSystems::Modal),
                ),
            )
            .add_systems(
                Update,
                (
                    read_server,
                    panels::toggle_pockets,
                    panels::forget_closed,
                    give::forget_closed,
                    panels::show_bins,
                    panels::show_drop,
                    ground::read_server,
                    ground::show_section,
                    ground::show_aim,
                    panels::show_grids,
                    body::show_clothes,
                    body::show_body_aim,
                    give::show_nearby,
                    give::show_give_aim,
                    panels::show_aim,
                    panels::show_held,
                    panels::show_tooltip,
                    panels::show_button,
                )
                    .chain(),
            );
        #[cfg(not(target_arch = "wasm32"))]
        app.add_message::<InventoryDebug>()
            .add_systems(Update, run_debug.before(panels::forget_closed));
    }
}

/// Opens the Character panel, or brings it to the front.
#[derive(Message, Clone, Copy, Debug)]
pub struct OpenCharacter;

/// What the character carries, as last heard from the server plus any move
/// since, which it answers.
#[derive(Resource, Default)]
pub struct Carried(pub Inventory);

/// What is picked up, if anything.
#[derive(Resource, Default)]
struct Holding {
    held: Option<HeldBy>,
    /// What dropping it now would do.
    aim: Option<Target>,
    /// Where the pointer last was, for one just lifted, in the interface's pixels.
    at: Option<Vec2>,
    /// If it is still someone else's: who, and the thing itself.
    stock: Option<(shared::id::Id, Item)>,
    /// Where (in the interface's pixels) and when the mouse picked it up.
    picked: Option<(Vec2, f64)>,
}

impl Holding {
    /// What is held.
    fn item<'a>(&'a self, carried: &'a Inventory) -> Option<&'a Item> {
        match &self.stock {
            Some((_, item)) => Some(item),
            None => carried.find(self.held?.item.id),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct HeldBy {
    item: Held,
    by: Hand,
}

/// What let go of it drops it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hand {
    /// The left button, until it is let go.
    Mouse,
    /// A click, until the next.
    Clicked,
    Finger,
    /// Nothing does (`InventoryDebug::PickUp`).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    Debug,
}

/// For looking at the panels (`dev_tools`).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Message, Clone, Copy, Debug)]
pub enum InventoryDebug {
    OpenPockets,
    /// Opens every bag in the pockets.
    OpenBags,
    /// Picks up the item in the pockets at this index, until the program ends.
    PickUp(usize),
    /// Puts all but the first of what is in the pockets on the ground.
    DropSome,
}

/// Whether a press and release were a click rather than a drag.
fn is_click(from: Vec2, to: Vec2, seconds: f64) -> bool {
    seconds < CLICK_SECONDS && from.distance(to) < CLICK_PIXELS
}

fn read_server(mut from_server: MessageReader<FromServer>, mut carried: ResMut<Carried>) {
    for message in from_server.read() {
        if let ServerMessage::Inventory(inventory) = &message.0 {
            carried.0 = inventory.clone();
        }
    }
}

/// Whether it is worth asking to wear something: the server may know of magic this
/// client doesn't, which would make it fit.
fn worth_wearing(check: Result<Fit, WearRefused>) -> bool {
    check.err().is_none_or(WearRefused::worth_trying)
}

/// The item cell in front of everything else at this point, if one is.
fn item_cell_in_front<'a>(
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

/// Esc puts back what is held, and goes no further.
fn cancel_held(mut keys: ResMut<ButtonInput<KeyCode>>, mut holding: ResMut<Holding>) {
    if holding.held.is_some() && keys.clear_just_pressed(KeyCode::Escape) {
        holding.held = None;
        holding.aim = None;
        holding.stock = None;
    }
}

/// A finger staying still on an item picks it up, once for each contact: a tap shows
/// its name, or opens it, and a finger sliding straight away is scrolling.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn pick_up_by_finger(
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
    mut holding: ResMut<Holding>,
    carried: Res<Carried>,
    god: Res<GodMode>,
    ui_scale: Res<UiScale>,
    mut decided: Local<Option<Vec2>>,
) {
    if gesture.first_finger.is_none() {
        *decided = None;
    }
    if holding.held.is_some_and(|held| held.by == Hand::Finger) {
        // Moving what it holds, not scrolling what is under it.
        gesture.drag = Vec2::ZERO;
    }
    let Some(start) = gesture
        .still_at
        .map(|start| in_interface(start, &ui_scale))
        .filter(|&start| holding.held.is_none() && *decided != Some(start))
    else {
        return;
    };
    *decided = Some(start);
    if let Some(cell) = item_cell_in_front(start, nodes.iter())
        && carried.0.covered_by(cell.id).is_none()
        && (god.0 || carried.0.may_remove(cell.id, Remover::Wearer))
    {
        holding.held = Some(HeldBy {
            item: Held {
                id: cell.id,
                rotation: cell.rotation,
                grip: cell.grip,
            },
            by: Hand::Finger,
        });
        return;
    }
    let front = stock
        .iter()
        .filter(|(_, node, transform, visible, _, clip)| {
            visible.get() && shown_at(start, node, transform, *clip)
        })
        .max_by_key(|(stack, ..)| stack.0);
    if let Some((.., Some(cell), _)) = front {
        holding.held = Some(HeldBy {
            item: Held {
                id: cell.item.id,
                rotation: cell.rotation,
                grip: cell.grip,
            },
            by: Hand::Finger,
        });
        holding.stock = Some((cell.with, cell.item.clone()));
    }
}

/// Everything about holding: picking up with the mouse, turning, following the
/// pointer and letting go. In `PreUpdate`, so what it takes reaches nothing else.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn hold_items(
    mut pressed: MessageReader<ButtonPressed>,
    (cells, stock_cells): (Query<&ItemCell>, Query<&StockCell>),
    grids: Query<(
        &panels::GridNode,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
        &ComputedStackIndex,
    )>,
    (pointer, ui_scale): (Res<UiPointer>, Res<UiScale>),
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut gesture: ResMut<TouchGesture>,
    mut holding: ResMut<Holding>,
    mut carried: ResMut<Carried>,
    mut open: ResMut<OpenGrids>,
    mut commands: Commands,
    mut raise: MessageWriter<RaisePanel>,
    mut to_server: MessageWriter<ToServer>,
    (mut over_gifts, mut gave, mut held_stock, mut took, mut spent, view, mut keys): (
        ResMut<HeldOverGifts>,
        MessageWriter<GaveItem>,
        ResMut<HeldStock>,
        MessageWriter<TookItem>,
        MessageWriter<SpentEnergy>,
        Res<State<GameView>>,
        ResMut<ButtonInput<KeyCode>>,
    ),
    time: Res<Time<Real>>,
    (mut last_press, mut was_down): (Local<Option<(shared::id::Id, f64)>>, Local<bool>),
    nodes: Query<
        (
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Has<BinNode>,
            Has<DropNode>,
            Has<BodyNode>,
            Option<&GiveRow>,
            Option<&TakesGifts>,
            Option<&CalculatedClip>,
        ),
        Without<Floating>,
    >,
) {
    let now = time.elapsed_secs_f64();
    // Widgets may have taken `just_pressed`.
    let new_press = mouse.pressed(MouseButton::Left) && !*was_down;
    *was_down = mouse.pressed(MouseButton::Left);
    let clicked_held = holding
        .held
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
        holding.held = None;
        holding.aim = None;
        open.open(Holder::Inside(id), &carried.0, &mut commands, &mut raise);
        pressed.clear();
        return;
    }
    let tapped = gesture.tap;
    let at_press = pointer.0;
    for press in pressed.read() {
        if clicked_held.is_some() {
            continue;
        }
        if let Ok(cell) = stock_cells.get(press.0) {
            if tapped {
                continue;
            }
            holding.picked = at_press.map(|at| (at, now));
            holding.held = Some(HeldBy {
                item: Held {
                    id: cell.item.id,
                    rotation: cell.rotation,
                    grip: cell.grip,
                },
                by: Hand::Mouse,
            });
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
            open.open(
                Holder::Inside(cell.id),
                &carried.0,
                &mut commands,
                &mut raise,
            );
            continue;
        }
        if tapped {
            continue;
        }
        holding.picked = at_press.map(|at| (at, now));
        holding.held = Some(HeldBy {
            item: Held {
                id: cell.id,
                rotation: cell.rotation,
                grip: cell.grip,
            },
            by: Hand::Mouse,
        });
    }

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
        return;
    };
    if holding.stock.is_none() && carried.0.find(held.item.id).is_none() {
        // The server says it has gone.
        holding.held = None;
        holding.aim = None;
        return;
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
    let finger = gesture.first_finger.map(|at| in_interface(at, &ui_scale));
    if held.by == Hand::Finger {
        // Two fingers on the screen here are holding and turning, not moving the
        // camera.
        *gesture = TouchGesture::default();
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
            Target::Body {
                allowed: worth_wearing(carried.0.check_wear_new(item)),
            }
        } else if on_bin || on_body || on_row.is_some() {
            Target::Nowhere
        } else {
            hold::target_for_new(&carried.0, item, held.item, over)
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
        hold::target(&carried.0, held.item, over)
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
    if let Some((with, item)) = holding.stock.take() {
        held_stock.set_if_neq(HeldStock(None));
        if let Some(gift) = over_gifts.0 {
            // Moved about in their grid: nothing to ask anyone.
            gave.write(GaveItem(gift));
        } else if aim == Target::Bin {
            // Only from the ground: see `aim`.
            let id = held.item.id;
            carried.0.to_bin_new(item);
            to_server.write(ToServer(ClientMessage::BinFromGround { item: id }));
            took.write(TookItem { with, item: id });
        } else if aim == (Target::Body { allowed: true }) {
            let id = held.item.id;
            let grams = item.total_weight();
            to_server.write(ToServer(ClientMessage::WearFromGround { item: id }));
            // Otherwise only the server can say.
            if carried.0.wear_new(item).is_ok() {
                took.write(TookItem { with, item: id });
                spent.write(SpentEnergy(dressing_cost(grams) + handling_cost(grams)));
            }
        } else if let Some((to, x, y, rotation)) = hold::drop_move(&carried.0, held.item, aim) {
            let grams = item.total_weight();
            if carried.0.add(item, to, x, y, rotation).is_ok() {
                let item = held.item.id;
                to_server.write(ToServer(if with == GROUND {
                    ClientMessage::TakeFromGround {
                        item,
                        to,
                        x,
                        y,
                        rotation,
                    }
                } else {
                    ClientMessage::TakeItem {
                        with,
                        item,
                        to,
                        x,
                        y,
                        rotation,
                    }
                }));
                took.write(TookItem { with, item });
                spent.write(SpentEnergy(handling_cost(grams)));
            }
        }
        return;
    }
    if aim == (Target::Body { allowed: true }) {
        let item = held.item.id;
        let grams = carried.0.find(item).map_or(0, Item::total_weight);
        to_server.write(ToServer(ClientMessage::WearItem { item }));
        // Otherwise only the server can say.
        if carried.0.wear(item).is_ok() {
            spent.write(SpentEnergy(dressing_cost(grams)));
        }
        return;
    }
    if aim == (Target::Drop { allowed: true }) {
        let item = held.item.id;
        let cost = carried.0.put_down_cost(item);
        if carried.0.hand_over(item).is_some() {
            to_server.write(ToServer(ClientMessage::DropItem { item }));
            spent.write(SpentEnergy(cost));
        }
        return;
    }
    if let Target::Someone(to) = aim {
        let item = held.item.id;
        let cost = carried.0.put_down_cost(item);
        if carried.0.hand_over(item).is_some() {
            to_server.write(ToServer(if to == GROUND {
                ClientMessage::DropItem { item }
            } else {
                ClientMessage::GiveItem { item, to }
            }));
            spent.write(SpentEnergy(cost));
            if let Some(gift) = over_gifts.0 {
                gave.write(GaveItem(gift));
            }
        }
        return;
    }
    if aim == Target::Bin {
        let item = held.item.id;
        if carried.0.to_bin(item) {
            to_server.write(ToServer(ClientMessage::BinItem { item }));
        }
        return;
    }
    if let Some((to, x, y, rotation)) = hold::drop_move(&carried.0, held.item, aim) {
        // Shown at once; `Inventory` follows, putting it right if the server disagrees.
        let item = held.item.id;
        let cost = carried.0.move_cost(item, to);
        if carried.0.move_item(item, to, x, y, rotation).is_ok() {
            spent.write(SpentEnergy(cost));
            to_server.write(ToServer(ClientMessage::MoveItem {
                item,
                to,
                x,
                y,
                rotation,
            }));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
fn run_debug(
    mut asked: MessageReader<InventoryDebug>,
    carried: Res<Carried>,
    mut holding: ResMut<Holding>,
    mut open: ResMut<OpenGrids>,
    mut nearby: ResMut<NearbyPanel>,
    nodes: Query<(), With<Node>>,
    mut commands: Commands,
    mut raise: MessageWriter<RaisePanel>,
    view: Res<State<GameView>>,
    mut to_server: MessageWriter<ToServer>,
) {
    if *view.get() == GameView::Connecting {
        return;
    }
    for asked in asked.read() {
        match *asked {
            InventoryDebug::OpenPockets => {
                open.open(Holder::Pockets, &carried.0, &mut commands, &mut raise);
                nearby.open(&mut commands, &mut raise, &nodes);
            }
            InventoryDebug::OpenBags => {
                for placed in &carried.0.pockets {
                    if placed.item.kind.def().container.is_some() {
                        let bag = Holder::Inside(placed.item.id);
                        open.open(bag, &carried.0, &mut commands, &mut raise);
                    }
                }
            }
            InventoryDebug::DropSome => {
                for placed in carried.0.pockets.iter().skip(1) {
                    let item = placed.item.id;
                    to_server.write(ToServer(ClientMessage::DropItem { item }));
                }
            }
            InventoryDebug::PickUp(index) => {
                if let Some(placed) = carried.0.pockets.get(index) {
                    holding.held = Some(HeldBy {
                        item: Held {
                            id: placed.item.id,
                            rotation: placed.rotation,
                            grip: (0, 0),
                        },
                        by: Hand::Debug,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_quick_press_in_one_place_is_a_click() {
        let from = Vec2::new(100.0, 100.0);
        assert!(is_click(from, from + Vec2::X, 0.1));
        assert!(!is_click(from, from + Vec2::X, CLICK_SECONDS));
        assert!(!is_click(from, from + Vec2::X * CLICK_PIXELS, 0.1));
    }
}
