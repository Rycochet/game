//! A piece of markup built into the program, by name.

use super::check::{CheckedNode, check};
use super::error::MarkupError;
use super::markup::parse;

/// Markup to build documents from. Make one with `html!`, so the source is
/// part of the program: interface files are never read while running.
#[derive(Debug, PartialEq, Eq)]
pub struct Template {
    /// For saying which template an error is in.
    pub name: &'static str,
    pub source: &'static str,
}

/// A `Template` from its name and its markup, written inline (`r#"…"#`) or
/// brought in from a file beside the module when compiling (`include_str!`).
#[macro_export]
macro_rules! html {
    ($name:literal, $source:expr) => {
        $crate::dom::Template {
            name: $name,
            source: $source,
        }
    };
}

impl Template {
    /// Its elements, or the first thing wrong with it.
    pub fn checked(&self) -> Result<Vec<CheckedNode>, MarkupError> {
        check(parse(self.source)?)
    }

    /// Fails a test if the template has anything wrong with it. Every module
    /// with templates has a test calling this for each, so a slip is found by
    /// `cargo xtask test` and not in the game.
    #[cfg(test)]
    pub fn assert_good(&self) {
        if let Err(error) = self.checked() {
            panic!("{}:{error}", self.name);
        }
    }
}
