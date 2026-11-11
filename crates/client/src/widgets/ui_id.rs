//! Names for widgets, as HTML's forms and ids: which form a widget belongs to and
//! its name in it, written `form/name`, or `form#<id>/name` for one of several
//! alike (a bag's, a shop's). The form is only a name every widget of it carries,
//! not a node round them, so a panel can hold several and a form can move between
//! pages. Scroll areas are found again by theirs, and the debug API reads, sets
//! and presses widgets by them.

use std::fmt;

use bevy::prelude::*;
use shared::id::Id;

/// A widget's name: its form, which one of those (when there can be several, as
/// bags), and its own name in the form.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UiId {
    form: &'static str,
    of: Option<Id>,
    name: &'static str,
    /// Which of a list of these in the form (a remembered server).
    at: Option<usize>,
}

impl UiId {
    pub const fn new(form: &'static str, name: &'static str) -> Self {
        Self {
            form,
            of: None,
            name,
            at: None,
        }
    }

    /// In one of several forms alike, told apart by `of`.
    pub const fn of(form: &'static str, of: Id, name: &'static str) -> Self {
        Self {
            form,
            of: Some(of),
            name,
            at: None,
        }
    }

    /// The one at this place in a list of them, written `name.<at>`.
    pub const fn at(self, at: usize) -> Self {
        Self {
            at: Some(at),
            ..self
        }
    }

    /// Its form's name and its own, without which of several or its place in
    /// a list.
    pub fn parts(&self) -> (&'static str, &'static str) {
        (self.form, self.name)
    }

    /// Its place in a list of them, if it is one of a list.
    pub fn index(&self) -> Option<usize> {
        self.at
    }

    /// Its form as written: `form`, or `form#<id>`.
    pub fn form(&self) -> String {
        match self.of {
            Some(of) => format!("{}#{of}", self.form),
            None => self.form.to_owned(),
        }
    }

    /// Whether it is in the form asked for: its name alone (every one alike),
    /// or with which one.
    #[cfg(any(test, feature = "debug-api"))]
    pub fn in_form(&self, asked: &str) -> bool {
        asked == self.form || asked == self.form()
    }
}

impl fmt::Display for UiId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.form(), self.name)?;
        match self.at {
            Some(at) => write!(f, ".{at}"),
            None => Ok(()),
        }
    }
}

/// The ids more than one of these has, sorted.
pub fn duplicate_ids<'a>(ids: impl IntoIterator<Item = &'a UiId>) -> Vec<String> {
    let mut written: Vec<String> = ids.into_iter().map(UiId::to_string).collect();
    written.sort();
    let mut twice: Vec<String> = written
        .windows(2)
        .filter(|pair| pair[0] == pair[1])
        .map(|pair| pair[0].clone())
        .collect();
    twice.dedup();
    twice
}

/// When widgets get ids, warns of any two with the same one: the debug API
/// can't tell them apart.
pub(super) fn warn_of_duplicates(added: Query<(), Added<UiId>>, ids: Query<&UiId>) {
    if added.is_empty() {
        return;
    }
    for id in duplicate_ids(ids.iter()) {
        warn!("More than one widget is called {id}");
    }
}

/// Put on a check button from `spawn_check_button`: whether it is ticked. Change
/// it and the tick follows.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Checked(pub bool);

#[cfg(test)]
mod tests;
