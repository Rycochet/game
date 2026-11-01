//! The menu itself: the warning button, the full-screen lists (the actions, and
//! a submenu above them), and the keys and presses that work them.

use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use shared::poi::PoiKind;

use super::FactSources;
use super::actions::{DebugAction, Facts, kind_name};
use super::panels::GenerateKind;
use crate::common::{GameView, TouchGesture};
use crate::minimap::FullMap;
use crate::widgets::{
    ButtonPressed, ButtonState, HighlightStep, LAYER_EDGE, Pressable, SoftKey, SoftKeyPressed,
    TextBox, TextEdited, TextFocus, layer_share, matches_filter, next_highlight, panel_title,
    spawn_button, spawn_check_button, spawn_close_button, spawn_fullscreen_layer,
    spawn_icon_button, spawn_text_box,
};

/// How long a toggled entry stays on show before the menu closes.
const TOGGLE_SECONDS: f32 = 1.0;

/// The layer of the actions, and of a submenu above them.
const ACTIONS: usize = 0;
const SUBMENU: usize = 1;

const WARNING_BUTTON: f32 = 34.0;
const CORNER_GAP: f32 = 10.0;

const MARGIN: f32 = 16.0;
const TITLE_ROW: f32 = 36.0;
/// Round the close button, still on it, for fingers.
const CLOSE_MARGIN: f32 = 4.0;
const FILTER_HEIGHT: f32 = 38.0;
const ROW: f32 = 34.0;
const GAP: f32 = 4.0;
const COLUMN_GAP: f32 = GAP * 2.0;
const COLUMN_WIDTH: f32 = 300.0;
const PAGE_BUTTON: f32 = 34.0;
const FONT: f32 = 18.0;

/// What has been typed in one layer's list, which entry the arrow keys are on
/// (counted among those on show), and the first column drawn.
#[derive(Default)]
struct Level {
    filter: String,
    highlight: Option<usize>,
    first_column: usize,
}

/// The debug menu: whether it is open, and its lists.
#[derive(Resource, Default)]
pub struct DebugMenu {
    pub open: bool,
    actions: Level,
    /// The submenu, if it is open: for now the only one, Generate POI's.
    submenu: Option<Level>,
    /// Counting down to closing, after a toggle.
    closing: Option<Timer>,
}

impl DebugMenu {
    /// Open, with this already typed (dev tools).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_with(filter: String) -> Self {
        Self {
            open: true,
            actions: Level {
                filter,
                ..default()
            },
            ..default()
        }
    }

    fn set_open(&mut self, open: bool) {
        *self = Self { open, ..default() };
    }

    /// Opens the menu at its submenu.
    pub(super) fn open_submenu(&mut self) {
        self.open = true;
        self.submenu = Some(Level::default());
    }

    fn top_layer(&self) -> usize {
        if self.submenu.is_some() {
            SUBMENU
        } else {
            ACTIONS
        }
    }

    fn level(&self, layer: usize) -> Option<&Level> {
        if layer == ACTIONS {
            Some(&self.actions)
        } else {
            self.submenu.as_ref()
        }
    }
}

#[derive(Component)]
pub(super) struct WarningButton;

/// A layer of the menu.
#[derive(Component, Clone, Copy)]
pub(super) struct MenuRoot(usize);

/// Holds the close button, and takes the presses on it.
#[derive(Component)]
pub(super) struct MenuClose;

#[derive(Component, Clone, Copy)]
pub(super) struct FilterBox(usize);

#[derive(Component, Clone, Copy)]
pub(super) struct MenuGrid(usize);

/// A button moving a layer's columns on or back; they only show when they can't
/// all fit.
#[derive(Component, Clone, Copy)]
pub(super) struct PageButton {
    layer: usize,
    forward: bool,
}

/// What an entry in a list is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    Action(DebugAction),
    Kind(PoiKind),
}

#[derive(Component, Clone, Copy)]
pub(super) struct MenuEntry(Entry);

