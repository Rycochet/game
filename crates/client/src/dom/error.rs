//! What is wrong with a piece of markup, and where.

use std::fmt;

/// A place in a template's source, both counted from 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    pub line: u32,
    pub column: u32,
}

impl Place {
    /// Where byte `at` of `source` is.
    pub(super) fn of(source: &str, at: usize) -> Self {
        let before = &source[..at.min(source.len())];
        let line = before.matches('\n').count() as u32 + 1;
        let start = before.rfind('\n').map_or(0, |newline| newline + 1);
        let column = before[start..].chars().count() as u32 + 1;
        Self { line, column }
    }
}

/// Markup that can't be used: nothing is ever guessed at or left out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkupError {
    pub at: Place,
    pub what: String,
}

impl MarkupError {
    pub(super) fn new(at: Place, what: impl Into<String>) -> Self {
        Self {
            at,
            what: what.into(),
        }
    }
}

impl fmt::Display for MarkupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}: {}", self.at.line, self.at.column, self.what)
    }
}
