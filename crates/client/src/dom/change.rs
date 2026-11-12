//! Changing documents from code: always by id, never by entity. Changes are
//! kept in the order asked for and made at the end of the frame, after new
//! documents are built, so a system may spawn a document and fill it at once.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

mod apply;
mod options;
mod rows;

pub(super) use apply::apply_changes;

/// Something to change about the element with this id (`form/name`).
#[derive(Message, Clone, Debug, PartialEq)]
pub(super) struct DomChange {
    pub id: String,
    pub change: Change,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Change {
    /// Its words. Elements in it stay.
    Text(String),
    /// An attribute, to a value or standing alone.
    Set(String, Option<String>),
    /// An attribute taken off.
    Clear(String),
    /// A copy of the template with this id added to the end of it.
    Copy { template: String, index: usize },
    /// Everything in it taken out.
    Empty,
    /// A `select`'s options, each its value and its words.
    Options(Vec<(String, String)>),
    /// A panel that moves, brought in front of the others.
    Raise,
}

/// How systems change documents. Ids are as written, `form/name`, with
/// `.<index>` on those in a copy of a template.
#[derive(SystemParam)]
pub struct Dom<'w> {
    changes: MessageWriter<'w, DomChange>,
}

// Not every way of changing things is used until more screens are markup.
#[allow(dead_code)]
impl Dom<'_> {
    fn ask(&mut self, id: &str, change: Change) {
        let id = id.to_owned();
        self.changes.write(DomChange { id, change });
    }

    /// Changes an element's words, leaving any elements in it.
    pub fn set_text(&mut self, id: &str, words: impl Into<String>) {
        self.ask(id, Change::Text(words.into()));
    }

    /// Sets an attribute, as markup would: `dom.set("a/bar", "value", "60")`.
    pub fn set(&mut self, id: &str, name: &str, value: &str) {
        self.ask(id, Change::Set(name.to_owned(), Some(value.to_owned())));
    }

    /// Turns a yes-or-no attribute (`hidden`, `disabled`, `inert`) on or off.
    pub fn set_flag(&mut self, id: &str, name: &str, on: bool) {
        let name = name.to_owned();
        let change = if on {
            Change::Set(name, None)
        } else {
            Change::Clear(name)
        };
        self.ask(id, change);
    }

    /// Takes an attribute off, back to what the element does by itself.
    pub fn clear(&mut self, id: &str, name: &str) {
        self.ask(id, Change::Clear(name.to_owned()));
    }

    /// Adds a copy of a `template` to the end of the element `into`. Ids in
    /// the copy get `.<index>`: `connect/server.2`.
    pub fn copy_template(&mut self, template: &str, into: &str, index: usize) {
        let template = template.to_owned();
        self.ask(into, Change::Copy { template, index });
    }

    /// Takes out everything in an element, to fill it again.
    pub fn empty(&mut self, id: &str) {
        self.ask(id, Change::Empty);
    }

    /// Where a panel that moves starts, in the interface's pixels: only
    /// until it is first placed, so for one just spawned.
    pub fn start_at(&mut self, id: &str, at: Vec2) {
        self.set(id, "left", &at.x.to_string());
        self.set(id, "top", &at.y.to_string());
    }

    /// Brings a panel that moves in front of the others.
    pub fn raise(&mut self, id: &str) {
        self.ask(id, Change::Raise);
    }

    /// Gives a `select` these options instead of its own, each its value
    /// and its words. Set its `value` after, to say which is chosen.
    pub fn set_options(&mut self, id: &str, options: impl IntoIterator<Item = (String, String)>) {
        self.ask(id, Change::Options(options.into_iter().collect()));
    }
}

#[cfg(test)]
mod tests;
