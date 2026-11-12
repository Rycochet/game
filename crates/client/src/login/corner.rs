//! The bottom left corner, at the startup menu and while making a character:
//! "Playing on" the server Play goes to, if it isn't the usual one, and
//! "Logged in as" the account's email address. Never in game. The menu's Logout
//! forgets both (`system_menu`, `connection`).
//!
//! Its place is markup, but its words are written here into that place and
//! never given to `dom`: what `dom` holds can be read back through the debug
//! API, which must never answer with an email address (CLAUDE.md's rule).

use bevy::prelude::*;

use crate::common::{AccountEmail, ChosenServer, MakingCharacter, Session};
use crate::dom::{Template, bottom_strip, document};
use crate::html;
use crate::widgets::{TextSize, UiId, layer_z};

/// Light, to read on the title's dark background.
const WORDS: Color = Color::srgb(0.93, 0.9, 0.82);

/// Where the words go: 16 from the window's bottom left corner, never in
/// the pointer's way.
static CORNER: Template = html!(
    "login.corner",
    r#"
<canvas id="corner" form="login" left="16" bottom="16" inert/>
"#
);

#[derive(Component)]
pub(super) struct CornerWords;

pub(super) fn spawn_corner(mut commands: Commands) {
    // Over the startup menu's layer, under the requesters above it.
    let corner = (document(&CORNER), GlobalZIndex(layer_z(1) - 1));
    commands.spawn(corner).insert(bottom_strip());
}

/// Puts the words, empty and hidden, in their place once it is built.
pub(super) fn bind_corner(new: Query<(Entity, &UiId), Added<UiId>>, mut commands: Commands) {
    for (entity, id) in &new {
        if id.parts() == ("login", "corner") {
            let font = TextSize::Medium.font();
            let words = (CornerWords, Text::new(""), font, TextColor(WORDS));
            commands
                .entity(entity)
                .with_child((words, Visibility::Hidden));
        }
    }
}

/// Shows the server and account at the startup menu and while making a
/// character (only there: CLAUDE.md's rule on email addresses).
pub(super) fn show_corner(
    email: Res<AccountEmail>,
    chosen: Res<ChosenServer>,
    session: Res<Session>,
    making: Res<MakingCharacter>,
    new: Query<(), Added<CornerWords>>,
    words: Option<Single<(&mut Text, &mut Visibility), With<CornerWords>>>,
) {
    // (Not there until its place is built.)
    let Some(mut words) = words else {
        return;
    };
    let changed = email.is_changed()
        || chosen.is_changed()
        || session.is_changed()
        || making.is_changed()
        || !new.is_empty();
    if !changed {
        return;
    }
    let mut lines = Vec::new();
    if let Some(server) = &chosen.0 {
        lines.push(format!("Playing on {}", server.label()));
    }
    if let Some(email) = &email.0 {
        lines.push(format!("Logged in as {email}"));
    }
    let shown = !lines.is_empty() && (!session.is_playing() || making.0);
    words.1.set_if_neq(if shown {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    words.0.0 = lines.join("\n");
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_corner_is_good_markup() {
        super::CORNER.assert_good();
    }
}
