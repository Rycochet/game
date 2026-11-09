//! The menu's keys, typing and presses.

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::FactSources;
use super::super::actions::DebugAction;
use super::super::panels::GenerateKind;
use super::{
    ACTIONS, DebugMenu, Entry, FilterBox, MenuClose, MenuEntry, MenuRoot, PageButton,
    TOGGLE_SECONDS, WarningButton, columns_in, entries_on_show, first_column, headings_of, lay_out,
    layout, on_show, return_chooses, rows_in,
};
use crate::common::{Action, Preferences, TouchGesture, interface_area};
use crate::minimap::FullMap;
use crate::widgets::{
    ButtonPressed, HighlightStep, SoftKey, SoftKeyPressed, TextBox, TextEdited, TextFocus,
};

/// Presses on these close the top layer: its close button, and outside it.
pub(super) type ClosesLayer = Or<(With<MenuClose>, With<MenuRoot>)>;

/// What the widgets say has happened.
#[derive(SystemParam)]
pub(in crate::debug_menu) struct MenuReaders<'w, 's> {
    pressed: MessageReader<'w, 's, ButtonPressed>,
    edited: MessageReader<'w, 's, TextEdited>,
    soft_keys: MessageReader<'w, 's, SoftKeyPressed>,
}

/// The parts of the menu that take input.
#[derive(SystemParam)]
pub(in crate::debug_menu) struct MenuNodes<'w, 's> {
    filter_boxes: Query<'w, 's, (Entity, &'static FilterBox, &'static TextBox)>,
    entries: Query<'w, 's, &'static MenuEntry>,
    pages: Query<'w, 's, &'static PageButton>,
    warning: Query<'w, 's, (), With<WarningButton>>,
    close: Query<'w, 's, (), ClosesLayer>,
}

/// The menu gets input after the widgets have looked for presses on it, and
/// before the maps and the game: while it is open, it has all of it.
#[allow(clippy::too_many_arguments)]
pub(in crate::debug_menu) fn take_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    mut gesture: ResMut<TouchGesture>,
    readers: MenuReaders,
    nodes: MenuNodes,
    mut focus: ResMut<TextFocus>,
    (full_map, preferences): (Res<FullMap>, Res<Preferences>),
    (window, ui_scale): (Single<&Window, With<PrimaryWindow>>, Res<UiScale>),
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
        && (preferences.controls.just_pressed(&keys, Action::DebugMenu)
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
        let room = interface_area(&window, &ui_scale);
        let (rows, fit) = (rows_in(room.y, layer), columns_in(room.x, layer));
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
        let columns = lay_out(&headings_of(&shown), rows);
        for (key, step) in [
            (KeyCode::ArrowUp, HighlightStep::Up),
            (KeyCode::ArrowDown, HighlightStep::Down),
            (KeyCode::ArrowLeft, HighlightStep::Left),
            (KeyCode::ArrowRight, HighlightStep::Right),
        ] {
            // Until Up or Down starts a highlight, Left and Right move the
            // filter's caret instead.
            let sideways = matches!(step, HighlightStep::Left | HighlightStep::Right);
            if keys.just_pressed(key) && !(sideways && level.highlight.is_none()) {
                level.highlight = layout::step(&columns, level.highlight, shown.len(), step);
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
        level.first_column = first_column(&columns, level.first_column, level.highlight, fit);

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
            Some(Entry::Action(DebugAction::GeneratePoi)) => {
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
