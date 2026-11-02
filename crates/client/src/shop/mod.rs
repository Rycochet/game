//! A town shop's panel: clicking a shop's door opens it, walking there first if
//! the player isn't beside it. What is for sale comes from the server, which only
//! tells someone at the door. See `docs/context/client/shop.md`.

use bevy::prelude::*;
use shared::item::{Item, Quality};
use shared::protocol::{ClientMessage, ServerMessage};
use shared::shop::{DOOR_REACH, sells};
use shared::site::BuildingLook;

use crate::common::{
    CurrentDungeon, GameView, ITEM_NAME_BACKGROUND, MapPosition, NO_QUALITY_NAME, PointerSystems,
    quality_colour,
};
use crate::connection::{FromServer, ToServer};
use crate::dungeon_map::DoorClicked;
use crate::player::{Arrived, Player, PlayerSystems, TakeStep, TravelTo};
use crate::widgets::{BUTTON_PADDING, INK, RaisePanel, WidgetSystems, spawn_moveable_panel};

/// To the left, leaving the player in view.
const AT: Vec2 = Vec2::new(40.0, 120.0);
/// How many things a column has, so the best shops' stock fits the screen.
const COLUMN_ROWS: usize = 12;
const FONT: f32 = 18.0;
const ROW_GAP: f32 = 2.0;
const COLUMN_GAP: f32 = 8.0;

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OpenShop>()
            .init_resource::<Heading>()
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
                    show_stock,
                    close_when_left.after(PlayerSystems::Travel),
                ),
            );
    }
}

/// The panel, if open, and the door of the shop it shows.
#[derive(Resource, Default)]
struct OpenShop(Option<(Entity, (i64, i64))>);

/// The shop the player is walking to, to open on arrival: its id, and the cell
/// in front of its door.
#[derive(Resource, Default)]
struct Heading(Option<(u16, (i64, i64))>);

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

/// A click on a shop's door opens it if the player is beside it, or else walks
/// them to it.
fn follow_clicked_door(
    mut clicks: MessageReader<DoorClicked>,
    dungeon: Res<CurrentDungeon>,
    player: Single<&MapPosition, With<Player>>,
    mut heading: ResMut<Heading>,
    mut travel: MessageWriter<TravelTo>,
    mut to_server: MessageWriter<ToServer>,
) {
    for click in clicks.read() {
        let building = dungeon.buildings.iter().find(|b| b.id == click.building);
        let Some(building) = building else {
            continue;
        };
        let front = front_of(building);
        if sells(building.kind).is_empty() {
            // Only somewhere to walk to.
            travel.write(TravelTo {
                x: front.0,
                y: front.1,
            });
        } else if beside(**player, building.door) {
            heading.0 = None;
            to_server.write(ToServer(ClientMessage::OpenShop {
                building: building.id,
            }));
        } else {
            heading.0 = Some((building.id, front));
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
    mut to_server: MessageWriter<ToServer>,
) {
    let Some(&Arrived { x, y }) = arrivals.read().last() else {
        return;
    };
    if let Some((building, front)) = heading.0.take()
        && front == (x, y)
    {
        // After the steps there, so the server has the player at it.
        to_server.write(ToServer(ClientMessage::OpenShop { building }));
    }
}

/// What the server says a shop has: its panel, drawn anew.
fn show_stock(
    mut from_server: MessageReader<FromServer>,
    dungeon: Res<CurrentDungeon>,
    nodes: Query<(), With<Node>>,
    mut open: ResMut<OpenShop>,
    mut raise: MessageWriter<RaisePanel>,
    mut commands: Commands,
) {
    for FromServer(message) in from_server.read() {
        let ServerMessage::ShopStock { building, items } = message else {
            continue;
        };
        let Some(look) = dungeon.buildings.iter().find(|b| b.id == *building) else {
            continue;
        };
        if let Some((panel, _)) = open.0.take()
            && nodes.contains(panel)
        {
            commands.entity(panel).despawn();
        }
        let (panel, content) = spawn_moveable_panel(&mut commands, look.kind.name(), AT);
        commands.entity(content).with_children(|columns| {
            if items.is_empty() {
                columns.spawn((
                    Text::new("Nothing for sale"),
                    TextFont::from_font_size(FONT),
                    TextColor(INK),
                ));
            }
            columns
                .spawn(Node {
                    column_gap: px(COLUMN_GAP),
                    align_items: AlignItems::FlexStart,
                    ..default()
                })
                .with_children(|columns| {
                    for column in sorted(items).chunks(COLUMN_ROWS) {
                        columns
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: px(ROW_GAP),
                                ..default()
                            })
                            .with_children(|rows| {
                                for item in column {
                                    spawn_row(rows, item);
                                }
                            });
                    }
                });
        });
        raise.write(RaisePanel(panel));
        open.0 = Some((panel, look.door));
    }
}

/// Best first, as a shop would show them.
fn sorted(items: &[Item]) -> Vec<&Item> {
    let mut items: Vec<&Item> = items.iter().collect();
    items.sort_by_key(|item| {
        (
            std::cmp::Reverse(item.quality.unwrap_or(Quality::Poor)),
            item.name().to_owned(),
        )
    });
    items
}

fn spawn_row(rows: &mut ChildSpawnerCommands, item: &Item) {
    rows.spawn((
        Node {
            padding: BUTTON_PADDING,
            ..default()
        },
        BackgroundColor(ITEM_NAME_BACKGROUND),
    ))
    .with_child((
        Text::new(item.name()),
        TextFont::from_font_size(FONT),
        TextColor(item.quality.map_or(NO_QUALITY_NAME, quality_colour)),
    ));
}

/// Esc closes the panel rather than opening the system menu.
fn close_on_escape(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut open: ResMut<OpenShop>,
    nodes: Query<(), With<Node>>,
    mut commands: Commands,
) {
    if let Some((panel, _)) = open.0
        && nodes.contains(panel)
        && keys.clear_just_pressed(KeyCode::Escape)
    {
        commands.entity(panel).despawn();
        open.0 = None;
    }
}

/// Walking away or leaving the map closes it, and its close button forgets it.
fn close_when_left(
    view: Res<State<GameView>>,
    player: Single<&MapPosition, With<Player>>,
    nodes: Query<(), With<Node>>,
    mut open: ResMut<OpenShop>,
    mut commands: Commands,
) {
    let Some((panel, door)) = open.0 else {
        return;
    };
    if !nodes.contains(panel) {
        open.0 = None;
    } else if *view.get() != GameView::Dungeon || !beside(**player, door) {
        commands.entity(panel).despawn();
        open.0 = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::id::Id;
    use shared::item::ItemKind;

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
        starts(show_stock);
        starts(close_on_escape);
        starts(close_when_left);
    }

    #[test]
    fn stock_is_shown_best_first() {
        let item = |n, quality| Item {
            quality,
            ..Item::new(Id::for_test(n), ItemKind::Sword)
        };
        let items = [
            item(1, None),
            item(2, Some(Quality::Poor)),
            item(3, Some(Quality::Epic)),
        ];
        let order: Vec<_> = sorted(&items).iter().map(|item| item.quality).collect();
        assert_eq!(order[0], Some(Quality::Epic));
    }
}
