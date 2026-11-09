//! The Audit Log page: the server's record of roles given, taken, run out and
//! renamed, newest first, a page at a time.

use bevy::prelude::*;

use super::FONT;
use super::fill::{control, words};
use super::state::{FORM, Panel};
use crate::widgets::{
    ButtonState, INK, Pressable, TextSize, UiId, spawn_button, spawn_scroll_area,
};

/// How many lines one page of the log is (`server`'s `AUDIT_PAGE`).
pub(super) const PAGE: u32 = 50;

#[derive(Component)]
pub(super) struct OlderButton;

#[derive(Component)]
pub(super) struct NewerButton;

pub(super) fn spawn_audit(page: &mut ChildSpawnerCommands, panel: &Panel) {
    if panel.audit.is_empty() {
        page.spawn(words("Nothing has been written down yet."));
    } else {
        let area = spawn_scroll_area(page, px(360));
        page.commands()
            .entity(area)
            .insert(UiId::new(FORM, "audit"))
            .with_children(|lines| {
                for line in &panel.audit {
                    lines.spawn((
                        Text::new(line.clone()),
                        TextSize::Small.font(),
                        TextColor(INK),
                    ));
                }
            });
    }
    let row = Node {
        column_gap: px(8),
        ..default()
    };
    page.spawn(row).with_children(|row| {
        let state = |on: bool| {
            if on {
                ButtonState::Normal
            } else {
                ButtonState::Disabled
            }
        };
        spawn_button(
            row,
            state(panel.audit_skip > 0),
            "Newer",
            FONT,
            control(110.0),
        )
        .insert((NewerButton, Pressable, UiId::new(FORM, "newer")));
        spawn_button(row, state(panel.audit_more), "Older", FONT, control(110.0)).insert((
            OlderButton,
            Pressable,
            UiId::new(FORM, "older"),
        ));
    });
}
