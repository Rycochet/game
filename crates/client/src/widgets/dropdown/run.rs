//! Opening a pressed dropdown, and its keys, typing and choices.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use bevy::window::PrimaryWindow;

use super::super::ButtonPressed;
use super::super::filter_list::{HighlightStep, next_highlight};
use super::super::text_box::{TextBox, TextEdited, TextFocus};
use super::{
    DRAG_PER_ROW, Dropdown, DropdownChosen, ListBacking, ListOption, OpenList, on_show, spawn_list,
    window_start,
};
use crate::common::{TouchGesture, in_interface, interface_area};

/// Opens a pressed dropdown, and while one is open, takes all the input. Before
/// `WidgetSystems::Modal`, so nothing else sees it.
#[allow(clippy::too_many_arguments)]
pub(in crate::widgets) fn run_dropdowns(
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
    ui_scale: Res<UiScale>,
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
            let area = interface_area(&window, &ui_scale);
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
    list.dragged += in_interface(gesture.drag, &ui_scale).y;
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
    if let Some(at) = choice.filter(|&at| !dropdown.is_greyed(at)) {
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

pub(super) fn close(commands: &mut Commands, open: &mut OpenList, focus: &mut TextFocus) {
    if let Some(list) = open.0.take() {
        commands.entity(list.backing).despawn();
        if focus.0 == list.search {
            focus.0 = None;
        }
    }
}
