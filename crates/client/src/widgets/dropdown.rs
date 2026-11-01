//! A button showing a choice, which opens a list to choose from; with search, the
//! list also has a text box that takes the keyboard and narrows it down. While a
//! list is open it has all the input. It scrolls by the wheel, a finger, or the
//! arrow keys, a window of rows at a time, as nodes outside a clip still take
//! presses here.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use bevy::window::PrimaryWindow;

use super::filter_list::{HighlightStep, matches_filter, next_highlight};
use super::fullscreen::layer_z;
use super::text_box::{RaisesKeyboard, TextBox, TextEdited, TextFocus, spawn_text_box};
use super::{
    ButtonPressed, ButtonState, INK, Pressable, panel, raised_border, spawn_button, stops_presses,
};
use crate::common::TouchGesture;

/// Above anything a dropdown can be on.
const LIST_LAYER: i32 = 4;
/// How many options show at once.
const ROWS: usize = 8;
const ROW: f32 = 30.0;
const GAP: f32 = 3.0;
const EDGE: f32 = 3.0;
const FONT: f32 = 16.0;
const SEARCH_HEIGHT: f32 = 32.0;
/// How far a finger drags to move by a row.
const DRAG_PER_ROW: f32 = 30.0;
const LEAST_WIDTH: f32 = 160.0;

/// A choice among `options`. Set `selected` to change it.
#[derive(Component, Clone, Debug, Default)]
pub struct Dropdown {
    pub options: Vec<String>,
    pub selected: usize,
    pub searchable: bool,
}

/// The player chose an option of this dropdown.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DropdownChosen(pub Entity, pub usize);

/// Clicks outside the list, which close it.
#[derive(Component)]
pub(super) struct ListBacking;

/// Holds the rows on show.
#[derive(Component)]
pub(super) struct ListRows;

/// An option in an open list, by its index in `options`.
#[derive(Component, Clone, Copy)]
pub(super) struct ListOption(usize);

/// The list that is open.
#[derive(Resource, Default)]
pub(super) struct OpenList(Option<List>);

pub(super) struct List {
    dropdown: Entity,
    backing: Entity,
    search: Option<Entity>,
    filter: String,
    /// Which of those on show the arrow keys are on.
    highlight: Option<usize>,
    /// The first of those on show that is drawn.
    first: usize,
    dragged: f32,
    drawn: Option<(Vec<usize>, usize, Option<usize>)>,
}

/// A dropdown, with `node` for its size and where it goes.
pub fn spawn_dropdown<'a>(
    parent: &'a mut ChildSpawnerCommands,
    dropdown: Dropdown,
    font_size: f32,
    node: Node,
) -> EntityCommands<'a> {
    let label = label_of(&dropdown);
    let searchable = dropdown.searchable;
    let mut button = spawn_button(parent, ButtonState::Normal, label, font_size, node);
    button.insert((dropdown, Pressable));
    if searchable {
        button.insert(RaisesKeyboard);
    }
    button
}

fn label_of(dropdown: &Dropdown) -> String {
    let chosen = dropdown.options.get(dropdown.selected);
    format!("{} v", chosen.map_or("", String::as_str))
}

/// The options matching what has been typed, by index.
fn on_show(options: &[String], filter: &str) -> Vec<usize> {
    (0..options.len())
        .filter(|&at| matches_filter(&options[at], filter))
        .collect()
}

/// The first row to draw so that `highlight` shows, from `first`, among `count`.
fn window_start(first: usize, highlight: Option<usize>, count: usize) -> usize {
    let most = count.saturating_sub(ROWS);
    let first = match highlight {
        Some(at) if at < first => at,
        Some(at) if at >= first + ROWS => at + 1 - ROWS,
        _ => first,
    };
    first.min(most)
}

/// Where a list goes under a button: on the screen, or above it if there is no
/// room below.
fn list_corner(button: Rect, height: f32, width: f32, window: Vec2) -> Vec2 {
    let below = button.max.y + GAP;
    let y = if below + height <= window.y {
        below
    } else {
        (button.min.y - GAP - height).max(0.0)
    };
    Vec2::new(button.min.x.min(window.x - width).max(0.0), y)
}

