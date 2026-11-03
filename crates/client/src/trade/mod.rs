//! Trading panels: for now a town shop's, opened by clicking its door, walking
//! there first if the player isn't beside it. What there is comes from the server,
//! which only tells someone in reach. See `docs/context/client/trade.md`.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use shared::id::Id;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::shop::DOOR_REACH;
use shared::site::BuildingLook;

use crate::common::{
    CurrentDungeon, GROUND, GameView, GaveItem, HeldOverGifts, HeldStock, MapPosition,
    PointerSystems, TookItem, Truesight,
};
use crate::connection::{FromServer, ToServer};
use crate::dungeon_map::DoorClicked;
use crate::player::{Arrived, Player, PlayerSystems, TakeStep, TravelTo};
use crate::widgets::{
    RaisePanel, StockAim, StockAimAt, StockGrid, StockGridNode, WidgetSystems, draw_stock_aim,
    spawn_moveable_panel, spawn_scroll_area,
};

/// To the left, leaving the player in view.
const AT: Vec2 = Vec2::new(40.0, 120.0);
/// How far along each panel open already puts the next.
const NEXT_AT: Vec2 = Vec2::new(36.0, 36.0);
/// How many cells across a panel's grid is.
const COLUMNS: u8 = 12;
/// The most of the window's height a grid takes, before it scrolls.
const MOST_HEIGHT: Val = Val::Vh(70.0);

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OpenTrades>()
            .init_resource::<Heading>()
            .init_resource::<Keepers>()
            .add_systems(
                PreUpdate,
                close_on_escape
                    .after(PointerSystems)
                    .before(WidgetSystems::Front),
            )
            // After the steps there have gone, so the server has the player at it.
            .add_systems(PostUpdate, open_on_arrival)
            .add_systems(
                Update,
                (
                    (change_of_mind, follow_clicked_door)
                        .chain()
                        .in_set(PlayerSystems::Input),
                    (
                        ask_again_with_truesight,
                        note_gifts,
                        read_server,
                        draw_grids,
                        show_gift_aim,
                    )
                        .chain(),
                    close_when_left.after(PlayerSystems::Travel),
                ),
            );
    }
}

/// A trading panel, with who it is with: never two for the same one.
struct OpenTrade {
    panel: Entity,
    /// What the player must stay near.
    place: (i64, i64),
    /// What the grid goes in.
    holder: Entity,
    grid: StockGrid,
}

/// The panels open, in the order they were opened.
#[derive(Resource, Default)]
struct OpenTrades(Vec<OpenTrade>);

impl OpenTrades {
    fn panel_with(&self, with: Id) -> Option<Entity> {
        let open = self.0.iter().find(|open| open.grid.with == with);
        open.map(|open| open.panel)
    }
}

/// Who keeps each shop on this level, by building.
#[derive(Resource, Default)]
struct Keepers(BTreeMap<u16, Id>);

/// The keeper the player is walking to, to open on arrival, and the cell in front
/// of their door.
#[derive(Resource, Default)]
struct Heading(Option<(Id, (i64, i64))>);

/// Whether someone here is near enough to this door. The server checks this too.
fn beside(from: MapPosition, door: (i64, i64)) -> bool {
    (from.x - door.0).abs().max((from.y - door.1).abs()) <= DOOR_REACH
}

fn front_of(building: &BuildingLook) -> (i64, i64) {
    (
        building.door.0 + building.faces.0,
        building.door.1 + building.faces.1,
    )
}

/// Walking anywhere else, or a step of their own: they aren't going to the shop.
fn change_of_mind(
    mut steps: MessageReader<TakeStep>,
    mut journeys: MessageReader<TravelTo>,
    mut heading: ResMut<Heading>,
) {
    let stepped = steps.read().count() > 0;
    let front = heading.0.map(|(_, front)| front);
    let elsewhere = journeys
        .read()
        .any(|journey| Some((journey.x, journey.y)) != front);
    if stepped || elsewhere {
        heading.0 = None;
    }
}

