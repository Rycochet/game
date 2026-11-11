//! The menu's requesters, written as markup: Quit, Ironman, Keep these
//! settings, Can't connect, Slow drawing, Online's four questions (Go online,
//! Go offline, Lock down, Open to everyone) and Can't go online. What they do is unchanged: once
//! built, each dialog and button is given the `Layer` and `Action` the rest
//! of the menu knows them by, found by its id.

use bevy::prelude::*;

use super::settings::KeepButton;
use super::{Action, Layer, SystemMenu};
use crate::common::IRONMAN_QUESTION;
use crate::dom::{Dom, Template, document};
use crate::html;
use crate::widgets::UiId;

static REQUESTERS: Template = html!(
    "system_menu.requesters",
    r#"
<form id="quit">
  <dialog id="requester" title="Quit" depth="1" hidden>
    <p>Are you sure?</p>
    <footer>
      <button id="quit" class="main">Yes, quit</button>
      <button id="back">No</button>
    </footer>
  </dialog>
</form>
<form id="ironman">
  <dialog id="requester" title="Ironman" depth="2" hidden>
    <p id="question" width="420"/>
    <footer>
      <button id="ironman" class="main">Yes, Ironman</button>
      <button id="no">No</button>
    </footer>
  </dialog>
</form>
<form id="settings.keep_graphics">
  <dialog id="requester" title="Graphics" depth="2" hidden>
    <p>Keep these settings?</p>
    <p id="countdown"/>
    <footer>
      <button id="keep" class="main">Keep</button>
      <button id="back">Go back</button>
    </footer>
  </dialog>
</form>
<form id="refused">
  <dialog id="requester" title="Can't connect" depth="1" hidden>
    <p id="words" class="warning" width="460"/>
    <footer>
      <button id="back" class="main">OK</button>
    </footer>
  </dialog>
</form>
<form id="slow_drawing">
  <dialog id="requester" title="Slow drawing" depth="1" hidden>
    <p id="words" width="460"/>
    <footer>
      <button id="back" class="main">Play Anyway</button>
      <button id="dont_warn">Don't Warn Again</button>
    </footer>
  </dialog>
</form>
<form id="going_online">
  <dialog id="requester" title="Go online" depth="1" hidden>
    <p width="420">
      This will let other players connect to this game. Are you sure?
    </p>
    <footer>
      <button id="yes" class="main">Yes</button>
      <button id="back">No</button>
    </footer>
  </dialog>
</form>
<form id="going_offline">
  <dialog id="requester" title="Go offline" depth="1" hidden>
    <p width="420">
      Anyone else playing will be told the server is going offline, and disconnected.
      Are you sure?
    </p>
    <footer>
      <button id="yes" class="main">Yes</button>
      <button id="back">No</button>
    </footer>
  </dialog>
</form>
<form id="locking_down">
  <dialog id="requester" title="Lock down" depth="1" hidden>
    <p width="420">
      Only staff who may change Server Options will be able to play here. Everyone
      else playing will be sent away. Are you sure?
    </p>
    <footer>
      <button id="yes" class="main">Yes</button>
      <button id="back">No</button>
    </footer>
  </dialog>
</form>
<form id="reopening">
  <dialog id="requester" title="Open to everyone" depth="1" hidden>
    <p width="420">
      This will let everyone play on this server again. Are you sure?
    </p>
    <footer>
      <button id="yes" class="main">Yes</button>
      <button id="back">No</button>
    </footer>
  </dialog>
</form>
<form id="online_failed">
  <dialog id="requester" title="Can't go online" depth="1" hidden>
    <p id="words" width="420"/>
    <footer>
      <button id="back" class="main">OK</button>
    </footer>
  </dialog>
</form>
"#
);

/// Builds the requesters, with the words that are kept in code.
pub(super) fn spawn_requesters(commands: &mut Commands, dom: &mut Dom, slow_drawing: &str) {
    commands.spawn(document(&REQUESTERS));
    dom.set_text("ironman/question", IRONMAN_QUESTION);
    dom.set_text("slow_drawing/words", slow_drawing);
}

/// When each requester shows, by its form.
const LAYERS: [(&str, Layer); 10] = [
    ("quit", Layer(&[SystemMenu::Confirming])),
    ("ironman", Layer(&[SystemMenu::Ironman])),
    ("settings.keep_graphics", Layer(&[SystemMenu::KeepGraphics])),
    ("refused", Layer(&[SystemMenu::Refused])),
    ("slow_drawing", Layer(&[SystemMenu::SlowDrawing])),
    ("going_online", Layer(&[SystemMenu::GoingOnline])),
    ("going_offline", Layer(&[SystemMenu::GoingOffline])),
    ("locking_down", Layer(&[SystemMenu::LockingDown])),
    ("reopening", Layer(&[SystemMenu::Reopening])),
    ("online_failed", Layer(&[SystemMenu::OnlineFailed])),
];

/// What each button does, by its form and name. Any other goes back.
fn action_of(form: &str, name: &str) -> Action {
    match (form, name) {
        ("quit", "quit") => Action::ReallyQuit,
        ("ironman", "ironman") => Action::ChooseIronman,
        ("settings.keep_graphics", "keep") => Action::KeepGraphics,
        ("slow_drawing", "dont_warn") => Action::QuietSlowDrawing,
        // Online's four questions share their Yes.
        (_, "yes") => Action::ReallyToggleOnline,
        _ => Action::Back,
    }
}

/// Gives each newly built requester and its buttons what the menu knows
/// them by. A press outside a requester goes back, as its No does.
pub(super) fn bind_requesters(mut commands: Commands, new: Query<(Entity, &UiId), Added<UiId>>) {
    for (entity, id) in &new {
        let (form, name) = id.parts();
        let Some((_, layer)) = LAYERS.iter().find(|(known, _)| *known == form) else {
            continue;
        };
        let mut bound = commands.entity(entity);
        bound.insert(action_of(form, name));
        match name {
            "requester" => bound.insert(*layer),
            "keep" => bound.insert(KeepButton),
            _ => &mut bound,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_requesters_are_good_markup() {
        REQUESTERS.assert_good();
    }

    #[test]
    fn buttons_keep_the_ids_their_actions_give_them() {
        for (form, action) in [
            ("quit", Action::ReallyQuit),
            ("quit", Action::Back),
            ("ironman", Action::ChooseIronman),
            ("settings.keep_graphics", Action::KeepGraphics),
            ("refused", Action::Back),
            ("slow_drawing", Action::QuietSlowDrawing),
            ("going_online", Action::ReallyToggleOnline),
            ("reopening", Action::ReallyToggleOnline),
            ("online_failed", Action::Back),
        ] {
            let (_, name) = action.ui_id(form).parts();
            assert_eq!(action_of(form, name), action, "{form}/{name}");
        }
    }
}
