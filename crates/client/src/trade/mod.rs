//! Trading panels: for now a town shop's, opened by clicking its door, walking
//! there first if the player isn't beside it. What there is comes from the server,
//! which only tells someone in reach. See `docs/context/client/trade.md`.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use shared::creature::Size;
use shared::id::Id;
use shared::item::{Item, ItemKind, Placed, Rotation, Shape};
use shared::protocol::{ClientMessage, ServerMessage};
use shared::shop::DOOR_REACH;
use shared::site::BuildingLook;

use crate::common::{
    CurrentDungeon, GameView, GaveItem, HeldOverGifts, HeldStock, MapPosition, PointerSystems,
    StockCell, TakesGifts, TookItem,
};
use crate::connection::{FromServer, ToServer};
use crate::dungeon_map::DoorClicked;
use crate::player::{Arrived, Player, PlayerSystems, TakeStep, TravelTo};
use crate::widgets::{
    ALLOWED, CELL, Floating, ItemLook, ItemName, RaisePanel, WidgetSystems, at_cell, covering,
    logical_rect, spawn_item_cells, spawn_item_grid, spawn_moveable_panel, spawn_scroll_area,
};

/// To the left, leaving the player in view.
const AT: Vec2 = Vec2::new(40.0, 120.0);
/// How far along each panel open already puts the next.
const NEXT_AT: Vec2 = Vec2::new(36.0, 36.0);
/// How many cells across a panel's grid is.
const COLUMNS: usize = 12;
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
                    (note_gifts, read_server, draw_grids, show_gift_aim).chain(),
                    close_when_left.after(PlayerSystems::Travel),
                ),
            );
    }
}

/// A trading panel, with who it is with: never two for the same one.
struct OpenTrade {
    with: Id,
    panel: Entity,
    /// What the player must stay near.
    place: (i64, i64),
    /// What the grid goes in.
    holder: Entity,
    /// What is in the grid, and where.
    stock: Vec<Placed>,
    /// Gifts on their way, and where they were dropped.
    gifts: Vec<Gift>,
    /// Its grid needs drawing again.
    changed: bool,
}

/// Something given, and where in the grid it was let go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Gift {
    item: Id,
    rotation: Rotation,
    wanted: (i32, i32),
}

/// A panel's grid.
#[derive(Component, Clone, Copy)]
struct TradeGrid(Id);

/// What shows in a grid where something held would go.
#[derive(Component, Clone, Copy)]
struct TradeAim(Id);

/// The panels open, in the order they were opened.
#[derive(Resource, Default)]
struct OpenTrades(Vec<OpenTrade>);

