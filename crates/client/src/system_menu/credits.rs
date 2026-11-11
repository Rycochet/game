//! The Credits layer: who made the game and what it is made with, scrolling by
//! itself when there is more than fits. `docs/credits.md` says the same for
//! the website, with the licences and links.

use bevy::prelude::*;

use super::{Action, Layer, SystemMenu};
use crate::dom::{Dom, Template, document};
use crate::html;
use crate::widgets::{AutoScroll, UiId};

/// The layer: a title with a close button, and the list, which scrolls by
/// itself. Each entry is a copy of the template, filled from `CREDITS`.
static CREDITS_LAYER: Template = html!(
    "system_menu.credits",
    r#"
<form id="credits">
  <dialog id="layer" title="Credits" modal closable depth="1" width="560" hidden>
    <scroll id="list" max-height="65%" auto>
      <template id="entry">
        <div class="tight" align="center" width="496" margin-bottom="12">
          <p id="what" class="faint"/>
          <p id="who"/>
        </div>
      </template>
    </scroll>
  </dialog>
</form>
"#
);

/// What was done, and by whom (or what it is, and whose).
pub(super) const CREDITS: &[(&str, &str)] = &[
    ("Created by", "Ryc O'Chet"),
    ("Code and placeholder 3D images", "Claude Code (Opus 5.5)"),
    (
        "Monster and terrain tiles, and the AngbandTk tiles",
        "David E. Gervais",
    ),
    (
        "Icons, from game-icons.net",
        "Lorc, Delapouite and the site's other contributors",
    ),
    ("Title font: Cinzel Decorative", "Natanael Gama"),
    (
        "Writing font: Lexend",
        "Bonnie Shaver-Troup, Thomas Jockin, Santiago Orozco, Héctor Gómez and Superunion",
    ),
    ("Emoji font: Noto Emoji", "Google"),
    (
        "Some creatures (the drider): System Reference Document 5.1, CC BY 4.0",
        "Wizards of the Coast LLC",
    ),
    (
        "Built with",
        "Bevy, and the many open source libraries it and the game use",
    ),
    (
        "Licence",
        "Hall of Worlds may be shared and changed, but not sold: Creative Commons \
         Attribution-NonCommercial 4.0. The art and fonts keep their own licences.",
    ),
    ("", "Thank you for playing!"),
];

/// The list, so opening the layer can start it from the top.
#[derive(Component)]
pub(super) struct CreditsList;

pub(super) fn spawn_credits(commands: &mut Commands, dom: &mut Dom) {
    commands.spawn(document(&CREDITS_LAYER));
    for (entry, &(what, who)) in CREDITS.iter().enumerate() {
        dom.copy_template("credits/entry", "credits/list", entry);
        dom.set_text(&format!("credits/who.{entry}"), who);
        let what_id = format!("credits/what.{entry}");
        if what.is_empty() {
            dom.set_flag(&what_id, "hidden", true);
        } else {
            dom.set_text(&what_id, what);
        }
    }
}

/// Gives the built layer, its close button and its list what the menu
/// knows them by, found by their ids.
pub(super) fn bind_credits(mut commands: Commands, new: Query<(Entity, &UiId), Added<UiId>>) {
    for (entity, id) in &new {
        let mut bound = commands.entity(entity);
        match id.parts() {
            ("credits", "layer") => bound.insert((Layer(&[SystemMenu::Credits]), Action::Back)),
            ("credits", "close") => bound.insert(Action::Back),
            ("credits", "list") => bound.insert(CreditsList),
            _ => continue,
        };
    }
}

/// Each time the credits open, they start from the top and scroll by themselves
/// again, until scrolled by hand.
pub(super) fn start_credits(
    menu: Res<SystemMenu>,
    mut list: Single<(Entity, &mut ScrollPosition), With<CreditsList>>,
    mut commands: Commands,
) {
    if menu.is_changed() && *menu == SystemMenu::Credits {
        list.1.y = 0.0;
        commands.entity(list.0).insert(AutoScroll::default());
    }
}

#[cfg(test)]
mod tests;
