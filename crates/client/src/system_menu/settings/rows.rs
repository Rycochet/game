//! What Settings' pages are made of: sections under headings, and rows of a
//! label and a control, each naming what it does when the pointer stays on
//! it (`widgets::HoverTip`), whether or not it can be used.

use bevy::prelude::*;

use super::{FONT, LABEL_GAP, ROW_GAP, ROW_HEIGHT};
use crate::widgets::{Dropdown, HoverTip, INK, spawn_dropdown};

const LABEL_WIDTH: f32 = 180.0;
const SECTION_GAP: f32 = 40.0;
const HEADING_LINE: f32 = 2.0;

/// Sections side by side, going onto more lines when the page is narrow.
pub(super) fn spawn_sections(
    page: &mut ChildSpawnerCommands,
    fill: impl FnOnce(&mut ChildSpawnerCommands),
) {
    page.spawn(Node {
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(SECTION_GAP),
        row_gap: px(SECTION_GAP),
        align_items: AlignItems::FlexStart,
        ..default()
    })
    .with_children(fill);
}

/// A column of rows under a heading.
pub(super) fn spawn_section(
    sections: &mut ChildSpawnerCommands,
    heading: &str,
    fill: impl FnOnce(&mut ChildSpawnerCommands),
) {
    sections
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(ROW_GAP),
            ..default()
        })
        .with_children(|section| {
            section.spawn((
                Text::new(heading),
                FONT.font(),
                TextColor(INK),
                Node {
                    border: UiRect::bottom(px(HEADING_LINE)),
                    ..default()
                },
                BorderColor::from(INK),
            ));
            fill(section);
        });
}

/// What a setting does, and why it can't be used, if it can't.
pub(super) fn tip(what: &str, why_not: Option<&str>) -> HoverTip {
    HoverTip(match why_not {
        Some(why) => format!("{what}\n\n{why}"),
        None => what.into(),
    })
}

/// A row: the label, then what `control` spawns, with `tip` over all of it.
pub(super) fn spawn_labelled(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    tip: HoverTip,
    control: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            tip,
            Node {
                column_gap: px(LABEL_GAP),
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                FONT.font(),
                TextColor(INK),
                Node {
                    width: px(LABEL_WIDTH),
                    ..default()
                },
            ));
            control(row);
        });
}

/// A labelled dropdown `width` across; returns the dropdown.
pub(super) fn spawn_dropdown_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    tip: HoverTip,
    dropdown: Dropdown,
    width: f32,
) -> Entity {
    let mut made = Entity::PLACEHOLDER;
    spawn_labelled(parent, label, tip, |row| {
        let node = Node {
            width: px(width),
            height: px(ROW_HEIGHT),
            ..default()
        };
        made = spawn_dropdown(row, dropdown, FONT, node).id();
    });
    made
}
