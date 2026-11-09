//! Building the panel's page again when what it shows changes, and the pieces
//! each page is made of.

use bevy::prelude::*;

use super::state::{Panel, Tab};
use super::{FONT, audit, find, roles};
use crate::widgets::{INK, WARNING_TEXT};

/// How wide the labels at the start of rows are.
const LABEL: f32 = 110.0;

/// The height of a row's controls.
pub(super) const ROW: f32 = 34.0;

pub(super) fn fill_page(mut commands: Commands, mut panel: ResMut<Panel>) {
    let Some(body) = panel.body.filter(|_| panel.dirty) else {
        return;
    };
    panel.dirty = false;
    commands.entity(body).despawn_related::<Children>();
    commands.entity(body).with_children(|page| {
        match panel.tab() {
            Some(Tab::Find) => find::spawn_find(page, &panel),
            Some(Tab::Roles) => roles::spawn_roles(page, &panel),
            Some(Tab::Audit) => audit::spawn_audit(page, &panel),
            None => {
                page.spawn(words("Your role has nothing here."));
            }
        }
        if let Some(why) = &panel.refused {
            page.spawn((
                Text::new(capitalised(why)),
                FONT.font(),
                TextColor(WARNING_TEXT),
            ));
        }
    });
}

/// Plain writing on the page.
pub(super) fn words(text: impl Into<String>) -> impl Bundle {
    (Text::new(text), FONT.font(), TextColor(INK))
}

/// A row with a label at the start, for the controls `fill` puts after it.
pub(super) fn labelled(
    page: &mut ChildSpawnerCommands,
    label: &str,
    fill: impl FnOnce(&mut ChildSpawnerCommands),
) {
    let row = Node {
        align_items: AlignItems::Center,
        column_gap: px(8),
        ..default()
    };
    page.spawn(row).with_children(|row| {
        row.spawn((
            words(label),
            Node {
                width: px(LABEL),
                ..default()
            },
        ));
        fill(row);
    });
}

/// A control's size in a row: `width` across.
pub(super) fn control(width: f32) -> Node {
    Node {
        width: px(width),
        height: px(ROW),
        ..default()
    }
}

/// The server's reasons start in lower case, as they follow other words.
fn capitalised(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}
