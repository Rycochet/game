//! Opening the panel (the system menu's Accounts), finding its parts once
//! built, and forgetting it once closed or no longer playing.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, RoleAsk};

use super::state::{Panel, Tab, tabs_for};
use crate::common::{AccountRole, OpenAccounts, Session};
use crate::connection::ToServer;
use crate::dom::{Dom, Template, UiEvent, UiValue, document};
use crate::html;
use crate::widgets::{RaisePanel, UiId};

/// The panel: its tabs (those the role may see are put in when it opens) and
/// the chosen tab's page.
pub(super) static ACCOUNTS: Template = html!(
    "accounts_panel.accounts",
    r#"
<form id="accounts">
  <panel id="panel" title="Accounts" moveable left="120" top="100">
    <tab-bar id="tabs">
      <option>Find</option>
      <option>Roles</option>
      <option>Audit Log</option>
    </tab-bar>
    <div id="page" width="520"/>
  </panel>
</form>
"#
);

pub(super) fn open_panel(
    mut open: MessageReader<OpenAccounts>,
    mut commands: Commands,
    mut panel: ResMut<Panel>,
    role: Res<AccountRole>,
    mut raise: MessageWriter<RaisePanel>,
    mut to_server: MessageWriter<ToServer>,
    mut dom: Dom,
) {
    if open.read().count() == 0 {
        return;
    }
    if panel.document.is_some() {
        if let Some(root) = panel.root {
            raise.write(RaisePanel(root));
        }
        return;
    }
    let tabs = tabs_for(&role.powers);
    let document = commands.spawn(document(&ACCOUNTS)).id();
    // Only the tabs the role has something on, and none for a single page.
    if tabs.len() > 1 {
        let label = |tab: &Tab| tab.label().to_owned();
        dom.set_options(
            "accounts/tabs",
            tabs.iter().map(|tab| (label(tab), label(tab))),
        );
    } else {
        dom.set_flag("accounts/tabs", "hidden", true);
    }
    for tab in &tabs {
        let ask = match tab {
            Tab::Roles => RoleAsk::Roles,
            Tab::Audit => RoleAsk::Audit { skip: 0 },
            Tab::Find => continue,
        };
        to_server.write(ToServer(ClientMessage::Role(ask)));
    }
    *panel = Panel {
        document: Some(document),
        tabs,
        dirty: true,
        ..default()
    };
}

/// Notes the built panel (to bring it to the front) and its page (to fill).
pub(super) fn bind_panel(new: Query<(Entity, &UiId), Added<UiId>>, mut panel: ResMut<Panel>) {
    for (entity, id) in &new {
        match id.parts() {
            ("accounts", "panel") => panel.root = Some(entity),
            ("accounts", "page") => panel.body = Some(entity),
            _ => {}
        }
    }
}

/// Forgets the panel once its close button has closed it, and closes it on
/// leaving the game.
pub(super) fn notice_closed(
    mut events: MessageReader<UiEvent>,
    mut commands: Commands,
    mut panel: ResMut<Panel>,
    session: Res<Session>,
) {
    let closed = |event: &UiEvent| {
        event.id == UiId::new("accounts", "panel") && event.value == UiValue::Closed
    };
    let closed = events.read().filter(|event| closed(event)).count() > 0;
    let Some(document) = panel.document else {
        return;
    };
    if closed || !session.is_playing() {
        // The rest of the document goes with its panel.
        commands.entity(document).despawn();
        *panel = Panel::default();
    }
}
