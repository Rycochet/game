//! What the server says of keepers and stock, the grids, and gifts let go on them.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use shared::id::Id;
use shared::protocol::ServerMessage;

use super::{AT, COLUMNS, Heading, Keepers, NEXT_AT, OpenTrade, OpenTrades};
use crate::common::{CurrentDungeon, GROUND, GaveItem, HeldOverGifts, HeldStock, TookItem};
use crate::connection::FromServer;
use crate::dom::{DocumentOf, Template, document_in};
use crate::html;
use crate::widgets::{
    RaisePanel, StockAim, StockAimAt, StockGrid, StockGridNode, UiId, draw_stock_aim,
    spawn_moveable_panel,
};

/// A panel's contents: the stock in a grid, scrolling when there is a lot. The
/// panel's title says whose it is, and stock weighs nothing to anyone. The
/// grid's columns are `COLUMNS` (a test checks).
pub(super) static TRADE: Template = html!(
    "trade",
    r#"
<form id="trade">
  <scroll id="scroll" max-height="70%">
    <object-grid id="stock" type="item" places="packed" stacks="false" columns="12"/>
  </scroll>
</form>
"#
);

/// Gives each panel's grid, once built, to its trade, to be drawn in.
pub(super) fn bind_grids(new: Query<(Entity, &UiId), Added<UiId>>, mut open: ResMut<OpenTrades>) {
    for (entity, id) in &new {
        let (("trade", "stock"), Some(with)) = (id.parts(), id.which()) else {
            continue;
        };
        if let Some(trade) = open.0.iter_mut().find(|trade| trade.grid.with == with) {
            trade.holder = Some(entity);
            trade.grid.changed = true;
        }
    }
}

/// Notes where things given were let go, for when they come back as stock, moves
/// what is theirs already, and takes away what was taken.
pub(super) fn note_gifts(
    mut gave: MessageReader<GaveItem>,
    mut took: MessageReader<TookItem>,
    grids: Query<(&StockGridNode, &ComputedNode, &UiGlobalTransform)>,
    mut open: ResMut<OpenTrades>,
) {
    for GaveItem(gift) in gave.read() {
        let grid = grids.iter().find(|(grid, ..)| grid.0 == gift.to);
        let trade = open.0.iter_mut().find(|trade| trade.grid.with == gift.to);
        if let (Some((_, node, transform)), Some(trade)) = (grid, trade) {
            trade.grid.gave(gift, node, transform);
        }
    }
    for took in took.read() {
        let trade = open.0.iter_mut().find(|trade| trade.grid.with == took.with);
        if let Some(trade) = trade {
            trade.grid.took(took.item);
        }
    }
}

/// Draws grids again when what is in them changes, or something is picked up from
/// them or put back. What is held isn't drawn where it was.
pub(super) fn draw_grids(
    held: Res<HeldStock>,
    mut open: ResMut<OpenTrades>,
    mut commands: Commands,
    mut drawn_without: Local<Option<Id>>,
) {
    let picked_up = *drawn_without != held.0;
    *drawn_without = held.0;
    for trade in &mut open.0 {
        // (Not before its panel's markup is built.)
        let Some(holder) = trade.holder else {
            continue;
        };
        if trade.grid.changed || picked_up {
            trade.grid.changed = false;
            commands.entity(holder).despawn_related::<Children>();
            trade.grid.draw(&mut commands, holder, held.0);
        }
    }
}

/// Who keeps what, and what the server says someone has: their panel, drawn again
/// if it is open, keeping things where they are. A new level closes every panel.
#[allow(clippy::too_many_arguments)]
pub(super) fn read_server(
    mut from_server: MessageReader<FromServer>,
    dungeon: Res<CurrentDungeon>,
    nodes: Query<(), With<Node>>,
    mut keepers: ResMut<Keepers>,
    mut open: ResMut<OpenTrades>,
    mut heading: ResMut<Heading>,
    mut raise: MessageWriter<RaisePanel>,
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
                for trade in open.0.drain(..) {
                    if nodes.contains(trade.panel) {
                        commands.entity(trade.panel).despawn();
                    }
                }
            }
            ServerMessage::Keepers(list) => keepers.0 = list.iter().copied().collect(),
            ServerMessage::TradeStock { with, items } => {
                let trade = open.0.iter_mut().find(|trade| trade.grid.with == *with);
                if let Some(trade) = trade {
                    trade.grid.heard(items);
                    continue;
                }
                let building = keepers.0.iter().find(|(_, keeper)| *keeper == with);
                let look = building.and_then(|(&id, _)| {
                    dungeon.buildings.iter().find(|building| building.id == id)
                });
                let Some(look) = look else {
                    continue;
                };
                let at = AT + NEXT_AT * open.0.len() as f32;
                let (panel, content) = spawn_moveable_panel(&mut commands, look.kind.name(), at);
                commands
                    .entity(panel)
                    .insert(UiId::of("trade", *with, "panel"));
                // Its contents are one of several documents alike, by who
                // it is with: `trade#<with>/stock`.
                commands.spawn((document_in(&TRADE, content), DocumentOf(*with)));
                raise.write(RaisePanel(panel));
                open.0.push(OpenTrade {
                    panel,
                    place: look.door,
                    holder: None,
                    grid: StockGrid::new(*with, COLUMNS, items),
                });
            }
            _ => {}
        }
    }
}

/// Green where something held would go if let go now.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn show_gift_aim(
    held: Res<HeldOverGifts>,
    carried: Res<crate::inventory::Carried>,
    open: Res<OpenTrades>,
    grids: Query<(&StockGridNode, &ComputedNode, &UiGlobalTransform)>,
    aims: Query<(Entity, &StockAim)>,
    new_aims: Query<(), Added<StockAim>>,
    mut commands: Commands,
    mut drawn: Local<Option<StockAimAt>>,
) {
    let aim = held.0.and_then(|gift| {
        let trade = open.0.iter().find(|trade| trade.grid.with == gift.to)?;
        let (_, node, transform) = grids.iter().find(|(grid, ..)| grid.0 == gift.to)?;
        trade.grid.aim(&gift, &carried.0, node, transform)
    });
    if *drawn == aim && new_aims.is_empty() {
        return;
    }
    *drawn = aim;
    // The ground's are `inventory`'s.
    let ours = aims.iter().filter(|(_, aim)| aim.0 != GROUND);
    draw_stock_aim(&mut commands, ours, aim);
}