impl OpenTrades {
    fn panel_with(&self, with: Id) -> Option<Entity> {
        let open = self.0.iter().find(|open| open.with == with);
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

/// The cell of this grid an item's corner is nearest.
fn cell_in_grid(corner: Vec2, node: &ComputedNode, transform: &UiGlobalTransform) -> (i32, i32) {
    let (grid, _) = logical_rect(node, transform);
    let cell = ((corner - grid) / CELL).round();
    (cell.x as i32, cell.y as i32)
}

/// Everything in a grid but this.
fn others(stock: &[Placed], item: Id) -> Vec<Placed> {
    let others = stock.iter().filter(|placed| placed.item.id != item);
    others.cloned().collect()
}

/// Notes where things given were let go, for when they come back as stock, moves
/// what is theirs already, and takes away what was taken.
fn note_gifts(
    mut gave: MessageReader<GaveItem>,
    mut took: MessageReader<TookItem>,
    grids: Query<(&TradeGrid, &ComputedNode, &UiGlobalTransform)>,
    mut open: ResMut<OpenTrades>,
) {
    for GaveItem(gift) in gave.read() {
        let grid = grids.iter().find(|(grid, ..)| grid.0 == gift.to);
        let trade = open.0.iter_mut().find(|trade| trade.with == gift.to);
        let (Some((_, node, transform)), Some(trade)) = (grid, trade) else {
            continue;
        };
        let wanted = cell_in_grid(gift.corner, node, transform);
        let rest = others(&trade.stock, gift.item);
        let moved = trade
            .stock
            .iter_mut()
            .find(|placed| placed.item.id == gift.item);
        if let Some(placed) = moved {
            let shape = placed.item.shape().turned(gift.rotation);
            (placed.x, placed.y) = landing(&rest, shape, wanted);
            placed.rotation = gift.rotation;
            trade.changed = true;
        } else {
            trade.gifts.push(Gift {
                item: gift.item,
                rotation: gift.rotation,
                wanted,
            });
        }
    }
    for took in took.read() {
        if let Some(trade) = open.0.iter_mut().find(|trade| trade.with == took.with) {
            trade.stock.retain(|placed| placed.item.id != took.item);
            trade.changed = true;
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
        if trade.changed || picked_up {
            trade.changed = false;
            commands.entity(trade.holder).despawn_related::<Children>();
            draw_grid(&mut commands, trade, held.0);
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
                if let Some(trade) = open.0.iter_mut().find(|trade| trade.with == *with) {
                    trade.stock = restocked(&trade.stock, items, &trade.gifts);
                    trade.gifts.clear();
                    trade.changed = true;
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
                let stock = arranged(items);
                raise.write(RaisePanel(panel));
                open.0.push(OpenTrade {
                    with: *with,
                    panel,
                    place: look.door,
                    holder,
                    stock,
                    gifts: Vec::new(),
                    changed: true,
                });
            }
            _ => {}
        }
    }
}

fn draw_grid(commands: &mut Commands, trade: &OpenTrade, without: Option<Id>) {
    let with = trade.with;
    let gifts = TakesGifts(with);
    commands.entity(trade.holder).with_children(|holder| {
        let grid = (TradeGrid(with), gifts);
        let rows = rows_for(&trade.stock);
        spawn_item_grid(holder, COLUMNS as u8, rows, grid, gifts, |grid| {
            grid.spawn((covering(), Floating)).with_children(|layer| {
                let shown = trade.stock.iter();
                for placed in shown.filter(|placed| Some(placed.item.id) != without) {
                    spawn_stock_item(layer, placed, with);
                }
            });
            grid.spawn((TradeAim(with), covering(), Floating));
        });
    });
}

fn spawn_stock_item(layer: &mut ChildSpawnerCommands, placed: &Placed, with: Id) {
    let name = ItemName {
        words: placed.item.name(),
        quality: placed.item.quality,
        set: placed.item.set,
    };
    spawn_item_cells(
        layer,
        ItemLook::of(&placed.item),
        placed.shape(),
        (placed.x as f32, placed.y as f32),
        true,
        |x, y| {
            (
                name.clone(),
                TakesGifts(with),
                StockCell {
                    with,
                    item: placed.item.clone(),
                    rotation: placed.rotation,
                    grip: (x, y),
                },
            )
        },
    );
}

/// Green where something held would go if let go now.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show_gift_aim(
    held: Res<HeldOverGifts>,
    carried: Res<crate::inventory::Carried>,
    open: Res<OpenTrades>,
    grids: Query<(&TradeGrid, &ComputedNode, &UiGlobalTransform)>,
    aims: Query<(Entity, &TradeAim)>,
    new_aims: Query<(), Added<TradeAim>>,
    mut commands: Commands,
    mut drawn: Local<Option<(Id, (u8, u8), Shape)>>,
) {
    let aim = held.0.and_then(|gift| {
        let trade = open.0.iter().find(|trade| trade.with == gift.to)?;
        let (_, node, transform) = grids.iter().find(|(grid, ..)| grid.0 == gift.to)?;
        let theirs = trade
            .stock
            .iter()
            .find(|placed| placed.item.id == gift.item);
        let item = theirs
            .map(|placed| &placed.item)
            .or(carried.0.find(gift.item))?;
        let shape = item.shape().turned(gift.rotation);
        let wanted = cell_in_grid(gift.corner, node, transform);
        let rest = others(&trade.stock, gift.item);
        Some((gift.to, landing(&rest, shape, wanted), shape))
    });
    if *drawn == aim && new_aims.is_empty() {
        return;
    }
    *drawn = aim;
    for (entity, shown) in &aims {
        commands.entity(entity).despawn_related::<Children>();
        let Some((_, (x, y), shape)) = aim.filter(|aim| aim.0 == shown.0) else {
            continue;
        };
        commands.entity(entity).with_children(|layer| {
            for (cell_x, cell_y) in shape.cells() {
                let (cell_x, cell_y) = ((x + cell_x) as f32, (y + cell_y) as f32);
                // Floating: the grid under them is still what the pointer is on.
                layer.spawn((at_cell(cell_x, cell_y), BackgroundColor(ALLOWED), Floating));
            }
        });
    }
}

/// The longest side of any kind of item at its largest: what the empty rows must
/// have room for, however it is turned.
fn longest_side() -> u8 {
    let shapes = ItemKind::ALL
        .iter()
        .map(|kind| kind.def().shape.scaled(Size::Giant));
    shapes
        .flat_map(|shape| [shape.width, shape.height])
        .max()
        .unwrap_or(1)
}

/// How many rows a grid has: all it holds, and room for the largest thing below.
fn rows_for(stock: &[Placed]) -> u8 {
    let cells = stock.iter().flat_map(|placed| placed.cells());
    let lowest = cells.map(|(_, y)| y + 1).max().unwrap_or(0);
    lowest + longest_side()
}

/// Whether a shape's cells are all free with its corner here.
fn fits(stock: &[Placed], shape: Shape, x: i32, y: i32) -> bool {
    let taken: Vec<(u8, u8)> = stock.iter().flat_map(|placed| placed.cells()).collect();
    x >= 0
        && y >= 0
        && shape.cells().all(|(cell_x, cell_y)| {
            let cell = (x + cell_x as i32, y + cell_y as i32);
            cell.0 < COLUMNS as i32 && !taken.contains(&(cell.0 as u8, cell.1 as u8))
        })
}

/// The first place a shape fits, along each row from the left. There is always one,
/// at the latest below everything.
fn first_place(stock: &[Placed], shape: Shape) -> (u8, u8) {
    let rows = rows_for(stock) as i32;
    (0..rows)
        .flat_map(|y| (0..COLUMNS as i32).map(move |x| (x, y)))
        .find(|&(x, y)| fits(stock, shape, x, y))
        .map_or((0, rows as u8), |(x, y)| (x as u8, y as u8))
}

/// Where something let go here ends up: there if it fits, or else the first place.
fn landing(stock: &[Placed], shape: Shape, wanted: (i32, i32)) -> (u8, u8) {
    if fits(stock, shape, wanted.0, wanted.1) {
        (wanted.0 as u8, wanted.1 as u8)
    } else {
        first_place(stock, shape)
    }
}

/// Oldest first. Things without a time are older than time was kept.
fn oldest_first(items: &[Item]) -> Vec<&Item> {
    let mut items: Vec<&Item> = items.iter().collect();
    items.sort_by_key(|item| (item.made_at.map(|made| made.minute), item.id));
    items
}

/// A grid drawn afresh, each thing in the first place it fits. Worked out at each
/// opening: places aren't kept.
fn arranged(items: &[Item]) -> Vec<Placed> {
    restocked(&[], items, &[])
}

/// The grid after hearing what there is: what was there stays where it was, what
/// was given goes where it was let go if it fits, and the rest in the first place.
fn restocked(old: &[Placed], items: &[Item], gifts: &[Gift]) -> Vec<Placed> {
    let mut stock: Vec<Placed> = old
        .iter()
        .filter(|placed| items.iter().any(|item| item.id == placed.item.id))
        .cloned()
        .collect();
    for item in oldest_first(items) {
        if stock.iter().any(|placed| placed.item.id == item.id) {
            continue;
        }
        let gift = gifts.iter().find(|gift| gift.item == item.id);
        let rotation = gift.map_or(Rotation::None, |gift| gift.rotation);
        let shape = item.shape().turned(rotation);
        let (x, y) = match gift {
            Some(gift) => landing(&stock, shape, gift.wanted),
            None => first_place(&stock, shape),
        };
        stock.push(Placed {
            item: item.clone(),
            x,
            y,
            rotation,
        });
    }
    stock
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
mod tests {
    use super::*;
    use shared::item::MadeAt;
    use shared::world::WorldId;

    #[test]
    fn beside_a_door_includes_diagonals_and_nothing_further() {
        let at = |x, y| MapPosition { x, y };
        assert!(beside(at(1, 1), (0, 0)));
        assert!(beside(at(0, 0), (0, 0)));
        assert!(!beside(at(2, 0), (0, 0)));
    }

    /// Bevy only finds two parameters clashing when a system starts.
    #[test]
    fn no_system_asks_for_what_another_of_its_parameters_has() {
        fn starts<M>(system: impl IntoSystem<(), (), M>) {
            IntoSystem::into_system(system).initialize(&mut World::new());
        }
        starts(change_of_mind);
        starts(follow_clicked_door);
        starts(open_on_arrival);
        starts(note_gifts);
        starts(read_server);
        starts(draw_grids);
        starts(show_gift_aim);
        starts(close_on_escape);
        starts(close_when_left);
    }

    fn made(n: u128, kind: ItemKind, minute: u64) -> Item {
        Item {
            made_at: Some(MadeAt {
                world: WorldId(Id::for_test(0).0),
                minute,
            }),
            ..Item::new(Id::for_test(n), kind)
        }
    }

    fn place_of(stock: &[Placed], n: u128) -> (u8, u8) {
        let placed = stock.iter().find(|p| p.item.id == Id::for_test(n));
        placed.map(|placed| (placed.x, placed.y)).unwrap()
    }

    #[test]
    fn the_oldest_goes_first_along_the_row() {
        let items = [
            made(1, ItemKind::Apple, 30),
            made(2, ItemKind::Apple, 10),
            made(3, ItemKind::Apple, 20),
        ];
        let arranged = arranged(&items);
        assert_eq!(place_of(&arranged, 2), (0, 0));
        assert_eq!(place_of(&arranged, 3), (1, 0));
        assert_eq!(place_of(&arranged, 1), (2, 0));
    }

    #[test]
    fn small_things_fill_the_gaps_left_by_large_ones() {
        // Swords are 1 × 5 and boots 2 × 2: boots that don't fit at the end of the
        // row go under it, leaving a gap at the end for the apple.
        let mut items: Vec<Item> = (0..11)
            .map(|n| made(n, ItemKind::Sword, n as u64))
            .collect();
        items.push(made(20, ItemKind::Boots, 20));
        items.push(made(21, ItemKind::Apple, 21));
        let arranged = arranged(&items);
        assert_eq!(place_of(&arranged, 20), (0, 5));
        assert_eq!(place_of(&arranged, 21), (11, 0));
    }

    #[test]
    fn nothing_overlaps_and_there_is_always_room_below() {
        let items: Vec<Item> = ItemKind::ALL
            .iter()
            .cycle()
            .take(60)
            .enumerate()
            .map(|(n, &kind)| made(n as u128, kind, n as u64))
            .collect();
        let stock = arranged(&items);
        let mut seen = std::collections::BTreeSet::new();
        for placed in &stock {
            for cell in placed.cells() {
                assert!(seen.insert(cell), "overlap");
                assert!(cell.0 < COLUMNS as u8);
            }
        }
        assert_eq!(stock.len(), items.len());
        let lowest = seen.iter().map(|cell| cell.1 + 1).max().unwrap();
        assert_eq!(rows_for(&stock), lowest + longest_side());
    }

    #[test]
    fn an_empty_grid_has_room_for_the_longest_thing() {
        assert_eq!(rows_for(&[]), longest_side());
        assert!(longest_side() >= 5, "a sword");
    }

    #[test]
    fn a_gift_goes_where_it_was_let_go_and_the_grid_grows_to_keep_its_empty_rows() {
        let old = arranged(&[made(1, ItemKind::Apple, 1)]);
        let rows = rows_for(&old);
        let items = [made(1, ItemKind::Apple, 1), made(2, ItemKind::Apple, 2)];
        let gift = Gift {
            item: Id::for_test(2),
            rotation: Rotation::None,
            wanted: (4, 5),
        };
        let stock = restocked(&old, &items, &[gift]);
        assert_eq!(place_of(&stock, 1), (0, 0));
        assert_eq!(place_of(&stock, 2), (4, 5));
        assert!(rows_for(&stock) > rows);
        assert_eq!(rows_for(&stock), 6 + longest_side());
    }

    #[test]
    fn a_gift_let_go_on_something_goes_in_the_first_place_instead() {
        let old = arranged(&[made(1, ItemKind::Apple, 1)]);
        let items = [made(1, ItemKind::Apple, 1), made(2, ItemKind::Apple, 2)];
        let gift = Gift {
            item: Id::for_test(2),
            rotation: Rotation::None,
            wanted: (0, 0),
        };
        assert_eq!(place_of(&restocked(&old, &items, &[gift]), 2), (1, 0));
    }

    #[test]
    fn opening_again_sorts_gifts_in_with_the_rest() {
        let items = [made(2, ItemKind::Apple, 2), made(1, ItemKind::Apple, 1)];
        assert_eq!(place_of(&arranged(&items), 1), (0, 0));
    }
}
