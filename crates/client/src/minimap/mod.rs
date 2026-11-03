//! The maps: the round minimap always in the top-right corner, and the full
//! parchment map that M or clicking the minimap opens. Both show only what is known, drawn on the CPU into
//! a picture. See `docs/context/client/minimap.md`.

mod canvas;
mod floors;
mod frame;
mod grid;
mod projection;
mod sky;
mod sprites;
mod world;

use std::collections::BTreeSet;

use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::image::ImageSampler;
use bevy::input::InputSystems;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use shared::protocol::{ClientMessage, SeenPoi};
use shared::site::StairsDirection;

use crate::common::{
    ClickMode, CurrentDungeon, CurrentWorld, FirstFloor, GameClock, GameView, OtherLevels,
    PointerSystems, Portal, SkyNow, SmoothPosition, Token, TouchGesture, UiPointer, Weather,
    in_interface, interface_area,
};
use crate::connection::ToServer;
use crate::dungeon_map::{TOWN_ATLAS, door_picture};
use crate::player::{Player, PlayerSystems};
use crate::surface_style::SurfaceStyle;
use crate::view3d::{HoveredToken, OrbitCamera, View3dSystems};
use crate::widgets::{
    CLOSE_PIXELS, PARCHMENT, WidgetSystems, darkening, layer_z, leaves_presses, panel, panel_frame,
    panel_title, spawn_close_button, spawn_tooltip,
};
use crate::world_map::{self, poi_picture, poi_picture_name};
use canvas::Canvas;
use frame::RoundFrame;
use grid::{CellLook, Looks};
use projection::Projection;
use sprites::Sprites;
use world::WorldTiles;

/// The minimap's size against the window's height, its border and notch, and
/// its gap from the window's corner. Lengths are in logical pixels.
const MINIMAP_OF_HEIGHT: f32 = 0.2;
const BORDER: f32 = 5.0;
const NOTCH: f32 = 6.0;
/// The window height at which the sizes round the minimap (`BORDER`, `NOTCH`,
/// and the suns and moons) are as given: the height a window opens at.
const MINIMAP_BASE_HEIGHT: f32 = 720.0;
const CORNER_GAP: f32 = 8.0;

/// A grid square, plus the grid line shared with the next one.
const CELL_PIXELS: f32 = 11.0;
/// How big a shop's sign is drawn on its square.
const SIGN_PIXELS: f32 = 10.0;
/// How big stairs are drawn: a little smaller than the player's dot, for more
/// of a gap round them in their square.
const STAIRS_PIXELS: f32 = 8.0;
/// How big a poi's picture is drawn.
const ICON_PIXELS: f32 = 16.0;
/// How solid a completed poi's picture is.
const COMPLETED_OPACITY: f32 = 0.5;

/// The player's dot, across, and its outline; how far inside the map's edge
/// the arrow to it, in the same style and as wide, sits.
const DOT_PIXELS: f32 = 10.0;
const DOT_OUTLINE: f32 = 1.0;
const ARROW_INSET: f32 = 12.0;
/// The little arrow beside them on another floor, saying which way their floor is.
const FLOOR_HINT_PIXELS: f32 = 4.9;
/// The player's dot and arrow on another floor, against their usual size.
const OTHER_FLOOR_SIZE: f32 = 0.8;

/// The full map's parchment against the window's shorter side, the margin
/// left bare on it, and the title's height above the map.
const FULL_MAP_OF_WINDOW: f32 = 0.9;
const FULL_MAP_MARGIN: f32 = 20.0;
const TITLE_HEIGHT: f32 = 34.0;
/// How far round the close button still counts as on it, for fingers.
const CLOSE_REACH: f32 = 10.0;
/// How far the box naming what is under the mouse sits above the margin.
const HOVER_LIFT: f32 = 12.0;

/// The full map's scale against the minimap's: when it opens, and its limits.
const OPENING_ZOOM: f32 = 2.0;
const MIN_ZOOM: f32 = 1.0;
const MAX_ZOOM: f32 = 4.0;
/// Scale per wheel notch; touchpads scroll in pixels, this many to a notch.
const ZOOM_PER_NOTCH: f32 = 1.25;
const PIXELS_PER_NOTCH: f32 = 100.0;

/// The compass's lines, and the cross's length against the window's height,
/// with the shortest it may be; how far it stands in from the map's corner.
const COMPASS_LINE: f32 = 5.0;
const COMPASS_OF_HEIGHT: f32 = 0.05;
const COMPASS_MIN_LENGTH: f32 = 36.0;
const COMPASS_INSET: f32 = 40.0;
/// A map dragged away has to keep something known this far inside its edges.
const KEEP_KNOWN_INSIDE: f64 = 40.0;

/// How long working out colours for the world map may take per redraw.
const MINIMAP_BUDGET_MS: f64 = 3.0;
const FULL_MAP_BUDGET_MS: f64 = 10.0;

/// A map is redrawn at least this often, since what is remembered can change
/// without marking anything changed (`dungeon_map` adds to it).
const REDRAW_SECONDS: f32 = 0.25;