/// A click on a shop's door brings its panel to the front if it is open, opens it
/// if the player is beside it, or else walks them to it.
#[allow(clippy::too_many_arguments)]
fn follow_clicked_door(
    mut clicks: MessageReader<DoorClicked>,
    dungeon: Res<CurrentDungeon>,
    keepers: Res<Keepers>,
    open: Res<OpenTrades>,
    player: Single<&MapPosition, With<Player>>,
    mut heading: ResMut<Heading>,
    mut travel: MessageWriter<TravelTo>,
    mut raise: MessageWriter<RaisePanel>,
    mut to_server: MessageWriter<ToServer>,
) {
    for click in clicks.read() {
        let building = dungeon.buildings.iter().find(|b| b.id == click.building);
        let Some(building) = building else {
            continue;
        };
        let front = front_of(building);
        // Only somewhere to walk to if it isn't a shop.
        let Some(&with) = keepers.0.get(&building.id) else {
            travel.write(TravelTo {
                x: front.0,
                y: front.1,
            });
            continue;
        };
        if let Some(panel) = open.panel_with(with) {
            raise.write(RaisePanel(panel));
        } else if beside(**player, building.door) {
            heading.0 = None;
            to_server.write(ToServer(ClientMessage::OpenTrade { with }));
        } else {
            heading.0 = Some((with, front));
            travel.write(TravelTo {
                x: front.0,
                y: front.1,
            });
        }
    }
}

/// Truesight changes what the server tells of what is shown here.
fn ask_again_with_truesight(
    truesight: Res<Truesight>,
    open: Res<OpenTrades>,
    mut to_server: MessageWriter<ToServer>,
) {
    if truesight.is_changed() && !truesight.is_added() {
        for trade in &open.0 {
            let with = trade.grid.with;
            to_server.write(ToServer(ClientMessage::OpenTrade { with }));
        }
    }
}

fn open_on_arrival(
    mut arrivals: MessageReader<Arrived>,
    mut heading: ResMut<Heading>,
    open: Res<OpenTrades>,
    mut to_server: MessageWriter<ToServer>,
) {
    let Some(&Arrived { x, y }) = arrivals.read().last() else {
        return;
    };
    if let Some((with, front)) = heading.0.take()
        && front == (x, y)
        && open.panel_with(with).is_none()
    {
        // After the steps there, so the server has the player at it.
        to_server.write(ToServer(ClientMessage::OpenTrade { with }));
    }
}

/// Notes where things given were let go, for when they come back as stock, moves
/// what is theirs already, and takes away what was taken.
fn note_gifts(
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
fn draw_grids(
    held: Res<HeldStock>,
    mut open: ResMut<OpenTrades>,
    mut commands: Commands,
    mut drawn_without: Local<Option<Id>>,
) {
    let picked_up = *drawn_without != held.0;
    *drawn_without = held.0;
    for trade in &mut open.0 {
        if trade.grid.changed || picked_up {
            trade.grid.changed = false;
            commands.entity(trade.holder).despawn_related::<Children>();
            trade.grid.draw(&mut commands, trade.holder, held.0);
        }
    }
}

/// Who keeps what, and what the server says someone has: their panel, drawn again
/// if it is open, keeping things where they are. A new level closes every panel.
#[allow(clippy::too_many_arguments)]
fn read_server(
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
            ServerMessage::EnterLevel(_) => {
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
                let mut holder = Entity::PLACEHOLDER;
                commands.entity(content).with_children(|content| {
                    holder = spawn_scroll_area(content, MOST_HEIGHT);
                });
                raise.write(RaisePanel(panel));
                open.0.push(OpenTrade {
                    panel,
                    place: look.door,
                    holder,
                    grid: StockGrid::new(*with, COLUMNS, items),
                });
            }
            _ => {}
        }
    }
}

/// Green where something held would go if let go now.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show_gift_aim(
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

/// Esc closes the panel opened last rather than opening the system menu.
fn close_on_escape(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut open: ResMut<OpenTrades>,
    nodes: Query<(), With<Node>>,
    mut commands: Commands,
) {
    open.0.retain(|trade| nodes.contains(trade.panel));
    if !open.0.is_empty()
        && keys.clear_just_pressed(KeyCode::Escape)
        && let Some(trade) = open.0.pop()
    {
        commands.entity(trade.panel).despawn();
    }
}

/// Walking out of reach or leaving the map closes a panel, and its close button
/// forgets it.
fn close_when_left(
    view: Res<State<GameView>>,
    player: Single<&MapPosition, With<Player>>,
    nodes: Query<(), With<Node>>,
    mut open: ResMut<OpenTrades>,
    mut commands: Commands,
) {
    let on_map = *view.get() == GameView::Dungeon;
    open.0.retain(|trade| {
        if !nodes.contains(trade.panel) {
            return false;
        }
        let stays = on_map && beside(**player, trade.place);
        if !stays {
            commands.entity(trade.panel).despawn();
        }
        stays
    });
}

#[cfg(test)]
mod tests;