/// Opens a pressed dropdown, and while one is open, takes all the input. Before
/// `WidgetSystems::Modal`, so nothing else sees it.
#[allow(clippy::too_many_arguments)]
pub(super) fn run_dropdowns(
    mut commands: Commands,
    mut open: ResMut<OpenList>,
    mut pressed: MessageReader<ButtonPressed>,
    mut edited: MessageReader<TextEdited>,
    mut focus: ResMut<TextFocus>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    mut gesture: ResMut<TouchGesture>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut dropdowns: Query<(&mut Dropdown, &ComputedNode, &UiGlobalTransform)>,
    options: Query<&ListOption>,
    boxes: Query<&TextBox>,
    backing: Query<(), With<ListBacking>>,
    mut chosen: MessageWriter<DropdownChosen>,
) {
    let presses: Vec<Entity> = pressed.read().map(|press| press.0).collect();
    let edits: Vec<Entity> = edited.read().map(|edit| edit.0).collect();
    let Some(list) = &mut open.0 else {
        let opening = presses.iter().find_map(|&press| {
            dropdowns
                .get(press)
                .ok()
                .map(|(dropdown, node, transform)| (press, dropdown, node, transform))
        });
        if let Some((entity, dropdown, node, transform)) = opening {
            let scale = node.inverse_scale_factor;
            let place = Rect::from_center_size(transform.translation * scale, node.size() * scale);
            let area = Vec2::new(window.width(), window.height());
            let new = spawn_list(&mut commands, entity, dropdown, place, area);
            focus.0 = new.search;
            open.0 = Some(new);
        }
        return;
    };

    if let Some(search) = list.search {
        focus.0 = Some(search);
        if edits.contains(&search)
            && let Ok(text_box) = boxes.get(search)
        {
            list.filter.clone_from(&text_box.text);
            list.highlight = None;
            list.first = 0;
        }
    }
    let Ok((mut dropdown, ..)) = dropdowns.get_mut(list.dropdown) else {
        close(&mut commands, &mut open, &mut focus);
        return;
    };
    let shown = on_show(&dropdown.options, &list.filter);
    for (key, step) in [
        (KeyCode::ArrowUp, HighlightStep::Up),
        (KeyCode::ArrowDown, HighlightStep::Down),
    ] {
        if keys.just_pressed(key) {
            list.highlight = next_highlight(list.highlight, shown.len(), shown.len(), step);
        }
    }
    let wheel = scroll.delta.y;
    if wheel != 0.0 {
        list.highlight = None;
        list.first = if wheel > 0.0 {
            list.first.saturating_sub(1)
        } else {
            list.first + 1
        };
    }
    list.dragged += gesture.drag.y;
    while list.dragged.abs() >= DRAG_PER_ROW {
        list.highlight = None;
        if list.dragged > 0.0 {
            list.first = list.first.saturating_sub(1);
            list.dragged -= DRAG_PER_ROW;
        } else {
            list.first += 1;
            list.dragged += DRAG_PER_ROW;
        }
    }
    list.first = window_start(list.first, list.highlight, shown.len());

    let mut choice = None;
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        choice = shown.get(list.highlight.unwrap_or(0)).copied();
    }
    for &press in &presses {
        if let Ok(option) = options.get(press) {
            choice = Some(option.0);
        }
    }
    let mut closing =
        keys.just_pressed(KeyCode::Escape) || presses.iter().any(|&press| backing.contains(press));
    if let Some(at) = choice {
        dropdown.selected = at;
        chosen.write(DropdownChosen(list.dropdown, at));
        closing = true;
    }
    keys.reset_all();
    mouse.reset(MouseButton::Right);
    scroll.delta = Vec2::ZERO;
    *gesture = TouchGesture::default();
    if closing {
        close(&mut commands, &mut open, &mut focus);
    }
}

fn close(commands: &mut Commands, open: &mut OpenList, focus: &mut TextFocus) {
    if let Some(list) = open.0.take() {
        commands.entity(list.backing).despawn();
        if focus.0 == list.search {
            focus.0 = None;
        }
    }
}

