//! The menu itself: the warning button, the full-screen list, and the keys and
//! presses that work it.

use bevy::asset::RenderAssetUsages;
use bevy::input::ButtonState as KeyState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;

use super::actions::{DebugAction, Facts, Step, next_highlight};
use super::{FactSources, soft_keyboard};
use crate::common::{GameView, TouchGesture};
use crate::widgets::{
    ButtonPressed, ButtonState, INK, Pressable, panel, panel_title, spawn_check_button,
    spawn_close_button, spawn_icon_button, stops_presses,
};

/// How long a toggled entry stays on show before the menu closes.
const TOGGLE_SECONDS: f32 = 1.0;

/// Above every panel, however many times they have been brought to the front.
const MENU_Z: i32 = 1_000_000;

const WARNING_BUTTON: f32 = 34.0;
const CORNER_GAP: f32 = 10.0;

const MARGIN: f32 = 16.0;
const TITLE_ROW: f32 = 36.0;
/// Round the close button, still on it, for fingers.
const CLOSE_MARGIN: f32 = 4.0;
const FILTER_HEIGHT: f32 = 38.0;
const ROW: f32 = 34.0;
const GAP: f32 = 4.0;
const COLUMN_WIDTH: f32 = 300.0;
const FONT: f32 = 18.0;

/// The debug menu: whether it is open, what has been typed and which entry the
/// arrow keys are on, counted among those on show.
#[derive(Resource, Default)]
pub struct DebugMenu {
    pub open: bool,
    pub filter: String,
    highlight: Option<usize>,
    /// Counting down to closing, after a toggle.
    closing: Option<Timer>,
}

impl DebugMenu {
    /// Open, with this already typed (dev tools).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_with(filter: String) -> Self {
        Self {
            open: true,
            filter,
            ..default()
        }
    }

    fn set_open(&mut self, open: bool) {
        *self = Self { open, ..default() };
    }
}

#[derive(Component)]
pub(super) struct WarningButton;

#[derive(Component)]
pub(super) struct MenuRoot;

/// Holds the close button, and takes the presses on it.
#[derive(Component)]
pub(super) struct MenuClose;

#[derive(Component)]
pub(super) struct FilterBox;

#[derive(Component)]
pub(super) struct FilterText;

#[derive(Component)]
pub(super) struct MenuGrid;

