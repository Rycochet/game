//! Opening the panel (the system menu's Accounts), and forgetting it once
//! closed or no longer playing.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, RoleAsk};

use super::FONT;
use super::state::{FORM, Panel, Tab, tabs_for};
use crate::common::{AccountRole, OpenAccounts, Session};
use crate::connection::ToServer;
use crate::widgets::{RaisePanel, UiId, spawn_moveable_panel, spawn_tabs};

/// Where it opens, in interface pixels from the top left.
const AT: Vec2 = Vec2::new(120.0, 100.0);

/// How wide its pages are.
pub(super) const WIDTH: f32 = 520.0;

pub(super) fn open_panel(
    mut open: MessageReader<OpenAccounts>,
    mut commands: Commands,
    mut panel: ResMut<Panel>,
    role: Res<AccountRole>,
    mut raise: MessageWriter<RaisePanel>,
    mut to_server: MessageWriter<ToServer>,
) {
    if open.read().count() == 0 {
        return;
    }
    if let Some(root) = panel.root {
        raise.write(RaisePanel(root));
        return;
    }
    let tabs = tabs_for(&role.powers);
    let labels: Vec<&str> = tabs.iter().map(|tab| tab.label()).collect();
    let (root, content) = spawn_moveable_panel(&mut commands, "Accounts", AT);
    commands.entity(root).insert(UiId::new(FORM, "panel"));
    let mut row = None;
    let mut body = None;
    commands.entity(content).with_children(|content| {
        if labels.len() > 1 {
            let tabs_row = spawn_tabs(content, &labels, 0, FONT);
            content
                .commands()
                .entity(tabs_row)
                .insert(UiId::new(FORM, "page"));
            row = Some(tabs_row);
        }
        let page = Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            width: px(WIDTH),
            ..default()
        };
        body = Some(content.spawn(page).id());
    });
    for tab in &tabs {
        let ask = match tab {
            Tab::Roles => RoleAsk::Roles,
            Tab::Audit => RoleAsk::Audit { skip: 0 },
            Tab::Find => continue,
        };
        to_server.write(ToServer(ClientMessage::Role(ask)));
    }
    *panel = Panel {
        root: Some(root),
        tabs_row: row,
        body,
        tabs,
        dirty: true,
        ..default()
    };
}

/// Forgets the panel once its close button has gone, and closes it on leaving
/// the game.
pub(super) fn notice_closed(
    mut commands: Commands,
    mut panel: ResMut<Panel>,
    session: Res<Session>,
    exists: Query<()>,
) {
    let Some(root) = panel.root else {
        return;
    };
    if !exists.contains(root) {
        *panel = Panel::default();
    } else if !session.is_playing() {
        commands.entity(root).despawn();
        *panel = Panel::default();
    }
}