fn spawn_list(
    commands: &mut Commands,
    dropdown: Entity,
    of: &Dropdown,
    place: Rect,
    window: Vec2,
) -> List {
    let width = place.width().max(LEAST_WIDTH);
    let search_room = if of.searchable {
        SEARCH_HEIGHT + GAP
    } else {
        0.0
    };
    let rows = of.options.len().clamp(1, ROWS) as f32;
    let height = search_room + rows * (ROW + GAP) + 2.0 * EDGE + GAP;
    let corner = list_corner(place, height, width, window);
    let backing = commands
        .spawn((
            ListBacking,
            Pressable,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            GlobalZIndex(layer_z(LIST_LAYER)),
        ))
        .id();
    let holder = commands.spawn((panel(), raised_border())).id();
    commands.entity(holder).insert((
        stops_presses(holder),
        Node {
            position_type: PositionType::Absolute,
            left: px(corner.x),
            top: px(corner.y),
            width: px(width),
            border: UiRect::all(px(EDGE)),
            padding: UiRect::all(px(GAP)),
            row_gap: px(GAP),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    ));
    commands.entity(backing).add_child(holder);
    let mut search = None;
    commands.entity(holder).with_children(|inside| {
        if of.searchable {
            let text_box = TextBox {
                prefix: "Search: ",
                ..default()
            };
            let node = Node {
                height: px(SEARCH_HEIGHT),
                min_height: px(SEARCH_HEIGHT),
                ..default()
            };
            search = Some(spawn_text_box(inside, text_box, FONT, node).id());
        }
        inside.spawn((
            ListRows,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(GAP),
                ..default()
            },
        ));
    });
    List {
        dropdown,
        backing,
        search,
        filter: String::new(),
        highlight: None,
        first: 0,
        dragged: 0.0,
        drawn: None,
    }
}

/// Keeps a dropdown's words and an open list's rows up to date.
pub(super) fn show_dropdowns(
    mut commands: Commands,
    mut open: ResMut<OpenList>,
    changed: Query<(&Dropdown, &Children), Changed<Dropdown>>,
    dropdowns: Query<&Dropdown>,
    mut labels: Query<&mut Text>,
    rows: Query<Entity, With<ListRows>>,
) {
    for (dropdown, children) in &changed {
        for &child in children {
            if let Ok(mut text) = labels.get_mut(child) {
                text.0 = label_of(dropdown);
            }
        }
    }
    let Some(list) = &mut open.0 else {
        return;
    };
    let (Ok(dropdown), Ok(holder)) = (dropdowns.get(list.dropdown), rows.single()) else {
        return;
    };
    let shown = on_show(&dropdown.options, &list.filter);
    let now = (shown, list.first, list.highlight);
    if list.drawn.as_ref() == Some(&now) {
        return;
    }
    commands.entity(holder).despawn_children();
    commands.entity(holder).with_children(|holder| {
        if now.0.is_empty() {
            holder.spawn((
                Text::new("No matches"),
                TextFont::from_font_size(FONT),
                TextColor(INK),
            ));
        }
        for (row, &at) in now.0.iter().enumerate().skip(now.1).take(ROWS) {
            let state = if now.2 == Some(row) {
                ButtonState::Selected
            } else {
                ButtonState::Normal
            };
            let node = Node {
                height: px(ROW),
                ..default()
            };
            spawn_button(holder, state, dropdown.options[at].clone(), FONT, node)
                .insert((ListOption(at), Pressable));
        }
        if now.0.len() > ROWS {
            let last = (now.1 + ROWS).min(now.0.len());
            holder.spawn((
                Text::new(format!("{} to {last} of {}", now.1 + 1, now.0.len())),
                TextFont::from_font_size(FONT),
                TextLayout::justify(Justify::Center),
                TextColor(INK),
            ));
        }
    });
    list.drawn = Some(now);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_narrows_the_options_keeping_their_indexes() {
        let options: Vec<String> = ["Castle", "Fort", "Stone Tower"].map(String::from).into();
        assert_eq!(on_show(&options, ""), [0, 1, 2]);
        assert_eq!(on_show(&options, "T"), [0, 1, 2]);
        assert_eq!(on_show(&options, "tower"), [2]);
        assert!(on_show(&options, "cave").is_empty());
    }

    #[test]
    fn the_rows_drawn_follow_the_highlight_and_stop_at_the_end() {
        assert_eq!(window_start(0, Some(3), 40), 0);
        assert_eq!(window_start(0, Some(ROWS), 40), 1);
        assert_eq!(window_start(10, Some(4), 40), 4);
        assert_eq!(window_start(100, None, 40), 40 - ROWS);
        assert_eq!(window_start(5, None, 3), 0);
    }

    #[test]
    fn a_list_goes_under_its_button_or_above_if_there_is_no_room() {
        let window = Vec2::new(800.0, 600.0);
        let button = Rect::new(10.0, 100.0, 110.0, 130.0);
        assert_eq!(
            list_corner(button, 200.0, 100.0, window),
            Vec2::new(10.0, 130.0 + GAP)
        );
        let low = Rect::new(750.0, 500.0, 790.0, 530.0);
        let corner = list_corner(low, 200.0, 100.0, window);
        assert_eq!(corner, Vec2::new(700.0, 500.0 - GAP - 200.0));
    }
}
