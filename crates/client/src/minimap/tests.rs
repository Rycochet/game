use bevy::ecs::system::RunSystemOnce;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::window::PrimaryWindow;
use shared::dungeon::Level;
use shared::protocol::{ClientMessage, SeenPoi};

use super::names::words;
use super::*;
use crate::common::{
    ClickMode, CurrentDungeon, CurrentWorld, FirstFloor, GameView, OtherLevels, SmoothPosition,
    TouchGesture, UiPointer,
};
use crate::connection::ToServer;
use crate::player::Player;
use crate::view3d::OrbitCamera;
use crate::widgets::CLOSE_PIXELS;

#[test]
fn ruined_places_are_named_as_ruins() {
    let mut dungeon = CurrentDungeon::default();
    dungeon.enter(shared::protocol::LevelInfo {
        kind: shared::poi::PoiKind::StoneTower,
        ruined: true,
        building: true,
        index: 1,
        levels: 3,
        basement: false,
        bounds: (0, 0, 5, 5),
        arrival: (2, 2),
        entering_site: true,
    });
    assert_eq!(
        words(&dungeon.describe()),
        "Ruined stone tower, floor 2 of 3"
    );
    let player = SmoothPosition::default();
    let title = |first| location_title(GameView::Dungeon, None, &dungeon, first, &player);
    assert_eq!(title(FirstFloor(1)), "Ruined stone tower, floor 2");
    assert_eq!(title(FirstFloor(0)), "Ruined stone tower, floor 1");
}

/// A level with a little remembered, a window, and the player on it.
fn app() -> App {
    let mut app = App::new();
    let level = Level::new(0, 0, 400, 400);
    let mut dungeon = CurrentDungeon {
        known: shared::dungeon::Sight::new(&level),
        remembered: shared::dungeon::Sight::new(&level),
        level,
        ..default()
    };
    dungeon.remembered.insert(100, 100);
    app.insert_resource(dungeon)
        .insert_resource(State::new(GameView::Dungeon))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<AccumulatedMouseScroll>()
        .init_resource::<ClickMode>()
        .init_resource::<UiPointer>()
        .init_resource::<UiScale>()
        .init_resource::<TouchGesture>()
        .init_resource::<OtherLevels>()
        .init_resource::<FullMap>();
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app.world_mut().spawn(OrbitCamera::new(0.5, 100.0, 0.0));
    app.world_mut()
        .spawn((Player, SmoothPosition { x: 100.0, y: 100.0 }));
    app
}

fn step(app: &mut App) {
    app.world_mut()
        .run_system_once(take_input)
        .expect("the system runs");
}

fn set_cursor(app: &mut App, at: Vec2) {
    app.world_mut().resource_mut::<UiPointer>().0 = Some(at);
}

fn press_key(app: &mut App, key: KeyCode) {
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(key);
    keys.clear();
    keys.press(key);
}

/// Presses M, and lets go.
fn open_map(app: &mut App) {
    press_key(app, KeyCode::KeyM);
    step(app);
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(KeyCode::KeyM);
    keys.clear();
    assert!(app.world().resource::<FullMap>().open);
}

/// A world map with the full map open and a poi at its middle, under the
/// mouse, ready for a click in Teleport Mode or not.
fn poi_under_the_mouse(teleporting: bool) -> App {
    let mut app = app();
    let chunk = shared::poi::ChunkPos { x: 1, y: 1 };
    let mut world = CurrentWorld::new(&shared::protocol::WorldInfo {
        id: shared::world::WorldSeed::new(1).id(),
        terrain_seed: shared::rng::Seed::new(1),
        world_map_version: shared::world::WORLD_MAP_VERSION,
    });
    let poi = SeenPoi {
        kind: shared::poi::PoiKind::Crypt,
        look: shared::site::PoiLook::Cave {
            entrance: shared::site::CaveEntrance::Hole,
        },
        chunk,
        x: 100,
        y: 100,
    };
    world.pois.insert(chunk, Some(poi));
    app.insert_resource(world)
        .insert_resource(State::new(GameView::WorldMap))
        .add_message::<ToServer>();
    app.world_mut().resource_mut::<FullMap>().set_open(true);
    let mut windows = app.world_mut().query::<&Window>();
    let window = Room::of(
        windows.single(app.world()).expect("a window"),
        &UiScale::default(),
    );
    let window = &window;
    let corner =
        full_map_corner(window) + Vec2::new(FULL_MAP_MARGIN, FULL_MAP_MARGIN + TITLE_HEIGHT);
    let middle = corner + full_map_area(window) / 2.0;
    set_cursor(&mut app, middle);
    if teleporting {
        *app.world_mut().resource_mut::<ClickMode>() = ClickMode::Teleport;
    }
    let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    mouse.press(MouseButton::Left);
    app.world_mut()
        .run_system_once(go_into_clicked_poi)
        .expect("the system runs");
    app
}

