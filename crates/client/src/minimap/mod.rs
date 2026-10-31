//! The maps: the round minimap always in the top-right corner, and the full
//! parchment map that M or clicking the minimap opens. Both show only what is known, drawn on the CPU into
//! a picture. See `docs/context/client/minimap.md`.

mod canvas;
mod floors;
mod frame;
mod grid;
mod projection;
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
use shared::protocol::SeenPoi;
use shared::site::StairsDirection;

use crate::common::{
    CurrentDungeon, CurrentWorld, FirstFloor, GameClock, GameView, OtherLevels, Portal,
    SmoothPosition, Token, Weather,
};
use crate::dungeon_map::{TOWN_ATLAS, door_picture};
use crate::player::{Player, PlayerSystems};
use crate::surface_style::SurfaceStyle;
use crate::ui::raised_box;
use crate::view3d::{HoveredToken, OrbitCamera, View3dSystems};
use crate::world_map::{self, poi_picture, poi_picture_name};
use canvas::Canvas;
use frame::RoundFrame;
use grid::{CellLook, Looks};
use projection::Projection;
use sprites::Sprites;
use world::WorldTiles;

/// The inside of the maps, the colour of parchment.
const PARCHMENT: [u8; 3] = [240, 225, 190];
const INK: Color = Color::srgb(0.25, 0.17, 0.1);

