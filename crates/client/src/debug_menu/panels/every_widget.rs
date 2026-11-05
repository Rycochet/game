//! Show Widgets: one of each thing `widgets` makes.

use bevy::prelude::*;
use shared::poi::PoiKind;

use super::super::actions::kind_name;
use super::{CHECK_ROW, EVERY_WIDGET_HEIGHT, FONT};
use crate::widgets::{
    ALLOWED, BUTTON_PADDING, ButtonState, ButtonStyle, Dropdown, GRID_COLOUR, INK, Pressable,
    TextBox, UiId, field_label, raised_box, spawn_button, spawn_check_button, spawn_close_button,
    spawn_dropdown, spawn_progress_bar, spawn_scroll_area, spawn_slider, spawn_tabs,
    spawn_text_box, spawn_toggle_row, spawn_tooltip,
};

/// One of each thing `widgets` makes, to see how they look together.
pub(super) fn spawn_every_widget(content: &mut ChildSpawnerCommands) {
    let area = spawn_scroll_area(content, Val::Vh(EVERY_WIDGET_HEIGHT));
    let key = UiId::new("debug.every_widget", "scroll");
    content.commands().entity(area).insert(key);
    content.commands().entity(area).with_children(|area| {
        area.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        })
        .with_children(spawn_widgets);
    });
}

pub(super) fn spawn_widgets(content: &mut ChildSpawnerCommands) {
    for (state, label) in [
        (ButtonState::Normal, "Normal"),
        (ButtonState::Selected, "Selected"),
        (ButtonState::Active, "Active"),
        (ButtonState::Disabled, "Disabled"),
        (ButtonState::DisabledSelected, "Disabled, selected"),
    ] {
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, state, label, FONT, node).insert(Pressable);
    }
    for (style, label) in [(ButtonStyle::Main, "Main"), (ButtonStyle::Quiet, "Quiet")] {
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, ButtonState::Normal, label, FONT, node).insert((style, Pressable));
    }
    for (on, label) in [
        (Some(true), "Check on"),
        (Some(false), "Check off"),
        (None, "No check"),
    ] {
        let node = Node {
            height: px(CHECK_ROW),
            ..default()
        };
        spawn_check_button(content, ButtonState::Normal, on, label, FONT, node).insert(Pressable);
    }
    spawn_tabs(content, &["Tab", "Other tab", "Third tab"], 0, FONT);
    spawn_toggle_row(content, &["Toggle", "Other", "Third"], 0, FONT);
    spawn_progress_bar(
        content,
        0.6,
        (GRID_COLOUR, ALLOWED),
        Node {
            height: px(CHECK_ROW),
            ..default()
        },
        (Text::new("Progress"), FONT.font(), TextColor(INK)),
    );
    spawn_close_button(content, default());
    spawn_slider(content, 0.5, None, 160.0);
    let kinds = || PoiKind::ALL.map(kind_name).to_vec();
    for searchable in [false, true] {
        let dropdown = Dropdown {
            options: kinds(),
            selected: 0,
            searchable,
            ..default()
        };
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_dropdown(content, dropdown, FONT, node);
    }
    let text_box = TextBox {
        text: "Text box".into(),
        takes_keys: true,
        ..default()
    };
    let node = Node {
        height: px(CHECK_ROW),
        ..default()
    };
    content.spawn(field_label("Label"));
    spawn_text_box(content, text_box, FONT, node.clone());
    let wrong = TextBox {
        text: "Something wrong".into(),
        takes_keys: true,
        error: true,
        ..default()
    };
    spawn_text_box(content, wrong, FONT, node);
    content.spawn(raised_box()).with_child((
        Text::new("Raised box"),
        FONT.font(),
        TextColor(crate::widgets::INK),
    ));
    content
        .spawn(Node::default())
        .with_children(|holder| spawn_tooltip(holder, SampleTooltip, "Tooltip"));
}

/// The sample tooltip's text.
#[derive(Component)]
pub(in crate::debug_menu) struct SampleTooltip;