fn sent(app: &mut App) -> Vec<ClientMessage> {
    let mut messages = app.world_mut().resource_mut::<Messages<ToServer>>();
    messages.drain().map(|message| message.0).collect()
}

#[test]
fn a_click_on_a_poi_in_teleport_mode_goes_in_and_closes_the_map() {
    let mut app = poi_under_the_mouse(true);
    assert_eq!(
        sent(&mut app),
        [
            ClientMessage::DevTeleport { x: 100, y: 100 },
            ClientMessage::EnterSite {
                chunk: shared::poi::ChunkPos { x: 1, y: 1 },
            },
        ]
    );
    assert!(!app.world().resource::<FullMap>().open);
    let mouse = app.world().resource::<ButtonInput<MouseButton>>();
    assert!(!mouse.just_pressed(MouseButton::Left));
}

#[test]
fn a_plain_click_on_a_poi_is_left_for_the_map() {
    let mut app = poi_under_the_mouse(false);
    assert!(sent(&mut app).is_empty());
    assert!(app.world().resource::<FullMap>().open);
    let mouse = app.world().resource::<ButtonInput<MouseButton>>();
    assert!(mouse.just_pressed(MouseButton::Left));
}

#[test]
fn m_opens_and_closes_and_esc_closes_without_reaching_the_game() {
    let mut app = app();
    press_key(&mut app, KeyCode::KeyM);
    step(&mut app);
    assert!(app.world().resource::<FullMap>().open);
    press_key(&mut app, KeyCode::KeyM);
    step(&mut app);
    assert!(!app.world().resource::<FullMap>().open);

    // With the map shut, Esc is left for the game.
    press_key(&mut app, KeyCode::Escape);
    step(&mut app);
    let keys = app.world().resource::<ButtonInput<KeyCode>>();
    assert!(keys.just_pressed(KeyCode::Escape));

    open_map(&mut app);
    press_key(&mut app, KeyCode::Escape);
    step(&mut app);
    assert!(!app.world().resource::<FullMap>().open);
    let keys = app.world().resource::<ButtonInput<KeyCode>>();
    assert!(!keys.just_pressed(KeyCode::Escape));
}

#[test]
fn dragging_moves_the_map_only_as_far_as_what_is_known() {
    let mut app = app();
    open_map(&mut app);
    let middle = Vec2::new(640.0, 360.0);
    set_cursor(&mut app, middle);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    step(&mut app);
    // The press is the map's, not the game's.
    let mouse = app.world().resource::<ButtonInput<MouseButton>>();
    assert!(!mouse.just_pressed(MouseButton::Left));

    // A little to the right: the land follows, so the centre goes west.
    set_cursor(&mut app, middle + Vec2::new(11.0, 0.0));
    step(&mut app);
    let (x, y) = app.world().resource::<FullMap>().centre.expect("dragged");
    // (Half a cell: it opens at twice the minimap's scale.)
    assert!((x - 99.5).abs() < 1e-6, "{x}");
    assert_eq!(y, 100.0);

    // Much too far: it stops with the remembered cell still on the map.
    set_cursor(&mut app, middle + Vec2::new(600.0, 0.0));
    step(&mut app);
    let (x, _) = app.world().resource::<FullMap>().centre.expect("dragged");
    assert!(x < 99.5, "{x}");
    assert!(x > 50.0, "{x}");

    // Letting go ends the drag.
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    step(&mut app);
    set_cursor(&mut app, middle);
    step(&mut app);
    assert_eq!(app.world().resource::<FullMap>().centre.expect("kept").0, x);
}

#[test]
fn the_open_map_keeps_the_mouse_from_the_camera() {
    let mut app = app();
    open_map(&mut app);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta = Vec2::Y;
    step(&mut app);
    let mouse = app.world().resource::<ButtonInput<MouseButton>>();
    assert!(!mouse.pressed(MouseButton::Right));
    let scroll = app.world().resource::<AccumulatedMouseScroll>();
    assert_eq!(scroll.delta, Vec2::ZERO);
}

fn click(app: &mut App, at: Vec2) {
    set_cursor(app, at);
    let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    mouse.release(MouseButton::Left);
    mouse.clear();
    mouse.press(MouseButton::Left);
    step(app);
}

