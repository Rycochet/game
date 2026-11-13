//! A shop's stock on its panel's Trade tab: what the server says of keepers
//! and stock, the grids, and gifts let go on them.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use shared::id::Id;
use shared::protocol::ServerMessage;

use super::spawn::{close_all, open_panel};
use super::{COLUMNS, Heading, Keepers, OpenTalks, Tab};
use crate::common::{CurrentDungeon, GROUND, GaveItem, HeldOverGifts, HeldStock, TookItem};
use crate::connection::FromServer;
use crate::dom::{Dom, GridShown};
use crate::widgets::{StockGrid, StockGridNode, StockOf, UiId, show_stock_aim};

/// Gives each panel's grid, once built, to its talk, to be drawn in.
pub(super) fn bind_grids(new: Query<(Entity, &UiId), Added<UiId>>, mut open: ResMut<OpenTalks>) {
    for (entity, id) in &new {
        let (("talk", "stock"), Some(key)) = (id.parts(), id.which()) else {
            continue;
        };
        if let Some(talk) = open.by_key(key) {
            talk.holder = Some(entity);
            if let Some(grid) = &mut talk.grid {
                grid.changed = true;
            }
        }
    }
}

/// Notes where things given were let go, for when they come back as stock, moves
/// what is theirs already, and takes away what was taken.
pub(super) fn note_gifts(
    mut gave: MessageReader<GaveItem>,
    mut took: MessageReader<TookItem>,
    grids: Query<(&StockGridNode, &ComputedNode, &UiGlobalTransform)>,
    mut open: ResMut<OpenTalks>,
) {
    for GaveItem(gift) in gave.read() {
        let grid = grids.iter().find(|(grid, ..)| grid.0 == gift.to);
        let talk = open.trading_with(gift.to);
        if let (Some((_, node, transform)), Some(Some(grid))) = (grid, talk.map(|t| &mut t.grid)) {
            grid.gave(gift, node, transform);
        }
    }
    for took in took.read() {
        if let Some(Some(grid)) = open.trading_with(took.with).map(|t| &mut t.grid) {
            grid.took(took.item);
        }
    }
}

/// Draws grids again when what is in them changes, or something is picked up from
/// them or put back. What is held isn't drawn where it was.
pub(super) fn draw_grids(
    held: Res<HeldStock>,
    mut open: ResMut<OpenTalks>,
    mut commands: Commands,
    mut drawn_without: Local<Option<Id>>,
) {
    let picked_up = *drawn_without != held.0;
    *drawn_without = held.0;
    for talk in &mut open.0 {
        // (Not before its panel's markup is built.)
        let (Some(holder), Some(grid)) = (talk.holder, &mut talk.grid) else {
            continue;
        };
        if grid.changed || picked_up {
            grid.changed = false;
            grid.show(&mut commands, holder, held.0);
        }
    }
}

/// Who keeps what, and what the server says someone has: their panel opened
/// at its Trade tab if it isn't open, or its stock caught up, keeping things
/// where they are. A new level closes every panel.
#[allow(clippy::too_many_arguments)]
pub(super) fn read_server(
    mut from_server: MessageReader<FromServer>,
    dungeon: Res<CurrentDungeon>,
    nodes: Query<(), With<Node>>,
    mut keepers: ResMut<Keepers>,
    mut open: ResMut<OpenTalks>,
    mut heading: ResMut<Heading>,
    mut dom: Dom,
    mut commands: Commands,
) {
    for FromServer(message) in from_server.read() {
        match message {
            // (A welcome may be from another server, after logging out.)
            ServerMessage::Welcome { .. }
            | ServerMessage::EnterLevel(_)
            | ServerMessage::EnterHall { .. } => {
                heading.0 = None;
                keepers.0.clear();
                close_all(&mut open, &nodes, &mut commands);
            }
            ServerMessage::Keepers(list) => keepers.0 = list.iter().copied().collect(),
            ServerMessage::TradeStock { with, items } => {
                if let Some(Some(grid)) = open.trading_with(*with).map(|t| &mut t.grid) {
                    grid.heard(items);
                    continue;
                }
                let building = keepers.0.iter().find(|(_, keeper)| *keeper == with);
                let Some((&building, _)) = building else {
                    continue;
                };
                let (open, dungeon) = (&mut open, &dungeon);
                let talk = open_panel(open, building, Tab::Trade, dungeon, &mut dom, &mut commands);
                if let Some(talk) = talk {
                    talk.grid = Some(StockGrid::new(*with, COLUMNS, items));
                    talk.dirty = true;
                }
            }
            _ => {}
        }
    }
}

/// Green where something held would go if let go now.
pub(super) fn show_gift_aim(
    held: Res<HeldOverGifts>,
    carried: Res<crate::inventory::Carried>,
    mut open: ResMut<OpenTalks>,
    grids: Query<(&StockGridNode, &ComputedNode, &UiGlobalTransform)>,
    mut shown: Query<(&StockOf, &mut GridShown)>,
) {
    let aim = held.0.and_then(|gift| {
        let grid = open.trading_with(gift.to)?.grid.as_ref()?;
        let (_, node, transform) = grids.iter().find(|(grid, ..)| grid.0 == gift.to)?;
        grid.aim(&gift, &carried.0, node, transform)
    });
    // The ground's are `inventory`'s.
    let ours = shown.iter_mut().filter(|(of, _)| of.0 != GROUND);
    show_stock_aim(ours, aim);
}
