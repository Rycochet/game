//! What the character carries: grids in moveable panels, and picking things up,
//! holding them and putting them down. See `docs/context/client/inventory.md`.

mod body;
mod give;
mod hold;
mod panels;

use bevy::prelude::*;
use bevy::ui::{ComputedStackIndex, UiGlobalTransform};
use shared::item::{Holder, Inventory};
use shared::protocol::{ClientMessage, ServerMessage};

#[cfg(not(target_arch = "wasm32"))]
use crate::common::GameView;
use crate::common::{PointerPosition, PointerSystems, TouchGesture};
use crate::connection::{FromServer, ToServer};
use crate::widgets::{ButtonPressed, CELL, Floating, RaisePanel, WidgetSystems};
use body::BodyNode;
use give::{GiveRow, NearbyPanel};
use hold::{Held, Target};
use panels::{BinNode, ItemCell, OpenGrids};

/// A press on an item this long after the last one on it opens it.
const DOUBLE_PRESS_SECONDS: f64 = 0.35;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Carried>()
            .init_resource::<Holding>()
            .init_resource::<OpenGrids>()
            .init_resource::<NearbyPanel>()
            .add_systems(Startup, panels::spawn_fixed_parts)
            .add_systems(
                PreUpdate,
                (
                    // Before widgets stop a finger's drag on a panel.
                    pick_up_by_finger
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
    /// Where the pointer last was, for one just lifted.
    at: Option<Vec2>,
}

#[derive(Clone, Copy, Debug)]
struct HeldBy {
    item: Held,
    by: Hand,
}

/// What let go of it drops it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hand {
    /// The left button, or a tap, which arrives as a click.
    Mouse,
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
}

fn read_server(mut from_server: MessageReader<FromServer>, mut carried: ResMut<Carried>) {
    for message in from_server.read() {
        if let ServerMessage::Inventory(inventory) = &message.0 {
            carried.0 = inventory.clone();
        }
    }
}

/// Where a node is on the screen, in logical pixels: its top left and size.
fn logical_rect(node: &ComputedNode, transform: &UiGlobalTransform) -> (Vec2, Vec2) {
    let size = node.size() * node.inverse_scale_factor;
    let centre = transform.translation * node.inverse_scale_factor;
    (centre - size / 2.0, size)
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

/// A finger sliding away from an item picks it up, once for each contact: a tap
/// is a click instead, for opening things.
#[allow(clippy::type_complexity)]
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
    mut holding: ResMut<Holding>,
    carried: Res<Carried>,
    mut decided: Local<Option<Vec2>>,
) {
    if gesture.first_finger.is_none() {
        *decided = None;
    }
    if gesture.drag == Vec2::ZERO || holding.held.is_some() || *decided == Some(gesture.drag_start)
    {
        return;
    }
    *decided = Some(gesture.drag_start);
    if let Some(cell) = item_cell_in_front(gesture.drag_start, nodes.iter())
        && carried.0.covered_by(cell.id).is_none()
    {
        holding.held = Some(HeldBy {
            item: Held {
                id: cell.id,
                rotation: cell.rotation,
                grip: cell.grip,
            },
            by: Hand::Finger,
        });
        gesture.drag = Vec2::ZERO;
    }
}

/// Everything about holding: picking up with the mouse, turning, following the
/// pointer and letting go. In `PreUpdate`, so what it takes reaches nothing else.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn hold_items(
    mut pressed: MessageReader<ButtonPressed>,
    cells: Query<&ItemCell>,
    grids: Query<(
        &panels::GridNode,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
        &ComputedStackIndex,
    )>,
    pointer: Res<PointerPosition>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut gesture: ResMut<TouchGesture>,
    mut holding: ResMut<Holding>,
    mut carried: ResMut<Carried>,
    mut open: ResMut<OpenGrids>,
    mut commands: Commands,
    mut raise: MessageWriter<RaisePanel>,
    mut to_server: MessageWriter<ToServer>,
    time: Res<Time<Real>>,
    mut last_press: Local<Option<(shared::id::Id, f64)>>,
    nodes: Query<
        (
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Has<BinNode>,
            Has<BodyNode>,
            Option<&GiveRow>,
        ),
        Without<Floating>,
    >,
) {
    for press in pressed.read() {
        let Ok(&cell) = cells.get(press.0) else {
            continue;
        };
        let now = time.elapsed_secs_f64();
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
        holding.held = Some(HeldBy {
            item: Held {
                id: cell.id,
                rotation: cell.rotation,
                grip: cell.grip,
            },
            by: Hand::Mouse,
        });
    }

    let Some(mut held) = holding.held else {
        holding.aim = None;
        holding.at = None;
        return;
    };
    if carried.0.find(held.item.id).is_none() {
        // The server says it has gone.
        holding.held = None;
        holding.aim = None;
        return;
    }

    let turn = mouse.just_pressed(MouseButton::Right) || gesture.other_finger_tapped;
    // So the camera doesn't turn with the same press.
    mouse.reset(MouseButton::Right);
    if turn {
        held.item = held.item.turned(&carried.0);
    }
    let finger = gesture.first_finger;
    if held.by == Hand::Finger {
        // Two fingers on the screen here are holding and turning, not moving the
        // camera.
        *gesture = TouchGesture::default();
    }
    let at = match held.by {
        Hand::Finger => finger.or(holding.at),
        Hand::Mouse | Hand::Debug => pointer.0.or(holding.at),
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
    let (on_bin, on_body, on_row) = at
        .and_then(|at| {
            nodes
                .iter()
                .filter(|(_, node, transform, visible, ..)| {
                    visible.get()
                        && node.contains_point(**transform, at / node.inverse_scale_factor)
                })
                .max_by_key(|(stack, ..)| stack.0)
                .map(|(.., is_bin, is_body, row)| (is_bin, is_body, row.map(|row| row.0)))
        })
        .unwrap_or_default();
    let aim = if on_bin {
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
                allowed: carried.0.check_wear(held.item.id).is_ok(),
            }
        }
    } else {
        hold::target(&carried.0, held.item, over)
    };

    let let_go = match held.by {
        Hand::Mouse => !mouse.pressed(MouseButton::Left),
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
    if aim == (Target::Body { allowed: true }) {
        let item = held.item.id;
        if carried.0.wear(item).is_ok() {
            to_server.write(ToServer(ClientMessage::WearItem { item }));
        }
        return;
    }
    if let Target::Someone(to) = aim {
        let item = held.item.id;
        if carried.0.hand_over(item).is_some() {
            to_server.write(ToServer(ClientMessage::GiveItem { item, to }));
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
        if carried.0.move_item(item, to, x, y, rotation).is_ok() {
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
