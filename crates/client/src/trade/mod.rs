//! Trading panels: for now a town shop's, opened by clicking its door, walking
//! there first if the player isn't beside it. What there is comes from the server,
//! which only tells someone in reach. See `docs/context/client/trade.md`.

use std::collections::BTreeMap;

use bevy::prelude::*;
use shared::id::Id;
use shared::item::{Item, ItemKind};
use shared::protocol::{ClientMessage, ServerMessage};
use shared::shop::DOOR_REACH;
use shared::site::BuildingLook;

use crate::common::{CurrentDungeon, GameView, MapPosition, PointerSystems};
use crate::connection::{FromServer, ToServer};
use crate::dungeon_map::DoorClicked;
use crate::player::{Arrived, Player, PlayerSystems, TakeStep, TravelTo};
use crate::widgets::{
    Floating, ItemName, RaisePanel, WidgetSystems, covering, spawn_item_cells, spawn_item_grid,
    spawn_moveable_panel,
};

/// To the left, leaving the player in view.
const AT: Vec2 = Vec2::new(40.0, 120.0);
/// How far along each panel open already puts the next.
const NEXT_AT: Vec2 = Vec2::new(36.0, 36.0);
/// How many cells across a panel's grid is.
const COLUMNS: usize = 12;

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
                    read_server,
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
}

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

/// Who keeps what, and what the server says someone has: their panel, unless it
/// is open already. A new level closes every panel.
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
                if let Some(panel) = open.panel_with(*with) {
                    raise.write(RaisePanel(panel));
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
                let panel = spawn_panel(&mut commands, look.kind.name(), at, items);
                raise.write(RaisePanel(panel));
                open.0.push(OpenTrade {
                    with: *with,
                    panel,
                    place: look.door,
                });
            }
            _ => {}
        }
    }
}

fn spawn_panel(commands: &mut Commands, title: &str, at: Vec2, items: &[Item]) -> Entity {
    let (panel, content) = spawn_moveable_panel(commands, title, at);
    let arranged = arranged(items);
    commands.entity(content).with_children(|content| {
        spawn_item_grid(content, COLUMNS as u8, arranged.rows, (), |grid| {
            grid.spawn((covering(), Floating)).with_children(|layer| {
                for &(item, x, y) in &arranged.places {
                    spawn_stock_item(layer, item, x, y);
                }
            });
        });
    });
    panel
}

fn spawn_stock_item(layer: &mut ChildSpawnerCommands, item: &Item, x: u8, y: u8) {
    let name = ItemName {
        words: item.name().to_owned(),
        quality: item.quality,
    };
    spawn_item_cells(
        layer,
        item.kind,
        item.quality,
        item.kind.def().shape,
        (x as f32, y as f32),
        false,
        |_, _| name.clone(),
    );
}

/// Where things go in a panel's grid, and how many rows it has.
struct Arranged<'a> {
    places: Vec<(&'a Item, u8, u8)>,
    rows: u8,
}

/// The longest side of any kind of item: what the empty rows must have room for,
/// however it is turned.
fn longest_side() -> u8 {
    let sides = ItemKind::ALL
        .iter()
        .flat_map(|kind| kind.def().shape.cells().flat_map(|(x, y)| [x + 1, y + 1]));
    sides.max().unwrap_or(1)
}

/// Oldest first, each in the first place it fits, along each row from the left,
/// with room for the largest thing below it all. Worked out at each opening: places
/// aren't kept.
fn arranged(items: &[Item]) -> Arranged<'_> {
    let mut items: Vec<&Item> = items.iter().collect();
    // Things without a time are older than time was kept.
    items.sort_by_key(|item| (item.made_at.map(|made| made.minute), item.id));
    let mut taken: Vec<[bool; COLUMNS]> = Vec::new();
    let mut places = Vec::new();
    for item in items {
        let cells: Vec<(u8, u8)> = item.kind.def().shape.cells().collect();
        let fits = |x: usize, y: usize, taken: &[[bool; COLUMNS]]| {
            cells.iter().all(|&(cell_x, cell_y)| {
                let (cell_x, cell_y) = (x + cell_x as usize, y + cell_y as usize);
                cell_x < COLUMNS && taken.get(cell_y).is_none_or(|row| !row[cell_x])
            })
        };
        // There is always somewhere, at the latest below everything.
        let place = (0..=taken.len())
            .flat_map(|y| (0..COLUMNS).map(move |x| (x, y)))
            .find(|&(x, y)| fits(x, y, &taken));
        let Some((x, y)) = place else {
            continue;
        };
        for &(cell_x, cell_y) in &cells {
            let row = y + cell_y as usize;
            if taken.len() <= row {
                taken.resize(row + 1, [false; COLUMNS]);
            }
            taken[row][x + cell_x as usize] = true;
        }
        places.push((item, x as u8, y as u8));
    }
    Arranged {
        places,
        rows: taken.len() as u8 + longest_side(),
    }
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
        starts(read_server);
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

    fn place_of(arranged: &Arranged, n: u128) -> (u8, u8) {
        let place = arranged.places.iter().find(|p| p.0.id == Id::for_test(n));
        place.map(|&(_, x, y)| (x, y)).unwrap()
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
        let arranged = arranged(&items);
        let mut seen = std::collections::BTreeSet::new();
        let mut lowest = 0;
        for &(item, x, y) in &arranged.places {
            for (cell_x, cell_y) in item.kind.def().shape.cells() {
                assert!(seen.insert((x + cell_x, y + cell_y)), "overlap");
                assert!(x + cell_x < COLUMNS as u8);
                lowest = lowest.max(y + cell_y + 1);
            }
        }
        assert_eq!(arranged.places.len(), items.len());
        assert_eq!(arranged.rows, lowest + longest_side());
    }

    #[test]
    fn an_empty_grid_has_room_for_the_longest_thing() {
        assert_eq!(arranged(&[]).rows, longest_side());
        assert!(longest_side() >= 5, "a sword");
    }
}
