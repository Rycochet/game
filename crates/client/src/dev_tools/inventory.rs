//! Bags of every look, and a few items in the inventory, on starting.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::item::{Favour, ItemKind, Quality};
use shared::protocol::ClientMessage;
use std::env;

use crate::common::{GameView, Truesight};
use crate::connection::ToServer;
use crate::ground::GroundDebug;
use crate::inventory::InventoryDebug;

/// `HOW_DEBUG_BAGS` and `HOW_DEBUG_INVENTORY` (with `HOW_DEBUG_HOLD`, `HOW_DEBUG_DROP`, `HOW_DEBUG_SECRETS`, `HOW_DEBUG_CREATURE`).
pub(super) fn add(app: &mut App) {
    if server::option_on("HOW_DEBUG_BAGS", false) {
        app.add_systems(
            Update,
            |time: Res<Time<Real>>,
             view: Res<State<GameView>>,
             mut done: Local<bool>,
             mut debug: MessageWriter<GroundDebug>| {
                if !*done && *view.get() == GameView::Dungeon && time.elapsed_secs() > 2.0 {
                    *done = true;
                    debug.write(GroundDebug::ShowLooks);
                }
            },
        );
    }

    if server::option_on("HOW_DEBUG_INVENTORY", false) {
        let hold = env::var("HOW_DEBUG_HOLD").ok().and_then(|n| n.parse().ok());
        let drop_some = server::option_on("HOW_DEBUG_DROP", false);
        let secrets = server::option_on("HOW_DEBUG_SECRETS", false);
        let creature = env::var("HOW_DEBUG_CREATURE")
            .ok()
            .and_then(|n| n.parse::<usize>().ok())
            .and_then(|n| CreatureKind::ALL.get(n).copied());
        app.add_systems(
            Update,
            move |time: Res<Time<Real>>,
                  view: Res<State<GameView>>,
                  mut step: Local<u8>,
                  mut truesight: ResMut<Truesight>,
                  mut to_server: MessageWriter<ToServer>,
                  mut debug: MessageWriter<InventoryDebug>| {
                if *view.get() == GameView::Connecting {
                    return;
                }
                if *step == 0 {
                    *step = 1;
                    if secrets {
                        truesight.0 = true;
                        to_server.write(ToServer(ClientMessage::DevTruesight(true)));
                        let curse = Some(Favour::Cursed);
                        let bless = Some(Favour::Blessed);
                        for (kind, magical, favour) in [
                            (ItemKind::Ring, true, None),
                            (ItemKind::Ring, false, bless),
                            (ItemKind::Ring, false, curse),
                            (ItemKind::Ring, true, curse),
                            (ItemKind::Hat, true, None),
                            (ItemKind::Hat, false, bless),
                            (ItemKind::Socks, false, curse),
                        ] {
                            to_server.write(ToServer(ClientMessage::DevGiveItem {
                                kind,
                                size: None,
                                magical,
                                quality: Quality::Rare,
                                set: false,
                                favour,
                            }));
                        }
                    }
                    if let Some(kind) = creature {
                        to_server.write(ToServer(ClientMessage::DevSetCreature(kind)));
                    }
                    for kind in [ItemKind::Backpack, ItemKind::Pouch, ItemKind::Boomerang] {
                        to_server.write(ToServer(ClientMessage::DevGiveItem {
                            kind,
                            size: None,
                            magical: false,
                            quality: Default::default(),
                            set: false,
                            favour: None,
                        }));
                    }
                } else if *step == 1 && time.elapsed_secs() > 2.0 {
                    *step = 2;
                    if drop_some {
                        debug.write(InventoryDebug::DropSome);
                    }
                    debug.write(InventoryDebug::OpenInventory);
                    match hold {
                        Some(index) => debug.write(InventoryDebug::PickUp(index)),
                        None => debug.write(InventoryDebug::OpenBags),
                    };
                }
            },
        );
    }
}