/// The minimap's size against the window's height, its border and notch, and
/// its gap from the window's corner. Lengths are in logical pixels.
const MINIMAP_OF_HEIGHT: f32 = 0.2;
const BORDER: f32 = 5.0;
const NOTCH: f32 = 12.0;
const CORNER_GAP: f32 = 10.0;

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
/// The close button, across.
const CLOSE_PIXELS: f32 = 28.0;
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

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FullMap>()
            .init_resource::<WorldTiles>()
            .init_resource::<Sprites>()
            .add_systems(Startup, (spawn_maps, floors::spawn_column))
            .add_systems(
                PreUpdate,
                (take_input, floors::back_to_own_floor)
                    .chain()
                    .after(InputSystems),
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
    let [red, green, blue] = PARCHMENT;
    commands
        .spawn((
            FullMapPanel,
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(red, green, blue)),
            GlobalZIndex(20),
            Visibility::Hidden,
        ))
        .with_children(|panel| {
            panel.spawn((
                FullMapTitle,
                Text::new(""),
                TextFont::from_font_size(24.0),
                TextColor(INK),
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
            panel
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(FULL_MAP_MARGIN),
                        right: px(FULL_MAP_MARGIN),
                        width: px(CLOSE_PIXELS),
                        height: px(CLOSE_PIXELS),
                        border: UiRect::all(px(2)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::from(INK),
                ))
                .with_child((
                    Text::new("X"),
                    TextFont::from_font_size(18.0),
                    TextColor(INK),
                ));
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
                .with_children(|holder| {
                    holder.spawn(raised_box()).with_child((
                        HoverText,
                        Text::new(""),
                        TextFont::from_font_size(18.0),
                        TextColor(INK),
                    ));
                });
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
        .with_children(|holder| {
            holder.spawn(raised_box()).with_child((
                WorldHoverText,
                Text::new(""),
                TextFont::from_font_size(18.0),
                TextColor(INK),
            ));
        });
    commands.insert_resource(MapPictures { minimap, full });
}

/// The minimap's round part across, in logical pixels.
fn minimap_across(window: &Window) -> f32 {
    // Never nothing: a minimised window has no height, and the scale of the
    // maps is worked out from this.
    (window.height() * MINIMAP_OF_HEIGHT).round().max(1.0)
}

/// The picture holding it, notch included.
fn minimap_picture_size(window: &Window) -> f32 {
    minimap_across(window) + 2.0 * (NOTCH + 1.0)
}

/// The full map's parchment, across, in logical pixels.
fn full_map_side(window: &Window) -> f32 {
    (window.width().min(window.height()) * FULL_MAP_OF_WINDOW).round()
}

/// The part of it the map is drawn on, in logical pixels.
fn full_map_area(window: &Window) -> Vec2 {
    let side = full_map_side(window);
    Vec2::new(
        side - 2.0 * FULL_MAP_MARGIN,
        side - 2.0 * FULL_MAP_MARGIN - TITLE_HEIGHT,
    )
    .max(Vec2::ONE)
}

/// The top-left corner of the full map's parchment, in logical pixels.
fn full_map_corner(window: &Window) -> Vec2 {
    let side = full_map_side(window);
    Vec2::new(window.width() - side, window.height() - side) / 2.0
}

/// The top-left corner of the full map's picture, in logical pixels.
fn full_map_picture_corner(window: &Window) -> Vec2 {
    full_map_corner(window) + Vec2::new(FULL_MAP_MARGIN, FULL_MAP_MARGIN + TITLE_HEIGHT)
}

/// Where the mouse is over the full map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverFullMap {
    Outside,
    CloseButton,
    Parchment,
}

fn over_full_map(window: &Window, at: Vec2) -> OverFullMap {
    let side = full_map_side(window);
    let at = at - full_map_corner(window);
    let inside = |from: f32, to: f32, at: f32| (from..to).contains(&at);
    if !inside(0.0, side, at.x) || !inside(0.0, side, at.y) {
        OverFullMap::Outside
    } else if inside(
        side - FULL_MAP_MARGIN - CLOSE_PIXELS,
        side - FULL_MAP_MARGIN,
        at.x,
    ) && inside(FULL_MAP_MARGIN, FULL_MAP_MARGIN + CLOSE_PIXELS, at.y)
    {
        OverFullMap::CloseButton
    } else {
        OverFullMap::Parchment
    }
}

/// Grid units per logical pixel at the minimap's scale. On the world map the minimap shows as much as
/// the window on screen does across, which zooming changes.
fn units_per_pixel(view: GameView, window: &Window, camera: &OrbitCamera) -> f64 {
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

/// The maps get the input first, in `PreUpdate`: what they use is taken away
/// before the game's systems look. M or clicking the minimap opens the full
/// map; M, Esc, its close button or clicking outside it closes it. While it is
/// open the mouse is the map's: dragging moves it and the wheel zooms it.
#[allow(clippy::too_many_arguments)]
fn take_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    window: Single<&Window, With<PrimaryWindow>>,
    view: Res<State<GameView>>,
    camera: Single<&OrbitCamera>,
    world: Option<Res<CurrentWorld>>,
    dungeon: Res<CurrentDungeon>,
    mut levels: ResMut<OtherLevels>,
    player: Single<&SmoothPosition, With<Player>>,
    mut full: ResMut<FullMap>,
) {
    let view = *view.get();
    if view == GameView::Connecting {
        return;
    }
    if keys.just_pressed(KeyCode::KeyM) {
        let open = !full.open;
        full.set_open(open);
    }
    let cursor = window.cursor_position();
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
    let unzoomed = units_per_pixel(view, &window, &camera);
    if notches != 0.0 {
        let before = unzoomed / f64::from(full.zoom);
        full.zoom = (full.zoom * ZOOM_PER_NOTCH.powf(notches)).clamp(MIN_ZOOM, MAX_ZOOM);
        let after = unzoomed / f64::from(full.zoom);
        // Towards the mouse: what is under it stays under it.
        let area = full_map_area(&window);
        let middle = full_map_picture_corner(&window) + area / 2.0;
        let over_map = cursor.map(|at| at - middle).filter(|from_middle| {
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
    if let (Some(from), Some(to)) = (full.drag_from, cursor) {
        full.drag_from = cursor;
        let centre = full.centre.unwrap_or(spot);
        // The land moves with the cursor, so the centre goes the other way.
        let wanted = (
            centre.0 - f64::from(to.x - from.x) * scale,
            centre.1 - f64::from(to.y - from.y) * scale,
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
    player: Single<'w, 's, &'static SmoothPosition, With<Player>>,
    camera: Single<'w, 's, &'static OrbitCamera>,
    world: Option<Res<'w, CurrentWorld>>,
    dungeon: Res<'w, CurrentDungeon>,
    levels: Res<'w, OtherLevels>,
    first_floor: Res<'w, FirstFloor>,
    style: Res<'w, SurfaceStyle>,
    clock: Res<'w, GameClock>,
    weather: Res<'w, Weather>,
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
        player,
        camera,
        world,
        dungeon,
        levels,
        first_floor,
        style,
        clock,
        weather,
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
    let scale = window.scale_factor();
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
    } + ICON_PIXELS * zoom * scale;
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
        let pixels = (pixels * scale).round().max(1.0) as u32;
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
                let pixels = ((ICON_PIXELS * zoom) * scale).round().max(1.0) as u32;
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
                    STAIRS_PIXELS * zoom * scale,
                    stairs.direction == StairsDirection::Down,
                );
            }
        }
    }

    let player_at = projection.to_picture(spot.0, spot.1);
    // The dot grows with the map.
    let (dot, outline) = (DOT_PIXELS * zoom * scale, DOT_OUTLINE * zoom * scale);
    if full.open {
        let picture_size = (size.0 as f32, size.1 as f32);
        // On another floor they are smaller, with what says which way beside
        // them. The arrow is as wide as the dot, outline included, so what is
        // beside it stays put as it moves round the edge.
        let (dot, hint) = if on_other_floor {
            (
                dot * OTHER_FLOOR_SIZE,
                Some(FLOOR_HINT_PIXELS * zoom * scale),
            )
        } else {
            (dot, None)
        };
        let arrow_place = frame::arrow_place(picture_size, player_at, ARROW_INSET * zoom * scale);
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
            size.0 as f32 - COMPASS_INSET * scale,
            size.1 as f32 - HOVER_LIFT * scale,
        );
        frame::compass(&mut canvas, foot, length * scale, COMPASS_LINE * scale);
    } else {
        frame::player_dot(&mut canvas, player_at, dot, outline);
        // North on the picture: the grid's north is `-y`.
        RoundFrame {
            centre: projection.middle,
            radius: clip,
            border: BORDER * scale,
            notch: NOTCH * scale,
            north: (-grid_right.1 as f32, -grid_down.1 as f32),
        }
        .draw(&mut canvas);
    }
}