/// A warning triangle with an exclamation mark, drawn here so there is no file
/// to ship for it.
fn warning_icon() -> Image {
    const SIZE: usize = 64;
    let mut bytes = vec![0; SIZE * SIZE * 4];
    let middle = SIZE as f32 / 2.0;
    for y in 4..60 {
        let half = (y - 4) as f32 / 56.0 * 30.0;
        for x in 0..SIZE {
            let across = (x as f32 + 0.5 - middle).abs();
            if across > half {
                continue;
            }
            let edge = across > half - 5.0 || y >= 54;
            let mark = across < 3.5 && ((22..41).contains(&y) || (44..50).contains(&y));
            let colour = if edge || mark {
                [60, 40, 20, 255]
            } else {
                [245, 200, 40, 255]
            };
            bytes[(y * SIZE + x) * 4..][..4].copy_from_slice(&colour);
        }
    }
    Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

pub(super) fn spawn_menu(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let icon = images.add(warning_icon());
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(CORNER_GAP),
                left: px(CORNER_GAP),
                ..default()
            },
            GlobalZIndex(15),
        ))
        .with_children(|holder| {
            let node = Node {
                width: px(WARNING_BUTTON),
                height: px(WARNING_BUTTON),
                ..default()
            };
            spawn_icon_button(holder, ButtonState::Normal, icon, node)
                .insert((WarningButton, Pressable));
        });
    spawn_layer(&mut commands, ACTIONS, "Debug Menu");
    spawn_layer(&mut commands, SUBMENU, DebugAction::GeneratePoi.name());
}

/// One layer of the menu, hidden: its title, filter box and an empty grid.
fn spawn_layer(commands: &mut Commands, layer: usize, title: &str) {
    let (root, list) = spawn_fullscreen_layer(
        commands,
        layer as i32,
        Node {
            padding: UiRect::all(px(MARGIN)),
            flex_direction: FlexDirection::Column,
            row_gap: px(GAP * 3.0),
            ..default()
        },
    );
    commands.entity(root).insert(MenuRoot(layer));
    commands.entity(list).with_children(|root| {
        root.spawn(Node {
            height: px(TITLE_ROW),
            min_height: px(TITLE_ROW),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|bar| {
            bar.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(0),
                column_gap: px(GAP),
                ..default()
            })
            .with_children(|pages| {
                for (label, forward) in [("<", false), (">", true)] {
                    let node = Node {
                        width: px(PAGE_BUTTON),
                        height: px(PAGE_BUTTON),
                        ..default()
                    };
                    spawn_button(pages, ButtonState::Normal, label, FONT, node).insert((
                        PageButton { layer, forward },
                        Pressable,
                        Visibility::Hidden,
                    ));
                }
            });
            bar.spawn(panel_title(title));
            bar.spawn((
                MenuClose,
                Pressable,
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0),
                    padding: UiRect::all(px(CLOSE_MARGIN)),
                    ..default()
                },
            ))
            .with_children(|holder| {
                spawn_close_button(holder, default());
            });
        });
        let text_box = TextBox {
            prefix: "Filter: ",
            ..default()
        };
        let node = Node {
            height: px(FILTER_HEIGHT),
            min_height: px(FILTER_HEIGHT),
            ..default()
        };
        spawn_text_box(root, text_box, FONT, node).insert(FilterBox(layer));
        root.spawn((
            MenuGrid(layer),
            Node {
                column_gap: px(COLUMN_GAP),
                ..default()
            },
        ));
    });
}

/// How many entries fit in a column of this layer, in a window this tall.
fn rows_in(window_height: f32, layer: usize) -> usize {
    let share = layer_share(layer as i32);
    let inside = window_height * share - 2.0 * (LAYER_EDGE + MARGIN);
    let room = inside - TITLE_ROW - FILTER_HEIGHT - 2.0 * GAP * 3.0;
    (((room + GAP) / (ROW + GAP)).floor() as usize).max(1)
}

/// How many columns of this layer fit, in a window this wide.
fn columns_in(window_width: f32, layer: usize) -> usize {
    let share = layer_share(layer as i32);
    let room = window_width * share - 2.0 * (LAYER_EDGE + MARGIN);
    (((room + COLUMN_GAP) / (COLUMN_WIDTH + COLUMN_GAP)).floor() as usize).max(1)
}

/// How many columns `count` entries fill.
fn columns_of(count: usize, rows: usize) -> usize {
    count.div_ceil(rows)
}

/// The first column to draw, from `first`, so the highlighted entry shows and
/// no room is wasted at the end.
fn first_column(
    first: usize,
    highlight: Option<usize>,
    count: usize,
    rows: usize,
    fit: usize,
) -> usize {
    let first = match highlight.map(|at| at / rows) {
        Some(column) if column < first => column,
        Some(column) if column >= first + fit => column + 1 - fit,
        _ => first,
    };
    first.min(columns_of(count, rows).saturating_sub(fit))
}

/// The actions on show, in order.
fn on_show(filter: &str) -> Vec<DebugAction> {
    DebugAction::ALL
        .into_iter()
        .filter(|action| action.matches(filter))
        .collect()
}