#[derive(Component, Clone, Copy)]
pub(super) struct MenuEntry(DebugAction);

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

    let root = commands
        .spawn((panel(), MenuRoot, GlobalZIndex(MENU_Z), Visibility::Hidden))
        .id();
    commands.entity(root).insert((
        stops_presses(root),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(MARGIN)),
            flex_direction: FlexDirection::Column,
            row_gap: px(GAP * 3.0),
            ..default()
        },
    ));
    commands.entity(root).with_children(|root| {
        root.spawn(Node {
            height: px(TITLE_ROW),
            min_height: px(TITLE_ROW),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|bar| {
            bar.spawn(panel_title("Debug Menu"));
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
        root.spawn((
            FilterBox,
            Node {
                height: px(FILTER_HEIGHT),
                min_height: px(FILTER_HEIGHT),
                border: UiRect::all(px(2)),
                padding: UiRect::horizontal(px(GAP * 2.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::from(INK),
        ))
        .with_child((
            FilterText,
            Text::new(""),
            TextFont::from_font_size(FONT),
            TextColor(INK),
        ));
        root.spawn((
            MenuGrid,
            Node {
                column_gap: px(GAP * 2.0),
                ..default()
            },
        ));
    });
}

/// How many entries fit in a column in a window this tall.
fn rows_in(window_height: f32) -> usize {
    let room = window_height - 2.0 * MARGIN - TITLE_ROW - FILTER_HEIGHT - 2.0 * GAP * 3.0;
    (((room + GAP) / (ROW + GAP)).floor() as usize).max(1)
}

/// The actions on show, in order.
fn on_show(filter: &str) -> Vec<DebugAction> {
    DebugAction::ALL
        .into_iter()
        .filter(|action| action.matches(filter))
        .collect()
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

/// The menu gets input after the widgets have looked for presses on it, and
/// before the maps and the game: while it is open, it has all of it.
#[allow(clippy::too_many_arguments)]
pub(super) fn take_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    mut gesture: ResMut<TouchGesture>,
    mut typed: MessageReader<KeyboardInput>,
    mut pressed: MessageReader<ButtonPressed>,
    entries: Query<&MenuEntry>,
    warning: Query<(), With<WarningButton>>,
    close: Query<(), With<MenuClose>>,
    window: Single<&Window, With<PrimaryWindow>>,
    sources: FactSources,
    // Real time: game time is capped per frame, so it runs slow when unfocused.
    time: Res<Time<Real>>,
    mut menu: ResMut<DebugMenu>,
    mut actions: MessageWriter<DebugAction>,
) {
    let presses: Vec<Entity> = pressed.read().map(|press| press.0).collect();
    let toggled = keys.just_pressed(KeyCode::Backquote)
        || presses.iter().any(|&entity| warning.contains(entity));
    let typed: Vec<KeyboardInput> = typed.read().cloned().collect();
    if toggled {
        let open = !menu.open;
        menu.set_open(open);
        soft_keyboard::show_text("");
    }
    if !menu.open {
        return;
    }

    let mut closing = presses.iter().any(|&entity| close.contains(entity));
    let mut choose_first = false;
    for key in soft_keyboard::take_keys() {
        match key {
            soft_keyboard::Key::Return => choose_first = true,
            soft_keyboard::Key::Escape => closing = true,
        }
    }
    if let Some(timer) = &mut menu.closing {
        closing |= timer.tick(time.delta()).is_finished();
    } else if !toggled {
        let facts = sources.facts();
        let before = menu.filter.clone();
        for key in typed.iter().filter(|key| key.state == KeyState::Pressed) {
            match &key.logical_key {
                _ if key.key_code == KeyCode::Backquote => {}
                Key::Backspace => {
                    menu.filter.pop();
                }
                Key::Space => menu.filter.push(' '),
                Key::Character(text) => menu.filter.push_str(text),
                _ => {}
            }
        }
        if menu.filter != before {
            soft_keyboard::show_text(&menu.filter);
        }
        if let Some(text) = soft_keyboard::typed(&menu.filter) {
            menu.filter = text;
        }
        if menu.filter != before {
            menu.highlight = None;
        }

        let shown = on_show(&menu.filter);
        let rows = rows_in(window.height());
        for (key, step) in [
            (KeyCode::ArrowUp, Step::Up),
            (KeyCode::ArrowDown, Step::Down),
            (KeyCode::ArrowLeft, Step::Left),
            (KeyCode::ArrowRight, Step::Right),
        ] {
            if keys.just_pressed(key) {
                menu.highlight = next_highlight(menu.highlight, shown.len(), rows, step);
            }
        }
        let mut chosen = None;
        if keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::NumpadEnter)
            || choose_first
        {
            chosen = return_chooses(&shown, menu.highlight, &facts);
        }
        for &press in &presses {
            if let Ok(entry) = entries.get(press) {
                chosen = Some(entry.0);
            }
        }
        closing |= keys.just_pressed(KeyCode::Escape);
        if let Some(action) = chosen {
            actions.write(action);
            if action.is_on(&facts).is_some() {
                menu.closing = Some(Timer::from_seconds(TOGGLE_SECONDS, TimerMode::Once));
            } else {
                closing = true;
            }
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        closing = true;
    }
    if closing {
        menu.set_open(false);
        soft_keyboard::show_text("");
    }

    // The rest is the menu's, so nothing behind it moves.
    keys.reset_all();
    mouse.reset(MouseButton::Right);
    scroll.delta = Vec2::ZERO;
    *gesture = TouchGesture::default();
}

/// What the grid was last built from.
#[derive(PartialEq)]
pub(super) struct GridDrawn {
    entries: Vec<(DebugAction, ButtonState, Option<bool>)>,
    rows: usize,
}

/// Shows or hides the menu, and builds the grid again when what it says changes.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_menu(
    mut commands: Commands,
    menu: Res<DebugMenu>,
    window: Single<&Window, With<PrimaryWindow>>,
    sources: FactSources,
    root: Single<&mut Visibility, With<MenuRoot>>,
    mut warning: Single<&mut Visibility, (With<WarningButton>, Without<MenuRoot>)>,
    grid: Single<Entity, With<MenuGrid>>,
    filter_box: Single<(&ComputedNode, &UiGlobalTransform), With<FilterBox>>,
    mut filter_text: Single<&mut Text, With<FilterText>>,
    mut drawn: Local<Option<GridDrawn>>,
) {
    warning.set_if_neq(if *sources.view.get() == GameView::Connecting {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
    let mut root = root.into_inner();
    root.set_if_neq(if menu.open {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    if !menu.open {
        soft_keyboard::set_box(None);
        return;
    }
    let (node, transform) = *filter_box;
    let scale = node.inverse_scale_factor;
    let size = node.size() * scale;
    let centre = transform.translation * scale;
    soft_keyboard::set_box(Some(Rect::from_center_size(centre, size)));

    let wanted = format!("Filter: {}_", menu.filter);
    if filter_text.0 != wanted {
        filter_text.0 = wanted;
    }

    let facts = sources.facts();
    let rows = rows_in(window.height());
    let entries = on_show(&menu.filter)
        .into_iter()
        .enumerate()
        .map(|(at, action)| {
            let highlighted = menu.highlight == Some(at);
            let state = match (action.is_available(&facts), highlighted) {
                (true, true) => ButtonState::Selected,
                (true, false) => ButtonState::Normal,
                (false, true) => ButtonState::DisabledSelected,
                (false, false) => ButtonState::Disabled,
            };
            (action, state, action.is_on(&facts))
        })
        .collect();
    let now = GridDrawn { entries, rows };
    if drawn.as_ref() == Some(&now) {
        return;
    }
    commands.entity(*grid).despawn_children();
    commands.entity(*grid).with_children(|grid| {
        for column in now.entries.chunks(now.rows) {
            grid.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(GAP),
                width: px(COLUMN_WIDTH),
                ..default()
            })
            .with_children(|column_node| {
                for &(action, state, on) in column {
                    let node = Node {
                        height: px(ROW),
                        ..default()
                    };
                    spawn_check_button(column_node, state, on, action.name(), FONT, node)
                        .insert((MenuEntry(action), Pressable));
                }
            });
        }
    });
    *drawn = Some(now);
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
    fn columns_hold_what_fits() {
        assert_eq!(rows_in(720.0), 15);
        assert_eq!(rows_in(10.0), 1);
    }
}
