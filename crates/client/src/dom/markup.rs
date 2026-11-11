//! Markup as written, before anything is known about what its elements are:
//! HTML's look with XML's closing rule. Every element is closed
//! (`<div>…</div>`) or closes itself (`<br/>`), names are lower case, and an
//! attribute may stand alone (`<p hidden>`), as in HTML.

use std::borrow::Cow;

use super::error::{MarkupError, Place};

mod cursor;
mod parse;
mod text;

pub(super) use parse::parse;

/// An element or a run of words.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum RawNode {
    Element(RawElement),
    /// Words, with their spaces already tidied, and where they start.
    Text(String, Place),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RawElement {
    pub name: &'static str,
    pub attrs: Vec<RawAttr>,
    pub children: Vec<RawNode>,
    pub at: Place,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RawAttr {
    pub name: &'static str,
    /// `None` for one standing alone. Borrowed from the source unless it had
    /// `&…;` in it.
    pub value: Option<Cow<'static, str>>,
    pub at: Place,
}

#[cfg(test)]
mod tests;
