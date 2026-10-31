//! The buttons for a site's known floors, in a column on the left of the full
//! map. Shown only when more than one floor is known.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{CORNER_GAP, FullMap, INK, full_map_area, full_map_picture_corner};
use crate::common::{CurrentDungeon, FirstFloor, GameView, OtherLevels};
use crate::ui::raised_box;

/// A button's height and width (all the same, with room for "-10"), and the
/// gap between two.
pub(super) const BUTTON: f32 = 26.0;
const BUTTON_WIDTH: f32 = 36.0;
pub(super) const BUTTON_GAP: f32 = 4.0;

/// The floor being looked at: darker, and flat where the others stand up.
const SHOWN_BACKGROUND: Color = Color::srgb(0.42, 0.32, 0.2);
const SHOWN_TEXT: Color = Color::srgb(0.95, 0.87, 0.7);

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
        GlobalZIndex(25),
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
    floors.sort_unstable();
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
    let from = at - corner;
    if count == 0 || from.x < 0.0 || from.y < 0.0 || from.x >= size.x || from.y >= size.y {
        return None;
    }
    let row = (from.y / (BUTTON + BUTTON_GAP)) as usize;
    // Not in the gap below a button.
    (from.y - row as f32 * (BUTTON + BUTTON_GAP) < BUTTON).then_some(row)
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
    let now = (floors, shown, current, first.0);
    if *drawn == now {
        return;
    }
    commands.entity(entity).despawn_children();
    commands.entity(entity).with_children(|column| {
        for &floor in &now.0 {
            let number = (floor + first.0).to_string();
            let size = Node {
                width: px(BUTTON_WIDTH),
                height: px(BUTTON),
                border: UiRect::all(px(2)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            };
            let text = (
                TextFont::from_font_size(16.0),
                TextLayout::justify(Justify::Center),
            );
            if Some(floor) == shown.or(current) {
                column
                    .spawn((
                        size,
                        BackgroundColor(SHOWN_BACKGROUND),
                        BorderColor::from(INK),
                    ))
                    .with_child((Text::new(number), text, TextColor(SHOWN_TEXT)));
            } else {
                // (Inserted over the box's own size and padding.)
                let mut button = column.spawn(raised_box());
                button.insert(size);
                // The player's floor is flat, shown or not.
                if Some(floor) == current {
                    button.insert(BorderColor::from(INK));
                }
                button.with_child((Text::new(number), text, TextColor(INK)));
            }
        }
    });
    *drawn = now;
}
