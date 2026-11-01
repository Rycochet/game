//! The buttons for a site's known floors, in a column on the left of the full
//! map. Shown only when more than one floor is known.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{CORNER_GAP, FullMap, full_map_area, full_map_picture_corner};
use crate::common::{CurrentDungeon, FirstFloor, GameView, OtherLevels};
use crate::widgets::{ButtonState, layer_z, spawn_button};

/// A button's height and width (all the same, with room for "-10"), and the
/// gap between two.
pub(super) const BUTTON: f32 = 26.0;
const BUTTON_WIDTH: f32 = 36.0;
pub(super) const BUTTON_GAP: f32 = 4.0;
/// How far either side of a button still counts as on it.
const SIDE_REACH: f32 = 10.0;

const FONT: f32 = 16.0;

#[derive(Component)]
pub(super) struct FloorColumn;

pub(super) fn spawn_column(mut commands: Commands) {
    commands.spawn((
        FloorColumn,
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            row_gap: px(BUTTON_GAP),
            ..default()
        },
        // Over the full map.
        GlobalZIndex(layer_z(super::FULL_MAP_LAYER) + 2),
    ));
}

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
pub(super) fn column(window: &Window, count: usize) -> (Vec2, Vec2) {
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

/// Which button, counted from the top, is at a point on the window.
pub(super) fn under(window: &Window, at: Vec2, count: usize) -> Option<usize> {
    let (corner, size) = column(window, count);
    // Half the gap each way belongs to the nearest button, and a little to the
    // sides, for fingers.
    let from = at - corner + Vec2::new(SIDE_REACH, BUTTON_GAP / 2.0);
    let reach = size + Vec2::new(2.0 * SIDE_REACH, BUTTON_GAP);
    if count == 0 || from.x < 0.0 || from.y < 0.0 || from.x >= reach.x || from.y >= reach.y {
        return None;
    }
    Some(((from.y / (BUTTON + BUTTON_GAP)) as usize).min(count - 1))
}

/// The corner minimap always shows the player's floor, so closing the full
/// map goes back to it.
pub(super) fn back_to_own_floor(full: Res<FullMap>, mut levels: ResMut<OtherLevels>) {
    if !full.open && levels.showing().is_some() {
        levels.show_own();
    }
}

/// What the buttons say: the floors, the one shown if not the player's, the
/// player's, and the first floor's number.
type ButtonsDrawn = (Vec<u32>, Option<u32>, Option<u32>, u32);

/// Puts the column on the full map, and builds its buttons again when
/// what they say changes.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_buttons(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    levels: Res<OtherLevels>,
    first: Res<FirstFloor>,
    full: Res<FullMap>,
    column_node: Single<(Entity, &mut Node), With<FloorColumn>>,
    mut drawn: Local<ButtonsDrawn>,
) {
    let floors = buttons(*view.get(), full.open, &dungeon, &levels);
    let (entity, mut node) = column_node.into_inner();
    let (corner, _) = column(&window, floors.len());
    node.left = px(corner.x);
    node.top = px(corner.y);

    let shown = levels.showing();
    let current = dungeon.info.as_ref().map(|info| info.index);
    let storey = |floor: u32| {
        dungeon
            .info
            .as_ref()
            .map_or(floor as i32, |i| i.storey(floor))
    };
    let now = (floors, shown, current, first.0);
    if *drawn == now {
        return;
    }
    commands.entity(entity).despawn_children();
    commands.entity(entity).with_children(|column| {
        for &floor in &now.0 {
            let number = match storey(floor) {
                ..0 => "B".to_string(),
                up => (up + first.0 as i32).to_string(),
            };
            let state = if Some(floor) == shown.or(current) {
                ButtonState::Selected
            } else if Some(floor) == current {
                // The player's floor is flat, shown or not.
                ButtonState::Active
            } else {
                ButtonState::Normal
            };
            let size = Node {
                width: px(BUTTON_WIDTH),
                height: px(BUTTON),
                ..default()
            };
            spawn_button(column, state, number, FONT, size);
        }
    });
    *drawn = now;
}
