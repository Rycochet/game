//! The maps' input: opening, dragging, zooming and clicks.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use shared::protocol::LevelKind;

use super::{
    CORNER_GAP, FullMap, KEEP_KNOWN_INSIDE, MAX_ZOOM, MIN_ZOOM, OverFullMap, PIXELS_PER_NOTCH,
    Room, ZOOM_PER_NOTCH, full_map_area, full_map_picture_corner, minimap_across,
    minimap_picture_size, over_full_map, shows_known, units_per_pixel,
};
use crate::common::{
    CurrentDungeon, CurrentWorld, GameView, OtherLevels, SmoothPosition, TouchGesture, UiPointer,
    in_interface,
};
use crate::player::Player;
use crate::view3d::OrbitCamera;

/// The maps get the input first, in `PreUpdate`: what they use is taken away
/// before the game's systems look. M or clicking the minimap opens the full
/// map; M, Esc, its close button or clicking outside it closes it. While it is
/// open the mouse and fingers are the map's: dragging moves it, and the wheel
/// or a pinch zooms it.
#[allow(clippy::too_many_arguments)]
pub(super) fn take_input(
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
    let buttons = super::floors::buttons(view, full.open, &dungeon, &levels);
    let button = cursor.and_then(|at| super::floors::under(&window, at, buttons.len()));
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
    // In the Hall of Worlds the map goes on for ever, so it is the player's
    // dot that is kept inside, not what is known (owner).
    let in_hall = shown
        .info
        .as_ref()
        .is_some_and(|info| matches!(info.kind, LevelKind::Hall { .. }));
    let known = |at: (f64, f64)| {
        if in_hall {
            (at.0 - spot.0).abs() <= half.0 && (at.1 - spot.1).abs() <= half.1
        } else {
            shows_known(view, world.as_deref(), shown, at, half)
        }
    };
    // Zoomed in, or somewhere else now (down the stairs, say): back towards
    // the player until something known shows. Another floor may show nothing
    // round the player's place: then the map is free until it finds something.
    let shown_floor = shown.info.as_ref().map(|info| info.index);
    if full.centre.is_some_and(known) {
        full.kept_on = None;
    }
    let kept = full.kept_on.is_some() && full.kept_on == shown_floor;
    if let Some(at) = full.centre.filter(|&at| !kept && !known(at) && known(spot)) {
        full.centre = Some(super::projection::slide(spot, at, known));
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
            super::projection::slide(centre, wanted, known)
        } else {
            wanted
        });
    }
}
