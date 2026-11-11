//! Connect's popups, as markup: the box to paste a server's certificate
//! into, and the requester asking whether to forget a server. Once built,
//! each part is given what Connect's systems know it by, found by its id.

use bevy::prelude::*;

use super::super::{Action, Layer, SystemMenu};
use crate::dom::{Template, document};
use crate::html;
use crate::widgets::UiId;

static POPUPS: Template = html!(
    "system_menu.connect_popups",
    r#"
<form id="connect.certificate">
  <dialog id="popup" title="Certificate" depth="2" hidden>
    <div width="480">
      <p class="medium">
        Paste in the server's certificate, from its Server Options or its
        /certificate page, or its key's fingerprint from its log. Only a server
        with this certificate will be accepted. Without one, any certificate is.
      </p>
      <input id="certificate" type="text"/>
      <small id="status"/>
    </div>
    <footer>
      <button id="keep" class="main">Keep</button>
      <button id="clear">Clear</button>
      <button id="back">Cancel</button>
    </footer>
  </dialog>
</form>
<form id="connect.forget">
  <dialog id="popup" title="Forget server" depth="2" hidden>
    <p id="words"/>
    <footer>
      <button id="forget" class="main">Yes, forget it</button>
      <button id="back">No</button>
    </footer>
  </dialog>
</form>
"#
);

/// The box the certificate is pasted into.
#[derive(Component)]
pub(in crate::system_menu) struct CertificateBox;

/// The certificate popup's Keep, only enabled for a certificate.
#[derive(Component)]
pub(in crate::system_menu) struct KeepButton;

pub(in crate::system_menu) fn spawn_popups(commands: &mut Commands) {
    commands.spawn(document(&POPUPS));
}

/// What each popup's button does, by its form and name.
fn action_of(form: &str, name: &str) -> Action {
    match (form, name) {
        ("connect.certificate", "keep") => Action::KeepCertificate,
        ("connect.certificate", "clear") => Action::ClearCertificate,
        ("connect.forget", "forget") => Action::ReallyForget,
        _ => Action::Back,
    }
}

/// Gives the built popups and their parts what Connect knows them by. A
/// press outside a popup goes back, as its Cancel or No does.
pub(in crate::system_menu) fn bind_popups(
    mut commands: Commands,
    new: Query<(Entity, &UiId), Added<UiId>>,
) {
    for (entity, id) in &new {
        let (form, name) = id.parts();
        let layer = match form {
            "connect.certificate" => Layer(&[SystemMenu::Certificate]),
            "connect.forget" => Layer(&[SystemMenu::Forgetting]),
            _ => continue,
        };
        let mut bound = commands.entity(entity);
        match name {
            "popup" => bound.insert((layer, Action::Back)),
            "certificate" => bound.insert(CertificateBox),
            "keep" => bound.insert((action_of(form, name), KeepButton)),
            // Words, set by id.
            "status" | "words" => continue,
            _ => bound.insert(action_of(form, name)),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_popups_are_good_markup() {
        POPUPS.assert_good();
    }

    #[test]
    fn buttons_keep_the_ids_their_actions_give_them() {
        for (form, action) in [
            ("connect.certificate", Action::KeepCertificate),
            ("connect.certificate", Action::ClearCertificate),
            ("connect.certificate", Action::Back),
            ("connect.forget", Action::ReallyForget),
            ("connect.forget", Action::Back),
        ] {
            let (_, name) = action.ui_id(form).parts();
            assert_eq!(action_of(form, name), action, "{form}/{name}");
        }
    }
}