/// The kinds of poi on show in the submenu: alphabetical, as the menu is.
fn kinds_on_show(filter: &str) -> Vec<PoiKind> {
    let mut kinds: Vec<PoiKind> = PoiKind::ALL
        .into_iter()
        .filter(|&kind| matches_filter(&kind_name(kind), filter))
        .collect();
    kinds.sort_by_key(|&kind| kind_name(kind));
    kinds
}

/// The entries on show in a layer.
fn entries_on_show(layer: usize, filter: &str) -> Vec<Entry> {
    if layer == ACTIONS {
        on_show(filter).into_iter().map(Entry::Action).collect()
    } else {
        kinds_on_show(filter).into_iter().map(Entry::Kind).collect()
    }
}

/// What Return chooses: the highlighted entry, or with none, the first that
/// can be used. A disabled entry that is highlighted does nothing.
fn return_chooses(
    shown: &[DebugAction],
    highlight: Option<usize>,
    facts: &Facts,
) -> Option<DebugAction> {
    match highlight {
        Some(at) => shown.get(at).copied(),
        None => shown.iter().copied().find(|a| a.is_available(facts)),
    }
    .filter(|action| action.is_available(facts))
}

/// Presses on these close the top layer: its close button, and outside it.
type ClosesLayer = Or<(With<MenuClose>, With<MenuRoot>)>;

/// What the widgets say has happened.
#[derive(SystemParam)]
pub(super) struct MenuReaders<'w, 's> {
    pressed: MessageReader<'w, 's, ButtonPressed>,
    edited: MessageReader<'w, 's, TextEdited>,
    soft_keys: MessageReader<'w, 's, SoftKeyPressed>,
}

/// The parts of the menu that take input.
#[derive(SystemParam)]
pub(super) struct MenuNodes<'w, 's> {
    filter_boxes: Query<'w, 's, (Entity, &'static FilterBox, &'static TextBox)>,
    entries: Query<'w, 's, &'static MenuEntry>,
    pages: Query<'w, 's, &'static PageButton>,
    warning: Query<'w, 's, (), With<WarningButton>>,
    close: Query<'w, 's, (), ClosesLayer>,
}

