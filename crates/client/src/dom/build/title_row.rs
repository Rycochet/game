//! The row along the top of a big dialog: its title in the middle and, if
//! it can be closed, a close button at the right.

use bevy::prelude::*;

use super::Builder;
use crate::widgets::{CLOSE_PIXELS, Pressable, UiId, panel_title, spawn_close_button};

/// How tall the row is.
const ROW: f32 = 36.0;
/// Round the close button, counting as on it, for fingers.
const CLOSE_MARGIN: f32 = 4.0;

impl Builder<'_, '_, '_> {
    /// Spawns the row in `panel`. With `close`, the id of a close button,
    /// there is one: a press on it is a `ButtonPressed` for what this returns.
    pub(super) fn title_row(
        &mut self,
        panel: Entity,
        title: &str,
        close: Option<UiId>,
    ) -> Option<Entity> {
        let row = Node {
            height: px(ROW),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let mut button = None;
        self.commands.entity(panel).with_children(|panel| {
            panel.spawn(row).with_children(|row| {
                row.spawn(panel_title(title));
                let Some(id) = close else {
                    return;
                };
                let holder = Node {
                    position_type: PositionType::Absolute,
                    right: px(-CLOSE_MARGIN),
                    top: px((ROW - CLOSE_PIXELS) / 2.0 - CLOSE_MARGIN),
                    padding: UiRect::all(px(CLOSE_MARGIN)),
                    ..default()
                };
                let mut holder = row.spawn((holder, Pressable, id));
                holder.with_children(|holder| {
                    spawn_close_button(holder, default());
                });
                button = Some(holder.id());
            });
        });
        button
    }
}