#[test]
fn clicking_the_minimap_opens_the_map_and_stays_off_the_land() {
    let mut app = app();
    // Elsewhere, the click is the game's.
    click(&mut app, Vec2::new(10.0, 10.0));
    assert!(!app.world().resource::<FullMap>().open);
    let mouse = app.world().resource::<ButtonInput<MouseButton>>();
    assert!(mouse.just_pressed(MouseButton::Left));

    let size = minimap_picture_size(&Room::of(&Window::default(), &UiScale::default()));
    let on = Vec2::new(1280.0 - CORNER_GAP - size / 2.0, CORNER_GAP + size / 2.0);
    click(&mut app, on);
    assert!(app.world().resource::<FullMap>().open);
    let mouse = app.world().resource::<ButtonInput<MouseButton>>();
    assert!(!mouse.just_pressed(MouseButton::Left));
}

#[test]
fn its_button_and_clicks_outside_close_the_map() {
    let mut app = app();
    let side = 648.0;
    let corner = Vec2::new(1280.0 - side, 720.0 - side) / 2.0;
    let button = corner
        + Vec2::new(
            side - FULL_MAP_MARGIN - CLOSE_PIXELS / 2.0,
            FULL_MAP_MARGIN + CLOSE_PIXELS / 2.0,
        );
    for (at, stays_open) in [
        (corner + Vec2::splat(side / 2.0), true),
        (button, false),
        (corner - Vec2::splat(5.0), false),
    ] {
        app.world_mut().resource_mut::<FullMap>().set_open(true);
        click(&mut app, at);
        assert_eq!(app.world().resource::<FullMap>().open, stays_open, "{at}");
        let mouse = app.world().resource::<ButtonInput<MouseButton>>();
        assert!(!mouse.just_pressed(MouseButton::Left));
    }
}

#[test]
fn the_wheel_zooms_from_twice_the_minimaps_scale_up_to_four_times() {
    let mut app = app();
    open_map(&mut app);
    assert_eq!(app.world().resource::<FullMap>().zoom, OPENING_ZOOM);
    for _ in 0..20 {
        app.world_mut()
            .resource_mut::<AccumulatedMouseScroll>()
            .delta = Vec2::Y;
        step(&mut app);
    }
    assert_eq!(app.world().resource::<FullMap>().zoom, MAX_ZOOM);
    // Opening again starts over.
    app.world_mut().resource_mut::<FullMap>().set_open(true);
    assert_eq!(app.world().resource::<FullMap>().zoom, OPENING_ZOOM);
}

#[test]
fn fingers_on_the_open_map_are_kept_from_the_camera() {
    let mut app = app();
    open_map(&mut app);
    // In a 1280 × 720 window, the map's picture is 608 × 574, at (336, 90).
    let middle = Vec2::new(640.0, 377.0);
    *app.world_mut().resource_mut::<TouchGesture>() = TouchGesture {
        pinch: 1.25,
        twist: 0.5,
        centre: middle + Vec2::new(110.0, 55.0),
        ..default()
    };
    step(&mut app);
    // As the wheel's notch does: the same place stays between the fingers.
    let full = app.world().resource::<FullMap>();
    assert_eq!(full.zoom, OPENING_ZOOM * 1.25);
    let (x, y) = full.centre.expect("moved");
    assert!((x - 101.0).abs() < 1e-6, "{x}");
    assert!((y - 100.5).abs() < 1e-6, "{y}");
    assert_eq!(
        *app.world().resource::<TouchGesture>(),
        TouchGesture::default()
    );

    // One finger drags it: the land follows, so the centre goes the other way.
    app.world_mut().resource_mut::<TouchGesture>().drag = Vec2::new(11.0, 0.0);
    step(&mut app);
    let (dragged, _) = app.world().resource::<FullMap>().centre.expect("moved");
    assert!(dragged < x, "{dragged}");
}

#[test]
fn fingers_are_left_for_the_camera_while_the_map_is_closed() {
    let mut app = app();
    app.world_mut().resource_mut::<TouchGesture>().twist = 0.5;
    step(&mut app);
    assert_eq!(app.world().resource::<TouchGesture>().twist, 0.5);
}

#[test]
fn the_wheel_zooms_towards_the_mouse() {
    let mut app = app();
    open_map(&mut app);
    // In a 1280 × 720 window, the map's picture is 608 × 574, at (336, 90).
    let middle = Vec2::new(640.0, 377.0);
    set_cursor(&mut app, middle + Vec2::new(110.0, 55.0));
    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta = Vec2::Y;
    step(&mut app);
    // From half a cell per 11 pixels to 0.4: the same place stays under
    // the mouse, 5 cells east and 2.5 south before, 4 and 2 after.
    let (x, y) = app.world().resource::<FullMap>().centre.expect("moved");
    assert!((x - 101.0).abs() < 1e-6, "{x}");
    assert!((y - 100.5).abs() < 1e-6, "{y}");
}