/// The menu gets input after the widgets have looked for presses on it, and
/// before the maps and the game: while it is open, it has all of it.
#[allow(clippy::too_many_arguments)]
pub(super) fn take_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    mut gesture: ResMut<TouchGesture>,
    readers: MenuReaders,
    nodes: MenuNodes,
    mut focus: ResMut<TextFocus>,
    full_map: Res<FullMap>,
    window: Single<&Window, With<PrimaryWindow>>,
    sources: FactSources,
    // Real time: game time is capped per frame, so it runs slow when unfocused.
    time: Res<Time<Real>>,
    mut menu: ResMut<DebugMenu>,
    mut actions: MessageWriter<DebugAction>,
    mut generate: MessageWriter<GenerateKind>,
) {
    let MenuReaders {
        mut pressed,
        mut edited,
        mut soft_keys,
    } = readers;
    let MenuNodes {
        filter_boxes,
        entries,
        pages,
        warning,
        close,
    } = nodes;
    let presses: Vec<Entity> = pressed.read().map(|press| press.0).collect();
    let edits: Vec<Entity> = edited.read().map(|edit| edit.0).collect();
    let soft: Vec<SoftKey> = soft_keys.read().map(|key| key.0).collect();
    // Not over the full map: it would be the bigger of the two.
    let toggled = !full_map.open
        && (keys.just_pressed(KeyCode::Backquote)
            || presses.iter().any(|&entity| warning.contains(entity)));
    if toggled {
        let open = !menu.open;
        menu.set_open(open);
    }
    let on_filter_box =
        |focus: &TextFocus| filter_boxes.iter().any(|(box_, ..)| focus.0 == Some(box_));
    if !menu.open {
        if on_filter_box(&focus) {
            focus.0 = None;
        }
        return;
    }

    let layer = menu.top_layer();
    focus.0 = filter_boxes
        .iter()
        .find(|(_, filter_box, _)| filter_box.0 == layer)
        .map(|(box_, ..)| box_);
    let mut back = presses.iter().any(|&entity| close.contains(entity))
        || soft.contains(&SoftKey::Escape)
        || keys.just_pressed(KeyCode::Escape);
    let mut closing = false;
    if let Some(timer) = &mut menu.closing {
        closing |= timer.tick(time.delta()).is_finished();
    } else if !toggled {
        let facts = sources.facts();
        let (rows, fit) = (
            rows_in(window.height(), layer),
            columns_in(window.width(), layer),
        );
        let wheel = scroll.delta.y;
        let edited_text = filter_boxes
            .iter()
            .find(|(box_, filter_box, _)| filter_box.0 == layer && edits.contains(box_))
            .map(|(.., text_box)| text_box.text.clone());
        let level = if layer == ACTIONS {
            &mut menu.actions
        } else {
            menu.submenu.as_mut().expect("the top layer")
        };
        if let Some(text) = edited_text {
            level.filter = text;
            level.highlight = None;
            level.first_column = 0;
        }
        let shown = entries_on_show(layer, &level.filter);
        for (key, step) in [
            (KeyCode::ArrowUp, HighlightStep::Up),
            (KeyCode::ArrowDown, HighlightStep::Down),
            (KeyCode::ArrowLeft, HighlightStep::Left),
            (KeyCode::ArrowRight, HighlightStep::Right),
        ] {
            if keys.just_pressed(key) {
                level.highlight = next_highlight(level.highlight, shown.len(), rows, step);
            }
        }
        // Moving the columns by hand, the highlight would only pull them back.
        let mut moved = wheel != 0.0;
        let mut forward = wheel < 0.0;
        for &press in &presses {
            if let Ok(page) = pages.get(press) {
                moved = true;
                forward = page.forward;
            }
        }
        if moved {
            level.highlight = None;
            level.first_column = if forward {
                level.first_column + 1
            } else {
                level.first_column.saturating_sub(1)
            };
        }
        level.first_column =
            first_column(level.first_column, level.highlight, shown.len(), rows, fit);

        let mut chosen = None;
        if keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::NumpadEnter)
            || soft.contains(&SoftKey::Return)
        {
            chosen = if layer == ACTIONS {
                return_chooses(&on_show(&level.filter), level.highlight, &facts).map(Entry::Action)
            } else {
                shown.get(level.highlight.unwrap_or(0)).copied()
            };
        }
        for &press in &presses {
            if let Ok(entry) = entries.get(press) {
                chosen = Some(entry.0);
            }
        }
        match chosen {
            Some(Entry::Action(action)) if action.has_submenu() => {
                menu.open_submenu();
                back = false;
            }
            Some(Entry::Action(action)) => {
                actions.write(action);
                if action.is_on(&facts).is_some() {
                    menu.closing = Some(Timer::from_seconds(TOGGLE_SECONDS, TimerMode::Once));
                } else {
                    closing = true;
                }
            }
            Some(Entry::Kind(kind)) => {
                generate.write(GenerateKind(kind));
                closing = true;
            }
            None => {}
        }
    }
    if back {
        if menu.submenu.is_some() {
            menu.submenu = None;
        } else {
            closing = true;
        }
    }
    if closing {
        menu.set_open(false);
    }

    // The rest is the menu's, so nothing behind it moves.
    keys.reset_all();
    mouse.reset(MouseButton::Right);
    scroll.delta = Vec2::ZERO;
    *gesture = TouchGesture::default();
}

/// What a grid was last built from.
#[derive(PartialEq)]
pub(super) struct GridDrawn {
    entries: Vec<(Entry, ButtonState, Option<bool>)>,
    rows: usize,
    first_column: usize,
    fit: usize,
}

