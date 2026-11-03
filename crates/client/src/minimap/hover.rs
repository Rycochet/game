//! Naming what the mouse is over: on the full map, the suns and moons, tokens.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::projection::Projection;
use super::{
    CORNER_GAP, FULL_MAP_MARGIN, FullMap, HoverBox, HoverText, ICON_PIXELS, Room, TITLE_HEIGHT,
    WorldHoverBox, WorldHoverText, full_map_area, full_map_corner, hovered_name, minimap_across,
    minimap_picture_size, minimap_trim, poi_name, under_the_sky, units_per_pixel,
};
use crate::common::{
    CurrentDungeon, CurrentWorld, GameView, OtherLevels, Portal, SkyNow, SmoothPosition, UiPointer,
};
use crate::player::Player;
use crate::view3d::{HoveredToken, OrbitCamera};

/// Names what the mouse is over on the full map, in a box near the bottom of
/// the parchment.
#[allow(clippy::too_many_arguments)]
pub(super) fn name_what_is_hovered(
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
pub(super) fn spot_on_full_map(
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
pub(super) fn hovered_body(
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
    let icons = super::sky::icons(&sky_now.bodies, middle, circle, trim);
    super::sky::under(&icons, middle, circle, at)
        .map(|number| super::sky::name(&sky_now.bodies[number]))
}

/// Names what the pointer is over in a box like the full map's near the bottom
/// of the window: a sun or moon by the minimap, or a poi on the world map. The
/// full map covers it when open.
#[allow(clippy::too_many_arguments)]
pub(super) fn name_hovered_token(
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
