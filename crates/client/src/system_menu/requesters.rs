//! The menu's requesters, written as markup: Quit, Ironman, Keep these
//! settings, Can't connect and Slow drawing. What they do is unchanged: once
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
    <p margin-bottom="0">Are you sure?</p>
    <footer>
      <button id="quit" class="main" width="122" height="38">Yes, quit</button>
      <button id="back" width="122" height="38">No</button>
    </footer>
  </dialog>
</form>
<form id="ironman">
  <dialog id="requester" title="Ironman" depth="2" hidden>
    <p id="question" width="420" margin-bottom="0"/>
    <footer>
      <button id="ironman" class="main" width="130" height="38">Yes, Ironman</button>
      <button id="no" width="130" height="38">No</button>
    </footer>
  </dialog>
</form>
<form id="settings.keep_graphics">
  <dialog id="requester" title="Graphics" depth="2" hidden>
    <p margin-bottom="0">Keep these settings?</p>
    <p id="countdown" margin-bottom="0"/>
    <footer>
      <button id="keep" class="main" width="130" height="38">Keep</button>
      <button id="back" width="130" height="38">Go back</button>
    </footer>
  </dialog>
</form>
<form id="refused">
  <dialog id="requester" title="Can't connect" depth="1" hidden>
    <p id="words" class="warning" width="460" margin-bottom="0"/>
    <footer>
      <button id="back" class="main" width="116" height="38">OK</button>
    </footer>
  </dialog>
</form>
<form id="slow_drawing">
  <dialog id="requester" title="Slow drawing" depth="1" hidden>
    <p id="words" width="460" margin-bottom="0"/>
    <footer>
      <button id="back" class="main" width="184" height="38">Play Anyway</button>
      <button id="dont_warn" width="184" height="38">Don't Warn Again</button>
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
const LAYERS: [(&str, Layer); 5] = [
    ("quit", Layer(&[SystemMenu::Confirming])),
    ("ironman", Layer(&[SystemMenu::Ironman])),
    ("settings.keep_graphics", Layer(&[SystemMenu::KeepGraphics])),
    ("refused", Layer(&[SystemMenu::Refused])),
    ("slow_drawing", Layer(&[SystemMenu::SlowDrawing])),
];

/// What each button does, by its form and name. Any other goes back.
fn action_of(form: &str, name: &str) -> Action {
    match (form, name) {
        ("quit", "quit") => Action::ReallyQuit,
        ("ironman", "ironman") => Action::ChooseIronman,
        ("settings.keep_graphics", "keep") => Action::KeepGraphics,
        ("slow_drawing", "dont_warn") => Action::QuietSlowDrawing,
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
        ] {
            let (_, name) = action.ui_id(form).parts();
            assert_eq!(action_of(form, name), action, "{form}/{name}");
        }
    }
}
