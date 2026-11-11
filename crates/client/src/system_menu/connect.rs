//! Connect, from the startup menu: a box for a server's address, the servers
//! connected to before (click one to put it in the box, or bin it), and, on
//! native, a padlock to paste in the server's certificate so that only it is
//! accepted. Connect's own Connect plays there, once `check.rs` has found a
//! server at the address. See `form.rs` for what the presses do.

use bevy::prelude::*;

use super::{Action, Layer, SystemMenu};
use crate::dom::{Dom, Pictures, Template, document};
use crate::html;
use crate::widgets::{ButtonTooltip, HoverTip, UiId};

// Split into files (`cargo xtask lint` keeps files small).
mod check;
mod form;
mod mark;
mod popups;
mod show;

pub(super) use check::{AddressCheck, check_address};
pub(super) use form::{ConnectForm, fill_server_list, open_connect, use_connect};
pub(super) use mark::show_mark;
pub(super) use popups::{bind_popups, spawn_popups};
pub(super) use show::show_connect;

/// The layer. The padlock and the bin are `picture:padlock` and
/// `picture:bin` (`spawn_connect` names them); each remembered server is a
/// copy of `row`; `mark` is where `mark.rs` shows how the check is going.
static CONNECT: Template = html!(
    "system_menu.connect",
    r#"
<form id="connect">
  <dialog id="layer" title="Connect" modal closable depth="1" width="520" hidden>
    <p class="medium">For example: play.example.com, or play.example.com:24691</p>
    <small id="browser_note" hidden>
      A browser only connects to servers whose certificate is from a certificate authority.
    </small>
    <span nowrap>
      <input id="address" type="text"/>
      <button id="certificate"><img src="picture:padlock"/></button>
    </span>
    <scroll id="servers" max-height="40%"/>
    <footer>
      <canvas id="mark" width="24" height="24"/>
      <button id="connect" class="main">Connect</button>
      <button id="back">Cancel</button>
    </footer>
    <template id="row">
      <span nowrap>
        <button id="server" width="100%"/>
        <button id="forget"><img src="picture:bin"/></button>
      </span>
    </template>
  </dialog>
</form>
"#
);

/// The game's own public server, in the box when no other server is known.
const SUGGESTED_SERVER: &str = "hallofworlds.app";
/// The box the address is typed into.
#[derive(Component)]
pub(super) struct AddressBox;

/// The padlock beside it, shut while a certificate is kept for the address.
#[derive(Component)]
pub(super) struct PadlockButton;

/// Connect's own Connect button, only enabled for an address.
#[derive(Component)]
pub(super) struct ConnectNowButton;

pub(super) fn spawn_connect(
    commands: &mut Commands,
    dom: &mut Dom,
    pictures: &mut Pictures,
    icons: &super::icons::ConnectIcons,
) {
    pictures.name("padlock", icons.open.clone());
    pictures.name("bin", icons.bin.clone());
    commands.spawn(document(&CONNECT));
    // In a browser the browser checks certificates, and says so.
    if cfg!(target_arch = "wasm32") {
        dom.set_flag("connect/certificate", "hidden", true);
        dom.set_flag("connect/browser_note", "hidden", false);
    }
}

/// Gives the built layer and its parts what Connect's systems know them
/// by, found by their ids: each remembered server's row by its place.
pub(super) fn bind_connect(mut commands: Commands, new: Query<(Entity, &UiId), Added<UiId>>) {
    const SHOWN: &[SystemMenu] = &[
        SystemMenu::Connect,
        SystemMenu::Certificate,
        SystemMenu::Forgetting,
    ];
    for (entity, id) in &new {
        let ("connect", name) = id.parts() else {
            continue;
        };
        let place = id.index().unwrap_or(0);
        let mut bound = commands.entity(entity);
        match name {
            "layer" => bound.insert((Layer(SHOWN), Action::Back)),
            "close" | "back" => bound.insert(Action::Back),
            "address" => bound.insert(AddressBox),
            "certificate" => bound.insert((
                PadlockButton,
                Action::Certificate,
                ButtonTooltip("Certificate".into()),
            )),
            "connect" => bound.insert((ConnectNowButton, Action::ConnectNow)),
            "server" => bound.insert(Action::PickServer(place)),
            "forget" => bound.insert((
                Action::ForgetServer(place),
                ButtonTooltip("Forget this server".into()),
            )),
            "mark" => {
                bound.insert((mark::AddressMark, HoverTip(String::new())));
                bound.with_children(mark::spawn_mark_parts)
            }
            _ => continue,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layer_is_good_markup() {
        CONNECT.assert_good();
    }

    #[test]
    fn buttons_keep_the_ids_their_actions_give_them() {
        for (action, name, place) in [
            (Action::Certificate, "certificate", None),
            (Action::ConnectNow, "connect", None),
            (Action::Back, "back", None),
            (Action::PickServer(2), "server", Some(2)),
            (Action::ForgetServer(2), "forget", Some(2)),
        ] {
            let id = action.ui_id("connect");
            assert_eq!((id.parts(), id.index()), (("connect", name), place));
        }
    }
}
