//! Showing what the panel knows, in place, whenever that changes: the chosen
//! tab's page, each page's words and choices, and why the last thing asked
//! was refused.

use bevy::prelude::*;

use super::state::{Panel, Tab};
use super::{audit, find, roles};
use crate::dom::Dom;

pub(super) fn show_panel(mut panel: ResMut<Panel>, mut dom: Dom) {
    if panel.document.is_none() || !panel.dirty {
        return;
    }
    panel.dirty = false;
    let chosen = panel.tab();
    for tab in Tab::ALL {
        dom.set_flag(tab.page(), "hidden", chosen != Some(tab));
    }
    dom.set_flag("accounts/nothing", "hidden", chosen.is_some());
    find::show_find(&mut panel, &mut dom);
    roles::show_roles(&mut panel, &mut dom);
    audit::show_audit(&panel, &mut dom);
    dom.set_flag("accounts/refused", "hidden", panel.refused.is_none());
    if let Some(why) = &panel.refused {
        dom.set_text("accounts/refused", capitalised(why));
    }
}

/// A list's options, each chosen by its place in the list.
pub(super) fn numbered<'a>(
    words: impl Iterator<Item = &'a str>,
) -> impl Iterator<Item = (String, String)> {
    words
        .enumerate()
        .map(|(index, words)| (index.to_string(), words.to_owned()))
}

/// The server's reasons start in lower case, as they follow other words.
pub(super) fn capitalised(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}
