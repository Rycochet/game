//! Debug actions on the panels, for `dev_tools`.

use bevy::prelude::*;
use shared::item::Holder;
use shared::protocol::ClientMessage;

use super::give::NearbyPanel;
use super::hold::Held;
use super::panels::OpenGrids;
use super::{Carried, Hand, HeldBy, Holding, InventoryDebug};
use crate::common::GameView;
use crate::connection::ToServer;
use crate::widgets::RaisePanel;

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
pub(super) fn run_debug(
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
            InventoryDebug::OpenInventory => {
                open.open(Holder::Inventory, &carried.0, &mut commands, &mut raise);
                nearby.open(&mut commands, &mut raise, &nodes);
            }
            InventoryDebug::OpenBags => {
                for placed in &carried.0.items {
                    if placed.item.kind.def().container.is_some() {
                        let bag = Holder::Inside(placed.item.id);
                        open.open(bag, &carried.0, &mut commands, &mut raise);
                    }
                }
            }
            InventoryDebug::DropSome => {
                for placed in carried.0.items.iter().skip(1) {
                    let item = placed.item.id;
                    to_server.write(ToServer(ClientMessage::DropItem { item }));
                }
            }
            InventoryDebug::PickUp(index) => {
                if let Some(placed) = carried.0.items.get(index) {
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
