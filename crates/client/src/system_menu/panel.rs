//! The menu's own panel, as markup: "Paused" above three sections of
//! buttons, with a gap between them. Once built, each button is given the
//! `Action` the menu knows it by, found by its id.

use bevy::prelude::*;

use super::online::{OWN_TIP, OnlineToggle};
use super::{Action, Layer, PausedText, SystemMenu};
use crate::dom::{Template, document};
use crate::html;
use crate::widgets::{ButtonTooltip, UiId};

/// The panel. The gear of Server Options is `picture:gear` (`spawn_menu`
/// names it); Online's box is ticked by `online::show_online`, not by being
/// pressed (`readonly`).
static MENU: Template = html!(
    "system_menu.menu",
    r#"
<form id="menu">
  <dialog id="layer" modal depth="0" width="260" hidden>
    <h1 id="paused" align="center" height="30" margin-bottom="24" hidden>Paused</h1>
    <div margin-bottom="24">
      <button id="continue" margin-bottom="8">Continue</button>
      <button id="characters" margin-bottom="8">Characters</button>
      <span id="online_row" class="joined" margin-bottom="8">
        <input id="online" type="checkbox" readonly label="Online"
               width="176" margin-bottom="0"/>
        <button id="server_options" margin-bottom="0">
          <img src="picture:gear"/>
        </button>
      </span>
      <button id="connect" margin-bottom="0">Connect...</button>
    </div>
    <div margin-bottom="24">
      <button id="settings" margin-bottom="8">Settings</button>
      <button id="credits" margin-bottom="0">Credits</button>
    </div>
    <div>
      <button id="log_out" margin-bottom="8">Logout</button>
      <button id="quit" margin-bottom="0">Quit</button>
    </div>
  </dialog>
</form>
"#
);

/// Everything the menu's panel stays behind.
const SHOWN: &[SystemMenu] = &[
    SystemMenu::Menu,
    SystemMenu::Confirming,
    SystemMenu::Settings,
    SystemMenu::KeepGraphics,
    SystemMenu::Ironman,
    SystemMenu::Credits,
    SystemMenu::Connect,
    SystemMenu::Certificate,
    SystemMenu::Forgetting,
    SystemMenu::Refused,
    SystemMenu::ServerOptions,
    SystemMenu::GoingOnline,
    SystemMenu::GoingOffline,
    SystemMenu::OnlineFailed,
    SystemMenu::LockingDown,
    SystemMenu::Reopening,
    SystemMenu::SlowDrawing,
];

pub(super) fn spawn_main_panel(commands: &mut Commands) {
    commands.spawn(document(&MENU));
}

/// What each of the panel's buttons does, by its name.
fn action_of(name: &str) -> Option<Action> {
    Some(match name {
        "continue" => Action::Continue,
        "characters" => Action::Characters,
        "online" => Action::ToggleOnline,
        "server_options" => Action::ServerOptions,
        "connect" => Action::Connect,
        "settings" => Action::Settings,
        "credits" => Action::Credits,
        "log_out" => Action::LogOut,
        "quit" => Action::Quit,
        _ => return None,
    })
}

/// Gives the built panel and its buttons what the menu knows them by.
pub(super) fn bind_main_panel(mut commands: Commands, new: Query<(Entity, &UiId), Added<UiId>>) {
    for (entity, id) in &new {
        let ("menu", name) = id.parts() else {
            continue;
        };
        let mut bound = commands.entity(entity);
        if let Some(action) = action_of(name) {
            bound.insert(action);
        }
        match name {
            "layer" => bound.insert((Layer(SHOWN), Action::Back)),
            "paused" => bound.insert(PausedText),
            "online" => bound.insert((OnlineToggle, ButtonTooltip(OWN_TIP.into()))),
            "server_options" => bound.insert(ButtonTooltip("Server Options".into())),
            _ => &mut bound,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_is_good_markup() {
        MENU.assert_good();
    }

    #[test]
    fn buttons_keep_the_ids_their_actions_give_them() {
        for name in [
            "continue",
            "characters",
            "online",
            "server_options",
            "connect",
            "settings",
            "credits",
            "log_out",
            "quit",
        ] {
            let action = action_of(name).expect("a button");
            assert_eq!(action.ui_id("menu").parts(), ("menu", name));
        }
    }
}