/// Names what the mouse is over on the full map, in a box near the bottom of
/// the parchment.
#[allow(clippy::too_many_arguments)]
fn name_what_is_hovered(
    window: Single<&Window, With<PrimaryWindow>>,
    view: Res<State<GameView>>,
    camera: Single<&OrbitCamera>,
    player: Single<&SmoothPosition, With<Player>>,
    world: Option<Res<CurrentWorld>>,
    dungeon: Res<CurrentDungeon>,
    levels: Res<OtherLevels>,
    full: Res<FullMap>,
    mut holder: Single<&mut Visibility, With<HoverBox>>,
    mut text: Single<&mut Text, With<HoverText>>,
) {
    let view = *view.get();
    let name = window
        .cursor_position()
        .filter(|_| full.open)
        .and_then(|cursor| {
            let corner = full_map_corner(&window)
                + Vec2::new(FULL_MAP_MARGIN, FULL_MAP_MARGIN + TITLE_HEIGHT);
            let area = full_map_area(&window);
            let at = cursor - corner;
            if at.x < 0.0 || at.y < 0.0 || at.x >= area.x || at.y >= area.y {
                return None;
            }
            let (right, down) = Projection::NORTH_UP;
            let map = Projection {
                centre: full
                    .centre
                    .unwrap_or(levels.spot(&dungeon, (player.x, player.y))),
                middle: (area.x / 2.0, area.y / 2.0),
                units_per_pixel: units_per_pixel(view, &window, &camera) / f64::from(full.zoom),
                right,
                down,
            };
            let spot = map.to_grid(at.x, at.y);
            hovered_name(
                view,
                world.as_deref(),
                levels.shown(&dungeon),
                spot,
                ICON_PIXELS * full.zoom / 2.0,
                map.units_per_pixel,
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

/// Names the poi the mouse is over on the world map, in a box like the full
/// map's near the bottom of the window. The full map covers it when open.
fn name_hovered_token(
    view: Res<State<GameView>>,
    full: Res<FullMap>,
    hovered: Res<HoveredToken>,
    portals: Query<&Portal>,
    mut holder: Single<&mut Visibility, With<WorldHoverBox>>,
    mut text: Single<&mut Text, With<WorldHoverText>>,
) {
    let name = hovered
        .0
        .filter(|_| !full.open && *view.get() == GameView::WorldMap)
        .and_then(|token| portals.get(token).ok())
        .map(|Portal::Poi(poi)| poi_name(poi));
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
        .map(|poi| poi_name(&poi))
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
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use shared::dungeon::Level;

    use super::*;

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
        let mut windows = app.world_mut().query::<&mut Window>();
        let mut window = windows.single_mut(app.world_mut()).expect("a window");
        window.set_cursor_position(Some(at));
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

        let size = 720.0 * MINIMAP_OF_HEIGHT + 2.0 * (NOTCH + 1.0);
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
            let (corner, _) = floors::column(&window, 2);
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
        assert_eq!(minimap_across(&window), 1.0);
    }
}