/// Shows or hides the menu's layers, and builds a grid again when what it says
/// changes.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn show_menu(
    mut commands: Commands,
    menu: Res<DebugMenu>,
    window: Single<&Window, With<PrimaryWindow>>,
    sources: FactSources,
    mut roots: Query<(&MenuRoot, &mut Visibility)>,
    mut warning: Single<
        &mut Visibility,
        (With<WarningButton>, Without<MenuRoot>, Without<PageButton>),
    >,
    grids: Query<(Entity, &MenuGrid)>,
    mut filter_boxes: Query<(&FilterBox, &mut TextBox)>,
    mut pages: Query<
        (&PageButton, &mut Visibility, &mut ButtonState),
        (Without<MenuRoot>, Without<WarningButton>),
    >,
    mut drawn: Local<[Option<GridDrawn>; 2]>,
) {
    warning.set_if_neq(if *sources.view.get() == GameView::Connecting {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
    for (root, mut visibility) in &mut roots {
        let shows = menu.open && menu.level(root.0).is_some();
        visibility.set_if_neq(if shows {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    if !menu.open {
        return;
    }
    let facts = sources.facts();
    for (filter_box, mut text_box) in &mut filter_boxes {
        let filter = menu.level(filter_box.0).map_or("", |level| &level.filter);
        if text_box.text != filter {
            text_box.text = filter.to_owned();
        }
    }
    for (grid, &MenuGrid(layer)) in &grids {
        let Some(level) = menu.level(layer) else {
            continue;
        };
        let rows = rows_in(window.height(), layer);
        let fit = columns_in(window.width(), layer);
        let entries: Vec<_> = entries_on_show(layer, &level.filter)
            .into_iter()
            .enumerate()
            .map(|(at, entry)| {
                let highlighted = level.highlight == Some(at);
                let available = match entry {
                    Entry::Action(action) => action.is_available(&facts),
                    Entry::Kind(_) => true,
                };
                let state = match (available, highlighted) {
                    (true, true) => ButtonState::Selected,
                    (true, false) => ButtonState::Normal,
                    (false, true) => ButtonState::DisabledSelected,
                    (false, false) => ButtonState::Disabled,
                };
                let on = match entry {
                    Entry::Action(action) => action.is_on(&facts),
                    Entry::Kind(_) => None,
                };
                (entry, state, on)
            })
            .collect();
        let columns = columns_of(entries.len(), rows);
        for (page, mut visibility, mut state) in &mut pages {
            if page.layer != layer {
                continue;
            }
            visibility.set_if_neq(if columns > fit {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
            let can_go = if page.forward {
                level.first_column + fit < columns
            } else {
                level.first_column > 0
            };
            state.set_if_neq(if can_go {
                ButtonState::Normal
            } else {
                ButtonState::Disabled
            });
        }
        let now = GridDrawn {
            entries,
            rows,
            first_column: level.first_column,
            fit,
        };
        if drawn[layer].as_ref() == Some(&now) {
            continue;
        }
        commands.entity(grid).despawn_children();
        commands.entity(grid).with_children(|grid| {
            let on_show = now.entries.chunks(now.rows);
            for column in on_show.skip(now.first_column).take(now.fit) {
                grid.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(GAP),
                    width: px(COLUMN_WIDTH),
                    ..default()
                })
                .with_children(|column_node| {
                    for &(entry, state, on) in column {
                        let node = Node {
                            height: px(ROW),
                            ..default()
                        };
                        let label = match entry {
                            Entry::Action(action) => action.label(),
                            Entry::Kind(kind) => kind_name(kind),
                        };
                        spawn_check_button(column_node, state, on, label, FONT, node)
                            .insert((MenuEntry(entry), Pressable));
                    }
                });
            }
        });
        drawn[layer] = Some(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> Facts {
        Facts {
            view: GameView::WorldMap,
            has_stairs: false,
            has_fields: false,
            has_herbs: false,
            walking: false,
            cycling: false,
            teleporting: false,
            changing_cells: false,
            showing_info: false,
            in_town: false,
            world_time_open: false,
            suns_and_moons_open: false,
            dev_tools: true,
        }
    }

    #[test]
    fn return_with_nothing_highlighted_skips_what_is_disabled() {
        // "state" finds Fields State and Herbs State, neither possible here.
        let shown = on_show("state");
        assert_eq!(return_chooses(&shown, None, &facts()), None);
        let shown = on_show("s");
        assert_eq!(
            return_chooses(&shown, None, &facts()),
            Some(DebugAction::ForgetMapOnServer),
            "{shown:?}"
        );
    }

    #[test]
    fn return_on_a_disabled_entry_does_nothing() {
        let shown = on_show("stairs");
        assert_eq!(return_chooses(&shown, Some(0), &facts()), None);
    }

    #[test]
    fn return_on_a_highlighted_entry_chooses_it() {
        let shown = on_show("");
        assert_eq!(
            return_chooses(&shown, Some(4), &facts()),
            Some(DebugAction::ForgetMap)
        );
    }

    #[test]
    fn the_columns_on_show_follow_the_highlight_and_stop_at_the_end() {
        // 40 entries in 10 rows, 3 columns fitting.
        assert_eq!(first_column(0, Some(5), 40, 10, 3), 0);
        assert_eq!(first_column(0, Some(35), 40, 10, 3), 1);
        assert_eq!(first_column(1, Some(3), 40, 10, 3), 0);
        assert_eq!(first_column(9, None, 40, 10, 3), 1);
        assert_eq!(first_column(9, None, 5, 10, 3), 0);
    }

    #[test]
    fn every_kind_is_on_the_submenu_until_filtered() {
        assert_eq!(kinds_on_show("").len(), PoiKind::ALL.len());
        assert_eq!(kinds_on_show("stone tower"), [PoiKind::StoneTower]);
    }

    #[test]
    fn columns_hold_what_fits() {
        assert_eq!(rows_in(720.0, 0), 13);
        assert_eq!(rows_in(10.0, 0), 1);
        assert!(rows_in(720.0, 1) < rows_in(720.0, 0));
    }
}
