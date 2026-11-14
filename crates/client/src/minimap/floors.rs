//! The buttons for a site's known floors, in a column on the left of the full
//! map (in its markup: `full_map/floors`, a `full_map/go.<n>` for each).
//! Shown only when more than one floor is known.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{CORNER_GAP, FullMap, Room, full_map_area, full_map_picture_corner};
use crate::common::{CurrentDungeon, FirstFloor, GameView, OtherLevels, SmoothPosition};
use crate::dom::Dom;
use crate::player::Player;
use crate::widgets::{ButtonPressed, ButtonState, UiId};

/// A button's height and width (all the same, with room for "-10"), and the
/// gap between two.
pub(super) const BUTTON: f32 = 26.0;
pub(super) const BUTTON_WIDTH: f32 = 36.0;
pub(super) const BUTTON_GAP: f32 = 4.0;
/// How far round a button still counts as on it, for fingers (`dom::Reach`).
pub(super) const SIDE_REACH: f32 = 10.0;

/// The floors to show buttons for, as they read from the top: a building's
/// go up, a cave's go down. None unless the full map is open in a site, and
/// more than one is known.
pub(super) fn buttons(
    view: GameView,
    full_map_open: bool,
    dungeon: &CurrentDungeon,
    levels: &OtherLevels,
) -> Vec<u32> {
    let in_site = full_map_open && view == GameView::Dungeon;
    let Some(info) = dungeon.info.as_ref().filter(|_| in_site) else {
        return Vec::new();
    };
    let mut floors: Vec<u32> = levels.floors().chain([info.index]).collect();
    if floors.len() < 2 {
        return Vec::new();
    }
    floors.sort_unstable_by_key(|&floor| info.storey(floor));
    if info.building {
        floors.reverse();
    }
    floors
}

/// The column's top-left corner and its size, in logical pixels, centred down
/// the side of the full map.
pub(super) fn column(window: &Room, count: usize) -> (Vec2, Vec2) {
    let count = count as f32;
    let size = Vec2::new(
        BUTTON_WIDTH,
        count * BUTTON + (count - 1.0).max(0.0) * BUTTON_GAP,
    );
    let corner = full_map_picture_corner(window);
    let (left, middle) = (corner.x, corner.y + full_map_area(window).y / 2.0);
    // (Many floors in a small window: the top stays on screen.)
    (
        Vec2::new(left, (middle - size.y / 2.0).max(CORNER_GAP)),
        size,
    )
}

/// A floor's button pressed (`dom` finds it, reaching a little round it for
/// fingers): that floor, round the player's place. The map stays as far from
/// the player's place as it was dragged, so floors can be compared by going
/// up and down.
#[allow(clippy::too_many_arguments)]
pub(super) fn go_to_pressed(
    mut pressed: MessageReader<ButtonPressed>,
    ids: Query<&UiId>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    mut levels: ResMut<OtherLevels>,
    player: Single<&SmoothPosition, With<Player>>,
    mut full: ResMut<FullMap>,
) {
    let floors = buttons(*view.get(), full.open, &dungeon, &levels);
    let row = |said: &ButtonPressed| {
        let id = ids.get(said.0).ok()?;
        (id.parts() == ("full_map", "go"))
            .then(|| id.index())
            .flatten()
    };
    let Some(floor) = pressed
        .read()
        .filter_map(row)
        .last()
        .and_then(|at| floors.get(at))
    else {
        return;
    };
    let from = levels.spot(&dungeon, (player.x, player.y));
    levels.show(*floor);
    let to = levels.spot(&dungeon, (player.x, player.y));
    full.centre = full
        .centre
        .map(|at| (at.0 - from.0 + to.0, at.1 - from.1 + to.1));
    full.kept_on = Some(*floor);
    full.drag_from = None;
}

/// The corner minimap always shows the player's floor, so closing the full
/// map goes back to it.
pub(super) fn back_to_own_floor(full: Res<FullMap>, mut levels: ResMut<OtherLevels>) {
    if !full.open && levels.showing().is_some() {
        levels.show_own();
    }
}

/// What the column was last told: its buttons' floors, the first floor's
/// number, and where it is.
type ColumnShown = (Vec<u32>, u32, Vec2);

/// Puts the column on the full map, and makes its buttons again when the
/// floors they are for change.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_buttons(
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    levels: Res<OtherLevels>,
    first: Res<FirstFloor>,
    full: Res<FullMap>,
    mut dom: Dom,
    mut buttons_built: Query<(&UiId, &mut ButtonState)>,
    mut shown: Local<Option<ColumnShown>>,
) {
    let window = Room::of(&window, &ui_scale);
    let floors = buttons(*view.get(), full.open, &dungeon, &levels);
    let (corner, _) = column(&window, floors.len());
    let info = dungeon.info.as_ref();
    let current = info.map(|info| info.index);
    let chosen = levels.showing().or(current);
    // The floor shown is dark; the player's own is flat, shown or not.
    for (id, mut state) in &mut buttons_built {
        let (("full_map", "go"), Some(at)) = (id.parts(), id.index()) else {
            continue;
        };
        let floor = floors.get(at).copied();
        let look = match floor {
            Some(_) if floor == chosen => ButtonState::Selected,
            Some(_) if floor == current => ButtonState::Active,
            _ => ButtonState::Normal,
        };
        state.set_if_neq(look);
    }
    let was = shown.as_ref();
    if was.is_none_or(|(.., at)| *at != corner) {
        dom.set("full_map/floors", "left", &corner.x.to_string());
        dom.set("full_map/floors", "top", &corner.y.to_string());
    }
    if was.is_none_or(|(known, number, _)| *known != floors || *number != first.0) {
        dom.set_flag("full_map/floors", "hidden", floors.is_empty());
        dom.empty("full_map/floors");
        for (at, &floor) in floors.iter().enumerate() {
            let number = match info.map_or(floor as i32, |info| info.storey(floor)) {
                ..0 => "B".to_string(),
                up => (up + first.0 as i32).to_string(),
            };
            dom.copy_template("full_map/floor", "full_map/floors", at);
            dom.set_text(&format!("full_map/go.{at}"), number);
        }
    }
    *shown = Some((floors, first.0, corner));
}