/// A second floor beside the app's, known round the player's place.
fn add_floor(app: &mut App) {
    let info = shared::protocol::LevelInfo {
        kind: shared::poi::PoiKind::StoneTower,
        ruined: false,
        building: true,
        index: 0,
        levels: 3,
        basement: false,
        bounds: (0, 0, 400, 400),
        arrival: (100, 100),
        entering_site: true,
    };
    let mut other = CurrentDungeon::default();
    other.enter(shared::protocol::LevelInfo {
        index: 2,
        ..info.clone()
    });
    other.remembered.insert(100, 100);
    app.world_mut().resource_mut::<CurrentDungeon>().info = Some(info);
    let mut levels = app.world_mut().resource_mut::<OtherLevels>();
    levels.insert_for_tests(2, other);
}

#[test]
fn clicking_a_floors_button_on_the_full_map_shows_it_round_the_player() {
    let mut app = app();
    add_floor(&mut app);
    let dungeon = app.world().resource::<CurrentDungeon>();
    let levels = app.world().resource::<OtherLevels>();
    // A building's floors go up: the highest on top.
    let buttons = floors::buttons(GameView::Dungeon, true, dungeon, levels);
    assert_eq!(buttons, [2, 0]);
    // Only on the full map, and only in a site.
    assert!(floors::buttons(GameView::Dungeon, false, dungeon, levels).is_empty());
    assert!(floors::buttons(GameView::WorldMap, true, dungeon, levels).is_empty());
    let window = Window::default();
    let on_button = |row: usize| {
        let (corner, _) = floors::column(&Room::of(&window, &UiScale::default()), 2);
        let down = row as f32 * (floors::BUTTON + floors::BUTTON_GAP);
        corner + Vec2::new(5.0, 5.0 + down)
    };
    app.world_mut().resource_mut::<FullMap>().set_open(true);
    click(&mut app, on_button(0));
    assert_eq!(app.world().resource::<OtherLevels>().showing(), Some(2));

    // A dragged map stays where it was, even with nothing known there.
    app.world_mut().resource_mut::<FullMap>().centre = Some((300.0, 300.0));
    click(&mut app, on_button(1));
    assert_eq!(app.world().resource::<OtherLevels>().showing(), None);
    let full = app.world().resource::<FullMap>();
    assert!(full.open);
    assert_eq!(full.centre, Some((300.0, 300.0)));
    assert_eq!(full.drag_from, None);
    step(&mut app);
    let full = app.world().resource::<FullMap>();
    assert_eq!(full.centre, Some((300.0, 300.0)));
}

#[test]
fn only_shop_doors_are_named_in_sites() {
    let level = Level::new(0, 0, 5, 5);
    let mut dungeon = CurrentDungeon {
        known: shared::dungeon::Sight::new(&level),
        remembered: shared::dungeon::Sight::new(&level),
        level,
        ..default()
    };
    dungeon.remembered.insert(2, 2);
    dungeon.level.set(2, 2, shared::dungeon::Cell::BuildingDoor);
    dungeon.buildings = vec![shared::site::BuildingLook {
        id: 0,
        kind: shared::site::BuildingKind::PotionShop,
        walls: shared::site::WallKind::Stone,
        roof: shared::site::RoofKind::SlateTiles,
        rects: Vec::new(),
        door: (2, 2),
        faces: (0, 1),
    }];
    let name = |x, y| hovered_name(GameView::Dungeon, None, &dungeon, (x, y), 8.0, 0.1);
    assert_eq!(name(2.3, 1.8).as_deref(), Some("Potion shop"));
    assert_eq!(name(0.0, 0.0), None);
}

#[test]
fn names_read_as_words() {
    assert_eq!(
        words("StoneTower, floor 2 of 3"),
        "Stone tower, floor 2 of 3"
    );
    assert_eq!(words("Crypt"), "Crypt");
}

#[test]
fn a_minimised_window_still_gives_the_maps_a_scale() {
    let mut window = Window::default();
    window.resolution.set(800.0, 0.0);
    assert_eq!(minimap_across(&Room::of(&window, &UiScale::default())), 1.0);
}

#[test]
fn the_maps_keep_their_share_of_the_window_at_any_interface_size() {
    let mut window = Window::default();
    window.resolution.set(1280.0, 720.0);
    let on_screen = |scale: f32| {
        let room = Room::of(&window, &UiScale(scale));
        (
            minimap_across(&room) * room.scale_factor(),
            full_map_side(&room) * room.scale_factor(),
        )
    };
    assert_eq!(on_screen(1.0), (144.0, 648.0));
    assert_eq!(on_screen(0.5), on_screen(1.0));
    assert_eq!(on_screen(2.0), on_screen(1.0));
}
