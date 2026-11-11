//! The login screen, as markup: how to log in to this server (a button for
//! each provider, Play as Guest if it takes guests, Cancel), then while the
//! player logs in on the provider's page in their browser (Open Again, Back).

use bevy::prelude::*;

use crate::common::{LoggingIn, LoginChoice, open_in_browser};
use crate::dom::{Dom, Template, document};
use crate::html;
use crate::widgets::{ButtonPressed, UiId};

/// Over the loading screen, as making a character is. Each provider's
/// button is a copy of the template; the buttons that don't go with what is
/// asked are hidden, and the first on show is the main one.
static LOGIN: Template = html!(
    "login",
    r#"
<form id="login">
  <dialog id="layer" title="Log in" depth="0" hidden>
    <div class="rows" width="420">
      <p id="words" class="medium"/>
      <p id="failed" class="refused" hidden/>
      <div id="buttons">
        <div id="providers" hidden/>
        <button id="guest" hidden>Play as Guest</button>
        <button id="cancel" hidden>Cancel</button>
        <button id="open_again" hidden>Open Again</button>
        <button id="back" hidden>Back</button>
      </div>
    </div>
    <template id="provider"><button id="provider"/></template>
  </dialog>
</form>
"#
);

/// The buttons that are always there, shown or hidden.
const FIXED: [&str; 4] = ["guest", "cancel", "open_again", "back"];

pub(super) fn spawn_screen(mut commands: Commands) {
    commands.spawn(document(&LOGIN));
}

/// What the screen says and offers, for where logging in has got to: its
/// words, whether they say a try failed, how many providers, and which of
/// the fixed buttons.
fn asked(logging_in: &LoggingIn) -> Option<(String, bool, Vec<String>, Vec<&'static str>)> {
    match logging_in {
        LoggingIn::Not => None,
        LoggingIn::Choosing {
            providers,
            guests,
            failed,
        } => {
            let labels = providers.iter();
            let labels = labels.map(|provider| format!("Log in with {}", provider.label));
            let shown = match guests {
                true => vec!["guest", "cancel"],
                false => vec!["cancel"],
            };
            let (words, failed) = match failed {
                Some(reason) => (format!("Logging in didn't work: {reason}."), true),
                None => (choosing_words(*guests), false),
            };
            Some((words, failed, labels.collect(), shown))
        }
        LoggingIn::InBrowser { provider, url } => {
            let (words, shown) = match url {
                Some(_) => (
                    format!(
                        "Log in with {} in your browser, then come back here. \
                         If no page opened, press Open Again.",
                        provider.label
                    ),
                    vec!["open_again", "back"],
                ),
                None => (
                    format!("Opening {}'s page in your browser...", provider.label),
                    vec!["back"],
                ),
            };
            Some((words, false, Vec::new(), shown))
        }
    }
}

/// Shows the screen while there is something to ask, set for it.
pub(super) fn show_screen(logging_in: Res<LoggingIn>, mut dom: Dom) {
    if !logging_in.is_changed() {
        return;
    }
    dom.set_flag("login/layer", "hidden", !logging_in.shows());
    let Some((words, failed, providers, shown)) = asked(&logging_in) else {
        return;
    };
    // A failure is said in its own colour.
    let (said, unsaid) = match failed {
        true => ("login/failed", "login/words"),
        false => ("login/words", "login/failed"),
    };
    dom.set_text(said, words);
    dom.set_flag(said, "hidden", false);
    dom.set_flag(unsaid, "hidden", true);
    dom.empty("login/providers");
    dom.set_flag("login/providers", "hidden", providers.is_empty());
    for (at, label) in providers.iter().enumerate() {
        dom.copy_template("login/provider", "login/providers", at);
        dom.set_text(&format!("login/provider.{at}"), label.clone());
    }
    for name in FIXED {
        dom.set_flag(&format!("login/{name}"), "hidden", !shown.contains(&name));
        dom.clear(&format!("login/{name}"), "class");
    }
    // The first button on show is the main thing to do.
    let first = match providers.is_empty() {
        true => format!("login/{}", shown[0]),
        false => "login/provider.0".to_owned(),
    };
    dom.set(&first, "class", "main");
}

fn choosing_words(guests: bool) -> String {
    if guests {
        "Log in to keep your characters with your account, or play as a guest on this \
         computer."
            .into()
    } else {
        "This server needs you to log in.".into()
    }
}

/// What the button with this id does now, if it is one of the screen's.
fn choice_of(id: &UiId, logging_in: &LoggingIn) -> Option<Result<LoginChoice, String>> {
    let ("login", name) = id.parts() else {
        return None;
    };
    match (name, logging_in) {
        ("provider", LoggingIn::Choosing { providers, .. }) => {
            let provider = providers.get(id.index()?)?;
            Some(Ok(LoginChoice::Provider(provider.name.clone())))
        }
        ("guest", _) => Some(Ok(LoginChoice::Guest)),
        ("cancel", _) => Some(Ok(LoginChoice::Cancel)),
        ("back", _) => Some(Ok(LoginChoice::Back)),
        ("open_again", LoggingIn::InBrowser { url: Some(url), .. }) => Some(Err(url.clone())),
        _ => None,
    }
}

/// Does what the button pressed says: a choice, or the login page again.
pub(super) fn press_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    ids: Query<&UiId>,
    logging_in: Res<LoggingIn>,
    mut choices: MessageWriter<LoginChoice>,
) {
    for press in pressed.read() {
        let Ok(id) = ids.get(press.0) else {
            continue;
        };
        match choice_of(id, &logging_in) {
            Some(Ok(choice)) => {
                choices.write(choice);
            }
            Some(Err(url)) => {
                if let Err(reason) = open_in_browser(&url) {
                    warn!("Can't open the login page: {reason}");
                }
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod tests;