/// Which full-screen layer the full map is.
const FULL_MAP_LAYER: i32 = 0;

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FullMap>()
            .init_resource::<WorldTiles>()
            .init_resource::<Sprites>()
            .add_systems(Startup, (spawn_maps, floors::spawn_column))
            .add_systems(
                PreUpdate,
                (go_into_clicked_poi, take_input, floors::back_to_own_floor)
                    .chain()
                    .after(InputSystems)
                    .after(PointerSystems)
                    .after(WidgetSystems::Modal),
            )
            .add_systems(
                Update,
                draw_maps
                    .after(PlayerSystems::Travel)
                    .after(View3dSystems::Place)
                    .run_if(not(in_state(GameView::Connecting))),
            )
            .add_systems(
                Update,
                (
                    name_what_is_hovered,
                    name_hovered_token,
                    floors::show_buttons,
                    show_darkening,
                )
                    .run_if(not(in_state(GameView::Connecting))),
            );
    }
}

/// The full map: whether it is open, where it has been dragged to, and how
/// far it is zoomed in.
#[derive(Resource)]
pub struct FullMap {
    pub open: bool,
    /// The grid position in its middle; `None` follows the player.
    pub centre: Option<(f64, f64)>,
    /// Its scale against the minimap's.
    pub zoom: f32,
    /// Where the cursor was last frame, while dragging.
    drag_from: Option<Vec2>,
    /// The floor just changed to, keeping the map's offset from the player
    /// even if nothing known shows there.
    kept_on: Option<u32>,
}

impl Default for FullMap {
    fn default() -> Self {
        Self {
            open: false,
            centre: None,
            zoom: OPENING_ZOOM,
            drag_from: None,
            kept_on: None,
        }
    }
}

impl FullMap {
    /// Opens (or closes) it as it first looks: on the player, at the opening
    /// zoom.
    fn set_open(&mut self, open: bool) {
        *self = Self { open, ..default() };
    }
}

#[derive(Component)]
struct Minimap;

#[derive(Component)]
struct FullMapPanel;

/// Darkens everything outside the full map, and stops widgets under it being
/// used; presses there are left for `take_input`, which closes the map.
#[derive(Component)]
struct FullMapDarkening;

#[derive(Component)]
struct FullMapPicture;

#[derive(Component)]
struct FullMapTitle;

/// Holds the box naming what is under the mouse, centred along the parchment.
#[derive(Component)]
struct HoverBox;

#[derive(Component)]
struct HoverText;

/// The box naming the poi the mouse is over on the world map itself.
#[derive(Component)]
struct WorldHoverBox;

#[derive(Component)]
struct WorldHoverText;

/// The pictures the maps are drawn in.
#[derive(Resource)]
struct MapPictures {
    minimap: Handle<Image>,
    full: Handle<Image>,
}

fn new_picture() -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

fn spawn_maps(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let minimap = images.add(new_picture());
    let full = images.add(new_picture());
    commands.spawn((
        Minimap,
        ImageNode::new(minimap.clone()),
        Node {
            position_type: PositionType::Absolute,
            top: px(CORNER_GAP),
            right: px(CORNER_GAP),
            ..default()
        },
        GlobalZIndex(10),
        Visibility::Hidden,
    ));
    let behind = commands
        .spawn((
            darkening(FULL_MAP_LAYER),
            FullMapDarkening,
            Visibility::Hidden,
        ))
        .id();
    commands.entity(behind).insert(leaves_presses(behind));
    commands
        .spawn((
            panel(),
            FullMapPanel,
            GlobalZIndex(layer_z(FULL_MAP_LAYER) + 1),
            Visibility::Hidden,
        ))
        .with_children(|panel| {
            panel.spawn(panel_frame());
            panel.spawn((
                FullMapTitle,
                panel_title(""),
                TextLayout::justify(Justify::Center),
                Node {
                    position_type: PositionType::Absolute,
                    top: px(FULL_MAP_MARGIN),
                    left: px(0),
                    right: px(0),
                    ..default()
                },
            ));
            panel.spawn((
                FullMapPicture,
                ImageNode::new(full.clone()),
                Node {
                    position_type: PositionType::Absolute,
                    top: px(FULL_MAP_MARGIN + TITLE_HEIGHT),
                    left: px(FULL_MAP_MARGIN),
                    ..default()
                },
            ));
            spawn_close_button(
                panel,
                Node {
                    position_type: PositionType::Absolute,
                    top: px(FULL_MAP_MARGIN),
                    right: px(FULL_MAP_MARGIN),
                    ..default()
                },
            );
            panel
                .spawn((
                    HoverBox,
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: px(FULL_MAP_MARGIN + HOVER_LIFT),
                        left: px(0),
                        right: px(0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    Visibility::Hidden,
                ))
                .with_children(|holder| spawn_tooltip(holder, HoverText, ""));
        });
    commands
        .spawn((
            WorldHoverBox,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(FULL_MAP_MARGIN + HOVER_LIFT),
                left: px(0),
                right: px(0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(10),
            Visibility::Hidden,
        ))
        .with_children(|holder| spawn_tooltip(holder, WorldHoverText, ""));
    commands.insert_resource(MapPictures { minimap, full });
}

/// The window as the maps' nodes see it: its size in the interface's pixels,
/// which `UiScale` makes bigger or smaller, so the maps keep their share of the
/// window whatever size the interface is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Room {
    size: Vec2,
    /// The screen's pixels to an interface pixel.
    scale: f32,
    /// The screen's pixels to a logical window pixel.
    screen: f32,
}

impl Room {
    pub(super) fn of(window: &Window, ui_scale: &UiScale) -> Self {
        Self {
            size: interface_area(window, ui_scale),
            scale: window.scale_factor() * ui_scale.0,
            screen: window.scale_factor(),
        }
    }

    pub(super) fn width(&self) -> f32 {
        self.size.x
    }

    pub(super) fn height(&self) -> f32 {
        self.size.y
    }

    /// The screen's pixels to an interface pixel.
    pub(super) fn scale_factor(&self) -> f32 {
        self.scale
    }

    /// The screen's pixels to a logical window pixel: for what is drawn in the
    /// maps' pictures, which follows the screen as the maps do.
    pub(super) fn screen_factor(&self) -> f32 {
        self.screen
    }
}

/// The minimap's round part across, in the interface's pixels.
fn minimap_across(window: &Room) -> f32 {
    // Never nothing: a minimised window has no height, and the scale of the
    // maps is worked out from this.
    (window.height() * MINIMAP_OF_HEIGHT).round().max(1.0)
}

/// How much bigger than given the sizes round the minimap are in this window.
fn minimap_trim(window: &Room) -> f32 {
    minimap_across(window) / (MINIMAP_BASE_HEIGHT * MINIMAP_OF_HEIGHT)
}

/// The picture holding it, with the suns and moons round it (more than the
/// notch needs).
fn minimap_picture_size(window: &Room) -> f32 {
    minimap_across(window) + 2.0 * (sky::BAND.max(NOTCH) * minimap_trim(window) + 1.0)
}

/// Whether the sky can be seen here, so the minimap shows what is in it.
fn under_the_sky(view: GameView, dungeon: &CurrentDungeon) -> bool {
    view != GameView::Dungeon || dungeon.knows_outside()
}

/// The full map's parchment, across, in logical pixels.
fn full_map_side(window: &Room) -> f32 {
    (window.width().min(window.height()) * FULL_MAP_OF_WINDOW).round()
}

/// The part of it the map is drawn on, in logical pixels.
fn full_map_area(window: &Room) -> Vec2 {
    let side = full_map_side(window);
    Vec2::new(
        side - 2.0 * FULL_MAP_MARGIN,
        side - 2.0 * FULL_MAP_MARGIN - TITLE_HEIGHT,
    )
    .max(Vec2::ONE)
}

/// The top-left corner of the full map's parchment, in logical pixels.
fn full_map_corner(window: &Room) -> Vec2 {
    let side = full_map_side(window);
    Vec2::new(window.width() - side, window.height() - side) / 2.0
}

/// The top-left corner of the full map's picture, in logical pixels.
fn full_map_picture_corner(window: &Room) -> Vec2 {
    full_map_corner(window) + Vec2::new(FULL_MAP_MARGIN, FULL_MAP_MARGIN + TITLE_HEIGHT)
}

/// Where the mouse is over the full map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverFullMap {
    Outside,
    CloseButton,
    Parchment,
}

