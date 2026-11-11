//! A place in a template's source, moving along it.

use super::{MarkupError, Place};

pub(super) struct Cursor {
    source: &'static str,
    at: usize,
}

impl Cursor {
    pub fn new(source: &'static str) -> Self {
        Self { source, at: 0 }
    }

    /// What hasn't been read yet.
    pub fn rest(&self) -> &'static str {
        &self.source[self.at..]
    }

    pub fn place(&self) -> Place {
        Place::of(self.source, self.at)
    }

    pub fn error(&self, what: impl Into<String>) -> MarkupError {
        MarkupError::new(self.place(), what)
    }

    /// Moves past `expected` if it is next.
    pub fn eat(&mut self, expected: &str) -> bool {
        let found = self.rest().starts_with(expected);
        if found {
            self.at += expected.len();
        }
        found
    }

    pub fn skip_space(&mut self) {
        let rest = self.rest();
        self.at += rest.len() - rest.trim_start().len();
    }

    /// Takes everything up to `end`, leaving `end` unread; `None` if it never comes.
    pub fn take_until(&mut self, end: &str) -> Option<&'static str> {
        let rest = self.rest();
        let length = rest.find(end)?;
        self.at += length;
        Some(&rest[..length])
    }

    /// Takes everything up to the next `<`, or all that is left.
    pub fn take_text(&mut self) -> &'static str {
        let rest = self.rest();
        let length = rest.find('<').unwrap_or(rest.len());
        self.at += length;
        &rest[..length]
    }

    /// An element's or attribute's name: lower case letters, digits and hyphens,
    /// starting with a letter.
    pub fn name(&mut self) -> Result<&'static str, MarkupError> {
        let rest = self.rest();
        let length = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .unwrap_or(rest.len());
        let name = &rest[..length];
        if !name.starts_with(|c: char| c.is_ascii_alphabetic()) {
            return Err(self.error("a name was expected here"));
        }
        if name.chars().any(|c| c.is_ascii_uppercase()) {
            return Err(self.error(format!("`{name}` must be in lower case")));
        }
        self.at += length;
        Ok(name)
    }
}
