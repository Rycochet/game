//! Show Widgets: one of each thing `widgets` makes.

use bevy::prelude::*;
use shared::poi::PoiKind;

use super::super::actions::kind_name;
use super::{CHECK_ROW, EVERY_WIDGET_HEIGHT, FONT};
use crate::common::{Binding, Mods};
use crate::widgets::{
    ALLOWED, BUTTON_PADDING, ButtonState, ButtonStyle, ColourPicker, Dropdown, GRID_COLOUR, INK,
    KeyBox, Pressable, SAMPLE_GRADIENT, Swatch, TextBox, UiId, field_label, raised_box,
    spawn_button, spawn_check_button, spawn_close_button, spawn_colour_picker, spawn_dropdown,
    spawn_key_box, spawn_progress_bar, spawn_scroll_area, spawn_slider, spawn_switch, spawn_tabs,
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

/// A sample's id: its own words, as there is one of each.
fn id(name: &'static str) -> UiId {
    UiId::new("debug.every_widget", name)
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
        spawn_button(content, state, label, FONT, node).insert((Pressable, id(label)));
    }
    for (style, label) in [(ButtonStyle::Main, "Main"), (ButtonStyle::Quiet, "Quiet")] {
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, ButtonState::Normal, label, FONT, node).insert((
            style,
            Pressable,
            id(label),
        ));
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
        spawn_check_button(content, ButtonState::Normal, on, label, FONT, node)
            .insert((Pressable, id(label)));
    }
    content
        .spawn(Node {
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            for (state, on, label) in [
                (ButtonState::Normal, true, "Switch on"),
                (ButtonState::Normal, false, "Switch off"),
                (ButtonState::Disabled, false, "Switch disabled"),
                (ButtonState::DisabledSelected, true, "Switch disabled on"),
            ] {
                spawn_switch(row, state, on, default()).insert(id(label));
            }
        });
    let tabs = spawn_tabs(content, &["Tab", "Other tab", "Third tab"], 0, FONT);
    content.commands().entity(tabs).insert(id("Tabs"));
    let toggles = spawn_toggle_row(content, &["Toggle", "Other", "Third"], 0, FONT);
    content.commands().entity(toggles).insert(id("Toggles"));
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
    spawn_close_button(content, default()).insert(id("Close"));
    spawn_slider(content, 0.5, None, 160.0).insert(id("Slider"));
    let colours = [
        ("Red", [190, 60, 50]),
        ("Green", [70, 150, 70]),
        ("Blue", [60, 90, 180]),
    ];
    let mut picker = ColourPicker {
        swatches: colours
            .iter()
            .map(|&(_, [r, g, b])| Swatch::Colour(Color::srgb_u8(r, g, b)))
            .collect(),
        names: colours.iter().map(|&(name, _)| name.to_owned()).collect(),
        selected: 0,
    };
    picker.swatches.push(Swatch::Picture(SAMPLE_GRADIENT));
    picker.names.push("Gradient".into());
    let node = Node {
        width: px(200),
        ..default()
    };
    spawn_colour_picker(content, picker, FONT, node).insert(id("Colour picker"));
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
        let name = if searchable { "Search" } else { "Dropdown" };
        spawn_dropdown(content, dropdown, FONT, node).insert(id(name));
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
    spawn_text_box(content, text_box, FONT, node.clone()).insert(id("Text box"));
    let wrong = TextBox {
        text: "Something wrong".into(),
        takes_keys: true,
        error: true,
        ..default()
    };
    spawn_text_box(content, wrong, FONT, node.clone()).insert(id("Wrong"));
    let key_box = KeyBox {
        binding: Some(Binding::new(KeyCode::KeyG, Mods::SHIFT)),
        warned: false,
    };
    spawn_key_box(content, key_box, ButtonState::Normal, FONT, node.clone()).insert(id("Key box"));
    spawn_key_box(
        content,
        KeyBox::default(),
        ButtonState::Disabled,
        FONT,
        node,
    )
    .insert(id("Disabled key box"));
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