fn over_full_map(window: &Room, at: Vec2) -> OverFullMap {
    let side = full_map_side(window);
    let at = at - full_map_corner(window);
    let inside = |from: f32, to: f32, at: f32| (from..to).contains(&at);
    if !inside(0.0, side, at.x) || !inside(0.0, side, at.y) {
        OverFullMap::Outside
    } else if inside(
        side - FULL_MAP_MARGIN - CLOSE_PIXELS - CLOSE_REACH,
        side,
        at.x,
    ) && inside(0.0, FULL_MAP_MARGIN + CLOSE_PIXELS + CLOSE_REACH, at.y)
    {
        OverFullMap::CloseButton
    } else {
        OverFullMap::Parchment
    }
}

/// Grid units per logical pixel at the minimap's scale. On the world map the minimap shows as much as
/// the window on screen does across, which zooming changes.
fn units_per_pixel(view: GameView, window: &Room, camera: &OrbitCamera) -> f64 {
    match view {
        GameView::WorldMap => {
            let shown = 2.0 * (world_map::HALF_SIZE as f64) * f64::from(camera.zoom());
            shown / f64::from(minimap_across(window))
        }
        _ => 1.0 / f64::from(CELL_PIXELS),
    }
}

/// Whether a full map centred somewhere still shows something known.
fn shows_known(
    view: GameView,
    world: Option<&CurrentWorld>,
    dungeon: &CurrentDungeon,
    centre: (f64, f64),
    half: (f64, f64),
) -> bool {
    let min = (centre.0 - half.0, centre.1 - half.1);
    let max = (centre.0 + half.0, centre.1 + half.1);
    match view {
        GameView::WorldMap => world.is_some_and(|world| world::shows_known(world, (min, max))),
        _ => {
            let cell = |at: f64| (at + 0.5).floor() as i64;
            (cell(min.1)..=cell(max.1))
                .any(|y| (cell(min.0)..=cell(max.0)).any(|x| dungeon.remembered.is_seen(x, y)))
        }
    }
}

/// Developer tools: in Teleport Mode, a click on a poi on the full map teleports
/// there, enters it and closes the map. Elsewhere on the map, it is an ordinary
/// click.
#[allow(clippy::too_many_arguments)]
fn go_into_clicked_poi(
    mode: Res<ClickMode>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    view: Res<State<GameView>>,
    camera: Single<&OrbitCamera>,
    player: Single<&SmoothPosition, With<Player>>,
    world: Option<Res<CurrentWorld>>,
    mut full: ResMut<FullMap>,
    mut to_server: MessageWriter<ToServer>,
    pointer: Res<UiPointer>,
) {
    let window = Room::of(&window, &ui_scale);
    let view = *view.get();
    let teleporting = *mode == ClickMode::Teleport;
    if !full.open
        || view != GameView::WorldMap
        || !teleporting
        || !mouse.just_pressed(MouseButton::Left)
    {
        return;
    }
    let centre = (player.x, player.y);
    let poi = pointer
        .0
        .and_then(|cursor| spot_on_full_map(&window, cursor, view, &camera, &full, centre))
        .and_then(|(spot, units_per_pixel)| {
            let reach = ICON_PIXELS * full.zoom / 2.0;
            hovered_poi(world.as_deref(), spot, reach, units_per_pixel)
        });
    let Some(poi) = poi else {
        return;
    };
    mouse.clear_just_pressed(MouseButton::Left);
    let (x, y) = (poi.x, poi.y);
    to_server.write(ToServer(ClientMessage::DevTeleport { x, y }));
    to_server.write(ToServer(ClientMessage::EnterSite { chunk: poi.chunk }));
    full.set_open(false);
}

