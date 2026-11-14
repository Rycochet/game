//! Naming what the mouse is over: on the full map, the suns and moons, tokens.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use shared::hall::GateCode;

use super::gate_runes;
use super::projection::Projection;
use super::{
    CORNER_GAP, FULL_MAP_MARGIN, FullMap, HoverRunesPicture, ICON_PIXELS, Room, TITLE_HEIGHT,
    full_map_area, full_map_corner, hovered_name, minimap_across, minimap_picture_size,
    minimap_trim, under_the_sky, units_per_pixel,
};
use crate::common::{
    CurrentDungeon, CurrentWorld, GameView, HoveredInSky, OtherLevels, SkyNow, SmoothPosition,
};
use crate::dom::{Dom, Hit};
use crate::player::Player;
use crate::view3d::OrbitCamera;

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
    (hit, parts): (Res<Hit>, Query<&super::MapPart>),
    mut dom: Dom,
    mut runes: RunesBox,
    mut was: Local<Named>,
) {
    let window = Room::of(&window, &ui_scale);
    let view = *view.get();
    let mut gate = None;
    let on_picture = full.open && super::MapPart::Picture.holds(&hit.chain, &parts);
    let name = hit.at.filter(|_| on_picture).and_then(|cursor| {
        let centre = levels.spot(&dungeon, (player.x, player.y));
        let (spot, units_per_pixel) =
            spot_on_full_map(&window, cursor, view, &camera, &full, centre)?;
        // A World Gate in the Hall of Worlds: only its code, in runes, a
        // language nobody can read naming it (owner).
        gate = gate_runes::hovered_gate(levels.shown(&dungeon), spot);
        if gate.is_some() {
            return None;
        }
        hovered_name(
            view,
            world.as_deref(),
            levels.shown(&dungeon),
            spot,
            ICON_PIXELS * full.zoom / 2.0,
            units_per_pixel,
        )
    });
    let now = Named { name, gate };
    if *was == now {
        return;
    }
    // Each part is changed only when it differs from what the box showed.
    let differs = |pick: fn(&Named) -> bool| pick(&was) != pick(&now);
    if differs(|named| named.name.is_none() && named.gate.is_none()) {
        let nothing = now.name.is_none() && now.gate.is_none();
        dom.set_flag("full_map/hover", "hidden", nothing);
    }
    // The name's own box shows only with a name.
    if differs(|named| named.name.is_none()) {
        dom.set_flag("full_map/name", "hidden", now.name.is_none());
    }
    if let Some(name) = now.name.as_ref().filter(|_| was.name != now.name) {
        dom.set_text("full_map/words", name.clone());
    }
    if differs(|named| named.gate.is_none()) {
        dom.set_flag("full_map/runes", "hidden", now.gate.is_none());
    }
    show_runes(&mut runes, now.gate);
    *was = now;
}

/// What the full map's hover box last showed: a name, or a World Gate's code.
#[derive(Default, PartialEq)]
pub(super) struct Named {
    name: Option<String>,
    gate: Option<GateCode>,
}

/// The hover box's runes: their picture, the pictures, and the code drawn
/// last.
type RunesBox<'w, 's> = (
    Single<'w, 's, &'static mut ImageNode, With<HoverRunesPicture>>,
    ResMut<'w, Assets<Image>>,
    Local<'s, Option<GateCode>>,
);

/// Draws a gate's code for the hover box, when it is another.
fn show_runes((picture, images, drawn): &mut RunesBox, gate: Option<GateCode>) {
    if let Some(code) = gate
        && **drawn != Some(code)
    {
        **drawn = Some(code);
        picture.image = images.add(gate_runes::picture(code));
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

/// Names the sun or moon by the minimap that the pointer is on, for the hover
/// box (`hover`). The full map covers them when open; anything of the
/// interface in front of them hides them from the pointer (`dom::Hit`), and
/// the picture they are drawn on is `inert`, so a click on one reaches
/// whatever is under it.
pub(super) fn find_hovered_body(
    view: Res<State<GameView>>,
    full: Res<FullMap>,
    (sky_now, dungeon, hit): (Res<SkyNow>, Res<CurrentDungeon>, Res<Hit>),
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut hovered: ResMut<HoveredInSky>,
) {
    let window = Room::of(&window, &ui_scale);
    let body = hit
        .at
        .filter(|_| !full.open && hit.target.is_none())
        .and_then(|at| hovered_body(*view.get(), &dungeon, &sky_now, &window, at));
    if hovered.0 != body {
        hovered.0 = body;
    }
}