/// The darkening shows while the full map does.
fn show_darkening(full: Res<FullMap>, mut behind: Single<&mut Visibility, With<FullMapDarkening>>) {
    behind.set_if_neq(if full.open {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
}

/// The maps get the input first, in `PreUpdate`: what they use is taken away
/// before the game's systems look. M or clicking the minimap opens the full
/// map; M, Esc, its close button or clicking outside it closes it. While it is
/// open the mouse and fingers are the map's: dragging moves it, and the wheel
/// or a pinch zooms it.
#[allow(clippy::too_many_arguments)]
fn take_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    view: Res<State<GameView>>,
    camera: Single<&OrbitCamera>,
    world: Option<Res<CurrentWorld>>,
    dungeon: Res<CurrentDungeon>,
    mut levels: ResMut<OtherLevels>,
    player: Single<&SmoothPosition, With<Player>>,
    mut full: ResMut<FullMap>,
    pointer: Res<UiPointer>,
    mut gesture: ResMut<TouchGesture>,
) {
    let window = Room::of(&window, &ui_scale);
    let view = *view.get();
    if view == GameView::Connecting {
        return;
    }
    if keys.just_pressed(KeyCode::KeyM) {
        let open = !full.open;
        full.set_open(open);
    }
    let cursor = pointer.0;
    // A floor's button: that floor, round the player's place.
    let buttons = floors::buttons(view, full.open, &dungeon, &levels);
    let button = cursor.and_then(|at| floors::under(&window, at, buttons.len()));
    if let Some(row) = button
        && mouse.clear_just_pressed(MouseButton::Left)
    {
        // The map stays as far from the player's place as it was dragged, so
        // floors can be compared by going up and down.
        let from = levels.spot(&dungeon, (player.x, player.y));
        levels.show(buttons[row]);
        let to = levels.spot(&dungeon, (player.x, player.y));
        full.centre = full
            .centre
            .map(|at| (at.0 - from.0 + to.0, at.1 - from.1 + to.1));
        full.kept_on = Some(buttons[row]);
        full.drag_from = None;
        return;
    }
    let spot = levels.spot(&dungeon, (player.x, player.y));
    let shown = levels.shown(&dungeon);
    if !full.open {
        let middle = Vec2::new(
            window.width() - CORNER_GAP - minimap_picture_size(&window) / 2.0,
            CORNER_GAP + minimap_picture_size(&window) / 2.0,
        );
        let over_minimap =
            cursor.is_some_and(|at| at.distance(middle) <= minimap_across(&window) / 2.0);
        if over_minimap && mouse.clear_just_pressed(MouseButton::Left) {
            full.set_open(true);
        }
        return;
    }
    if keys.clear_just_pressed(KeyCode::Escape) {
        full.set_open(false);
        return;
    }
    keys.clear_just_pressed(KeyCode::Home);

    // A press anywhere is the map's.
    if mouse.clear_just_pressed(MouseButton::Left) {
        match cursor.map(|at| over_full_map(&window, at)) {
            Some(OverFullMap::Outside | OverFullMap::CloseButton) => {
                full.set_open(false);
                return;
            }
            Some(OverFullMap::Parchment) => full.drag_from = cursor,
            None => {}
        }
    }
    if !mouse.pressed(MouseButton::Left) {
        full.drag_from = None;
    }
    mouse.reset(MouseButton::Right);
    let notches = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / PIXELS_PER_NOTCH,
    };
    scroll.delta = Vec2::ZERO;
    // The fingers are the map's too: the camera gets none of this.
    let touch = std::mem::take(&mut *gesture);
    let unzoomed = units_per_pixel(view, &window, &camera);
    if notches != 0.0 || touch.pinch != 1.0 {
        let before = unzoomed / f64::from(full.zoom);
        let factor = ZOOM_PER_NOTCH.powf(notches) * touch.pinch;
        full.zoom = (full.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let after = unzoomed / f64::from(full.zoom);
        // Towards the mouse, or between the fingers: what is there stays there.
        let area = full_map_area(&window);
        let middle = full_map_picture_corner(&window) + area / 2.0;
        let towards = if touch.pinch != 1.0 {
            Some(in_interface(touch.centre, &ui_scale))
        } else {
            cursor
        };
        let over_map = towards.map(|at| at - middle).filter(|from_middle| {
            from_middle.x.abs() <= area.x / 2.0 && from_middle.y.abs() <= area.y / 2.0
        });
        if let Some(from_middle) = over_map
            && before != after
        {
            let centre = full.centre.unwrap_or(spot);
            full.centre = Some((
                centre.0 + f64::from(from_middle.x) * (before - after),
                centre.1 + f64::from(from_middle.y) * (before - after),
            ));
        }
    }

    let scale = unzoomed / f64::from(full.zoom);
    let area = full_map_area(&window).as_dvec2();
    let half = (
        (area.x / 2.0 - KEEP_KNOWN_INSIDE).max(1.0) * scale,
        (area.y / 2.0 - KEEP_KNOWN_INSIDE).max(1.0) * scale,
    );
    let known = |at| shows_known(view, world.as_deref(), shown, at, half);
    // Zoomed in, or somewhere else now (down the stairs, say): back towards
    // the player until something known shows. Another floor may show nothing
    // round the player's place: then the map is free until it finds something.
    let shown_floor = shown.info.as_ref().map(|info| info.index);
    if full.centre.is_some_and(known) {
        full.kept_on = None;
    }
    let kept = full.kept_on.is_some() && full.kept_on == shown_floor;
    if let Some(at) = full.centre.filter(|&at| !kept && !known(at) && known(spot)) {
        full.centre = Some(projection::slide(spot, at, known));
    }
    let moved = match (full.drag_from, cursor) {
        (Some(from), Some(to)) => {
            full.drag_from = cursor;
            Some(to - from)
        }
        _ => (touch.drag != Vec2::ZERO).then_some(in_interface(touch.drag, &ui_scale)),
    };
    if let Some(moved) = moved {
        let centre = full.centre.unwrap_or(spot);
        // The land moves with the cursor, so the centre goes the other way.
        let wanted = (
            centre.0 - f64::from(moved.x) * scale,
            centre.1 - f64::from(moved.y) * scale,
        );
        full.centre = Some(if known(centre) {
            projection::slide(centre, wanted, known)
        } else {
            wanted
        });
    }
}

/// What a picture was last drawn from: if nothing here changes, it needn't be.
#[derive(Clone, PartialEq)]
struct DrawnFrom {
    view: GameView,
    floor: Option<u32>,
    open: bool,
    centre: (f64, f64),
    player: (f64, f64),
    turn: (f32, f32),
    scale: f64,
    size: (u32, u32),
    water_and_snow: (u64, u64),
}

/// What maps are drawn from.
#[derive(SystemParam)]
struct Sources<'w, 's> {
    view: Res<'w, State<GameView>>,
    window: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
    ui_scale: Res<'w, UiScale>,
    player: Single<'w, 's, &'static SmoothPosition, With<Player>>,
    camera: Single<'w, 's, &'static OrbitCamera>,
    world: Option<Res<'w, CurrentWorld>>,
    dungeon: Res<'w, CurrentDungeon>,
    levels: Res<'w, OtherLevels>,
    first_floor: Res<'w, FirstFloor>,
    style: Res<'w, SurfaceStyle>,
    clock: Res<'w, GameClock>,
    weather: Res<'w, Weather>,
    sky: Res<'w, SkyNow>,
    pois: Query<'w, 's, (&'static Portal, &'static Token)>,
    asset_server: Res<'w, AssetServer>,
    full: Res<'w, FullMap>,
}

/// A map's node, to size and show or hide.
type Shown = (&'static mut Node, &'static mut Visibility);
type OnlyMinimap = (With<Minimap>, Without<FullMapPanel>);
type OnlyPanel = (With<FullMapPanel>, Without<Minimap>);
type OnlyPicture = (
    With<FullMapPicture>,
    Without<Minimap>,
    Without<FullMapPanel>,
);

/// The maps' parts on screen.
#[derive(SystemParam)]
struct Screen<'w, 's> {
    minimap: Single<'w, 's, Shown, OnlyMinimap>,
    panel: Single<'w, 's, Shown, OnlyPanel>,
    picture: Single<'w, 's, &'static mut Node, OnlyPicture>,
    title: Single<'w, 's, &'static mut Text, With<FullMapTitle>>,
}

#[allow(clippy::too_many_arguments)]
fn draw_maps(
    sources: Sources,
    mut screen: Screen,
    pictures: Res<MapPictures>,
    time: Res<Time>,
    mut images: ResMut<Assets<Image>>,
    mut tiles: ResMut<WorldTiles>,
    mut sprites: ResMut<Sprites>,
    mut last: Local<Option<DrawnFrom>>,
    mut pending: Local<bool>,
    mut since: Local<f32>,
) {
    let Sources {
        view,
        window,
        ui_scale,
        player,
        camera,
        world,
        dungeon,
        levels,
        first_floor,
        style,
        clock,
        weather,
        sky: sky_now,
        pois,
        asset_server,
        full,
    } = sources;
    let Screen {
        minimap,
        panel,
        picture,
        title,
    } = &mut screen;
    let view = *view.get();
    let window = Room::of(&window, &ui_scale);
    // The pictures' pixels to an interface pixel, for their sizes; what is drawn in
    // them follows the screen (`dots`), as the maps do, not the interface.
    let scale = window.scale_factor();
    let dots = window.screen_factor();
    let on_other_floor = levels.showing().is_some();
    let spot = levels.spot(&dungeon, (player.x, player.y));
    // Which way the player's floor is from the one shown.
    let player_is_up = dungeon
        .info
        .as_ref()
        .zip(levels.showing())
        .is_some_and(|(info, shown)| {
            (info.storey(info.index) > info.storey(shown)) == info.building
        });
    let players_level = &*dungeon;
    let dungeon = levels.shown(players_level);
    *minimap.1 = if full.open {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    *panel.1 = if full.open {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    // The turn: the full map always has north at the top, like a sheet of
    // paper, while the minimap turns with the camera.
    let (right, down) = camera.ground_axes();
    let (grid_right, grid_down) = if full.open {
        Projection::NORTH_UP
    } else {
        (
            (f64::from(right.x), f64::from(right.y)),
            (f64::from(down.x), f64::from(down.y)),
        )
    };

    let across = if full.open {
        let side = full_map_side(&window);
        panel.0.width = px(side);
        panel.0.height = px(side);
        panel.0.left = px((window.width() - side) / 2.0);
        panel.0.top = px((window.height() - side) / 2.0);
        let area = full_map_area(&window);
        picture.width = px(area.x);
        picture.height = px(area.y);
        let name = location_title(view, world.as_deref(), dungeon, *first_floor, &player);
        if title.0 != name {
            title.0 = name;
        }
        area
    } else {
        let size = minimap_picture_size(&window);
        minimap.0.width = px(size);
        minimap.0.height = px(size);
        Vec2::splat(size)
    };
    let size = (
        ((across.x * scale).round() as u32).max(1),
        ((across.y * scale).round() as u32).max(1),
    );

    let centre = full.centre.filter(|_| full.open).unwrap_or(spot);
    let zoom = if full.open { full.zoom } else { 1.0 };
    let units = units_per_pixel(view, &window, &camera) / f64::from(scale * zoom);
    let water_and_snow = world.as_deref().map_or((0, 0), |world| {
        let conditions = world.conditions(&clock, &weather);
        (conditions.water.to_bits(), conditions.snow_line.to_bits())
    });
    let now = DrawnFrom {
        view,
        floor: levels.showing(),
        open: full.open,
        centre,
        player: (player.x, player.y),
        turn: (grid_right.0 as f32, grid_right.1 as f32),
        scale: units,
        size,
        water_and_snow,
    };
    *since += time.delta_secs();
    if last.as_ref() == Some(&now) && !*pending && *since < REDRAW_SECONDS {
        return;
    }
    *since = 0.0;
    *last = Some(now);

    let projection = Projection {
        centre,
        middle: (size.0 as f32 / 2.0, size.1 as f32 / 2.0),
        units_per_pixel: units,
        right: grid_right,
        down: grid_down,
    };
    // Round the circle, or to the corners of the full map.
    let clip = if full.open {
        f32::MAX
    } else {
        minimap_across(&window) * scale / 2.0
    };
    let reach = if full.open {
        (size.0 as f32).hypot(size.1 as f32) / 2.0
    } else {
        clip
    } + ICON_PIXELS * zoom * dots;
    let grid_box = projection.grid_box(reach);

    // What can be got ready before the picture is taken out of the assets.
    let mut stamps = Vec::new();
    let mut busy = false;
    let looks = (view != GameView::WorldMap).then(|| {
        let cell = |at: f64| (at + 0.5).floor() as i64;
        Looks::of(
            dungeon,
            (cell(grid_box.0.0), cell(grid_box.0.1)),
            (cell(grid_box.1.0), cell(grid_box.1.1)),
        )
    });
    let mut prepare = |path: String, part, pixels: f32, at, opacity| {
        let pixels = (pixels * dots).round().max(1.0) as u32;
        if sprites.prepare(&asset_server, &images, &path, part, pixels) {
            stamps.push((path, part, pixels, at, opacity));
            false
        } else {
            true
        }
    };
    match (&looks, world.as_deref()) {
        (Some(looks), _) => {
            for ((x, y), kind) in looks.signs() {
                let at = projection.to_picture(x as f64, y as f64);
                let part = Some(door_picture(kind));
                busy |= prepare(TOWN_ATLAS.to_string(), part, SIGN_PIXELS * zoom, at, 1.0);
            }
        }
        (None, Some(world)) => {
            let width = world.surface.width();
            let completed: BTreeSet<_> = pois
                .iter()
                .filter(|(_, token)| token.completed)
                .map(|(Portal::Poi(poi), _)| (poi.x.rem_euclid(width), poi.y))
                .collect();
            let ((min_x, min_y), (max_x, max_y)) = grid_box;
            let area = (
                (min_x.floor() as i64, min_y.floor() as i64),
                (max_x.ceil() as i64, max_y.ceil() as i64),
            );
            for poi in world.pois.in_area(area.0, area.1) {
                let at = projection.to_picture(poi.x as f64, poi.y as f64);
                let opacity = if completed.contains(&(poi.x.rem_euclid(width), poi.y)) {
                    COMPLETED_OPACITY
                } else {
                    1.0
                };
                let pixels = ((ICON_PIXELS * zoom) * dots).round().max(1.0) as u32;
                let name = poi_picture_name(poi.kind, poi.look);
                sprites.make(&name, pixels, |size| poi_picture(poi.kind, poi.look, size));
                stamps.push((name, None, pixels, at, opacity));
            }
        }
        (None, None) => {}
    }
    if let (None, Some(world)) = (&looks, world.as_deref()) {
        let budget = if full.open {
            FULL_MAP_BUDGET_MS
        } else {
            MINIMAP_BUDGET_MS
        };
        let conditions = world.conditions(&clock, &weather);
        busy |= tiles.fill(world, &style, conditions, grid_box, budget);
    }
    *pending = busy;

    let Some(mut image) = images.get_mut(if full.open {
        &pictures.full
    } else {
        &pictures.minimap
    }) else {
        return;
    };
    if image.width() != size.0 || image.height() != size.1 {
        image.resize(Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        });
    }
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let mut canvas = Canvas {
        pixels: data,
        width: size.0,
        height: size.1,
    };
    let [red, green, blue] = PARCHMENT;
    canvas.fill([red, green, blue, 255]);
    match (&looks, world.as_deref()) {
        (Some(looks), _) => grid::draw(&mut canvas, &projection, looks, PARCHMENT, clip),
        (None, Some(world)) => {
            let width = world.surface.width();
            tiles.draw(&mut canvas, &projection, width, clip, PARCHMENT);
        }
        (None, None) => {}
    }
    for (path, part, pixels, at, opacity) in &stamps {
        if let Some(sprite) = sprites.get(path, *part, *pixels) {
            canvas.sprite(sprite, *at, *opacity);
        }
    }

    if looks.is_some() {
        for stairs in &dungeon.stairs {
            if dungeon.remembered.is_seen(stairs.x, stairs.y) {
                frame::stairs_icon(
                    &mut canvas,
                    projection.to_picture(stairs.x as f64, stairs.y as f64),
                    STAIRS_PIXELS * zoom * dots,
                    stairs.direction == StairsDirection::Down,
                );
            }
        }
    }

    let player_at = projection.to_picture(spot.0, spot.1);
    // The dot grows with the map.
    let (dot, outline) = (DOT_PIXELS * zoom * dots, DOT_OUTLINE * zoom * dots);
    if full.open {
        let picture_size = (size.0 as f32, size.1 as f32);
        // On another floor they are smaller, with what says which way beside
        // them. The arrow is as wide as the dot, outline included, so what is
        // beside it stays put as it moves round the edge.
        let (dot, hint) = if on_other_floor {
            (
                dot * OTHER_FLOOR_SIZE,
                Some(FLOOR_HINT_PIXELS * zoom * dots),
            )
        } else {
            (dot, None)
        };
        let arrow_place = frame::arrow_place(picture_size, player_at, ARROW_INSET * zoom * dots);
        let mark = arrow_place.map_or(player_at, |(place, _)| place);
        match arrow_place {
            Some((place, direction)) => {
                let arrow = dot - 2.0 * outline;
                frame::arrow(&mut canvas, place, direction, arrow, outline)
            }
            None => frame::player_dot(&mut canvas, player_at, dot, outline),
        }
        if let Some(hint) = hint {
            frame::floor_hint(&mut canvas, mark, dot / 2.0, hint, player_is_up);
        }
        let length = (window.height() * COMPASS_OF_HEIGHT).max(COMPASS_MIN_LENGTH);
        let foot = (
            size.0 as f32 - COMPASS_INSET * dots,
            size.1 as f32 - HOVER_LIFT * scale,
        );
        frame::compass(&mut canvas, foot, length * scale, COMPASS_LINE * dots);
    } else {
        frame::player_dot(&mut canvas, player_at, dot, outline);
        let trim = scale * minimap_trim(&window);
        // North on the picture: the grid's north is `-y`.
        let round = RoundFrame {
            centre: projection.middle,
            radius: clip,
            border: BORDER * trim,
            notch: NOTCH * trim,
            north: (-grid_right.1 as f32, -grid_down.1 as f32),
        };
        round.draw(&mut canvas);
        // Last, as they are outside what that cuts off.
        if under_the_sky(view, players_level) {
            let middle = Vec2::from(projection.middle);
            let icons = sky::icons(&sky_now.bodies, middle, clip, trim);
            sky::draw(&mut canvas, &icons, &sky_now.bodies, &round, trim);
        }
    }
}

/// Names what the mouse is over on the full map, in a box near the bottom of
/// the parchment.
#[allow(clippy::too_many_arguments)]
fn name_what_is_hovered(
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    view: Res<State<GameView>>,
    camera: Single<&OrbitCamera>,
    player: Single<&SmoothPosition, With<Player>>,
    world: Option<Res<CurrentWorld>>,
    dungeon: Res<CurrentDungeon>,
    levels: Res<OtherLevels>,
    full: Res<FullMap>,
    mut holder: Single<&mut Visibility, With<HoverBox>>,
    mut text: Single<&mut Text, With<HoverText>>,
    pointer: Res<UiPointer>,
) {
    let window = Room::of(&window, &ui_scale);
    let view = *view.get();
    let name = pointer.0.filter(|_| full.open).and_then(|cursor| {
        let centre = levels.spot(&dungeon, (player.x, player.y));
        let (spot, units_per_pixel) =
            spot_on_full_map(&window, cursor, view, &camera, &full, centre)?;
        hovered_name(
            view,
            world.as_deref(),
            levels.shown(&dungeon),
            spot,
            ICON_PIXELS * full.zoom / 2.0,
            units_per_pixel,
        )
    });
    let shown = if name.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    holder.set_if_neq(shown);
    if let Some(name) = name
        && text.0 != name
    {
        text.0 = name;
    }
}

/// The grid position under the cursor on the open full map, if it is over the
/// picture, and the grid units per pixel there. `player_spot` is the map's middle
/// unless it has been dragged.
fn spot_on_full_map(
    window: &Room,
    cursor: Vec2,
    view: GameView,
    camera: &OrbitCamera,
    full: &FullMap,
    player_spot: (f64, f64),
) -> Option<((f64, f64), f64)> {
    let corner =
        full_map_corner(window) + Vec2::new(FULL_MAP_MARGIN, FULL_MAP_MARGIN + TITLE_HEIGHT);
    let area = full_map_area(window);
    let at = cursor - corner;
    if at.x < 0.0 || at.y < 0.0 || at.x >= area.x || at.y >= area.y {
        return None;
    }
    let (right, down) = Projection::NORTH_UP;
    let map = Projection {
        centre: full.centre.unwrap_or(player_spot),
        middle: (area.x / 2.0, area.y / 2.0),
        units_per_pixel: units_per_pixel(view, window, camera) / f64::from(full.zoom),
        right,
        down,
    };
    Some((map.to_grid(at.x, at.y), map.units_per_pixel))
}

/// Names the sun or moon by the minimap that the pointer is on, if any.
fn hovered_body(
    view: GameView,
    dungeon: &CurrentDungeon,
    sky_now: &SkyNow,
    window: &Room,
    at: Vec2,
) -> Option<String> {
    if !under_the_sky(view, dungeon) {
        return None;
    }
    let middle = Vec2::new(
        window.width() - CORNER_GAP - minimap_picture_size(window) / 2.0,
        CORNER_GAP + minimap_picture_size(window) / 2.0,
    );
    let circle = minimap_across(window) / 2.0;
    let trim = minimap_trim(window);
    let icons = sky::icons(&sky_now.bodies, middle, circle, trim);
    sky::under(&icons, middle, circle, at).map(|number| sky::name(&sky_now.bodies[number]))
}

/// Names what the pointer is over in a box like the full map's near the bottom
/// of the window: a sun or moon by the minimap, or a poi on the world map. The
/// full map covers it when open.
#[allow(clippy::too_many_arguments)]
fn name_hovered_token(
    view: Res<State<GameView>>,
    full: Res<FullMap>,
    hovered: Res<HoveredToken>,
    portals: Query<&Portal>,
    (sky_now, dungeon, pointer): (Res<SkyNow>, Res<CurrentDungeon>, Res<UiPointer>),
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut holder: Single<&mut Visibility, With<WorldHoverBox>>,
    mut text: Single<&mut Text, With<WorldHoverText>>,
) {
    let window = Room::of(&window, &ui_scale);
    let body = pointer
        .0
        .filter(|_| !full.open)
        .and_then(|at| hovered_body(*view.get(), &dungeon, &sky_now, &window, at));
    let name = body.or_else(|| {
        hovered
            .0
            .filter(|_| !full.open && *view.get() == GameView::WorldMap)
            .and_then(|token| portals.get(token).ok())
            .map(|Portal::Poi(poi)| poi_name(poi))
    });
    let shown = if name.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    holder.set_if_neq(shown);
    if let Some(name) = name
        && text.0 != name
    {
        text.0 = name;
    }
}

/// What a poi is called.
fn poi_name(poi: &SeenPoi) -> String {
    let ruined = if poi.look.is_ruined() { "Ruined" } else { "" };
    words(&format!("{ruined}{:?}", poi.kind))
}

/// The name of a shop door or world map poi at a grid position.
/// `icon_reach` is how far a poi's picture reaches, in logical pixels.
fn hovered_name(
    view: GameView,
    world: Option<&CurrentWorld>,
    dungeon: &CurrentDungeon,
    (x, y): (f64, f64),
    icon_reach: f32,
    units_per_pixel: f64,
) -> Option<String> {
    if view != GameView::WorldMap {
        let (cell_x, cell_y) = ((x + 0.5).floor() as i64, (y + 0.5).floor() as i64);
        return match grid::look_of(dungeon, cell_x, cell_y) {
            CellLook::Sign(kind) => Some(words(&format!("{kind:?}"))),
            _ => None,
        };
    }
    hovered_poi(world, (x, y), icon_reach, units_per_pixel).map(|poi| poi_name(&poi))
}

/// The world map poi whose picture is at a grid position, the nearest if several.
fn hovered_poi(
    world: Option<&CurrentWorld>,
    (x, y): (f64, f64),
    icon_reach: f32,
    units_per_pixel: f64,
) -> Option<SeenPoi> {
    let reach = f64::from(icon_reach) * units_per_pixel;
    let min = ((x - reach).floor() as i64, (y - reach).floor() as i64);
    let max = ((x + reach).ceil() as i64, (y + reach).ceil() as i64);
    let distance = |poi: &SeenPoi| (poi.x as f64 - x).hypot(poi.y as f64 - y);
    world?
        .pois
        .in_area(min, max)
        .into_iter()
        .filter(|poi| (poi.x as f64 - x).abs().max((poi.y as f64 - y).abs()) <= reach)
        .min_by(|a, b| distance(a).total_cmp(&distance(b)))
}

/// The full map's title: the place, with the floor or level being looked at,
/// numbered as its button is.
fn location_title(
    view: GameView,
    world: Option<&CurrentWorld>,
    dungeon: &CurrentDungeon,
    first_floor: FirstFloor,
    player: &SmoothPosition,
) -> String {
    match (view, world) {
        (GameView::WorldMap, Some(world)) => {
            let (x, y) = (player.x.round() as i64, player.y.round() as i64);
            let height = world.surface.sample(x, y).height;
            let biome = world.climate.sample(x, y, height).biome();
            format!("World map: {}", words(&format!("{biome:?}")))
        }
        _ => words(&dungeon.floor_name(first_floor)),
    }
}

/// Spaces out a name written in code style: "StoneTower" as "Stone tower".
fn words(name: &str) -> String {
    let mut out = String::new();
    for (index, letter) in name.chars().enumerate() {
        if letter.is_uppercase() && index > 0 {
            out.push(' ');
            out.extend(letter.to_lowercase());
        } else {
            out.push(letter);
        }
    }
    out
}

#[cfg(test)]
mod tests;
