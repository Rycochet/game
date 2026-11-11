//! An element's attributes, read into what they mean. There is no CSS: places
//! and sizes are `top`, `left`, `right`, `bottom`, `width` and `height`,
//! spacing is the theme's unless `margin` says otherwise, and `class` only
//! picks one of the theme's looks.

use std::borrow::Cow;

use super::tag::Tag;

mod own;
mod set;
mod values;
#[cfg(any(test, feature = "debug-api"))]
mod write;

pub use values::{Align, Length, VAlign};

/// The four sides, in the order `margin` values are kept.
pub(super) const SIDES: [&str; 4] = ["top", "right", "bottom", "left"];

/// Everything an element says about itself, but its `id` and `form`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Attrs {
    pub class: Option<Cow<'static, str>>,
    /// Words shown on pointing at it.
    pub title: Option<Cow<'static, str>>,
    /// Not laid out or shown, though still there.
    pub hidden: bool,
    pub disabled: bool,
    /// Never what the pointer is on, with all that is in it.
    pub inert: bool,
    pub align: Option<Align>,
    pub valign: Option<VAlign>,
    pub top: Option<Length>,
    pub left: Option<Length>,
    pub right: Option<Length>,
    pub bottom: Option<Length>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    /// In place of the theme's, by side as in `SIDES`.
    pub margin: [Option<f32>; 4],
    /// The attributes only some elements take (`own::OWN`), as written and
    /// already checked, in the order they were set. One standing alone is "".
    own: Vec<(&'static str, Cow<'static, str>)>,
}

impl Attrs {
    /// One of an element's own attributes, as written: `attrs.text("src")`.
    pub fn text(&self, name: &str) -> Option<&str> {
        let found = self.own.iter().find(|(known, _)| *known == name);
        found.map(|(_, value)| value.as_ref())
    }

    /// One as written, if it is still a piece of its template's source (not
    /// set since by code, and with no `&…;` in it).
    pub(super) fn static_text(&self, name: &str) -> Option<&'static str> {
        let found = self.own.iter().find(|(known, _)| *known == name);
        found.and_then(|(_, value)| match value {
            Cow::Borrowed(written) => Some(*written),
            Cow::Owned(_) => None,
        })
    }

    /// Puts one that stands alone there, or takes it off.
    pub(super) fn set_flag(&mut self, name: &'static str, on: bool) {
        self.own.retain(|(known, _)| *known != name);
        if on {
            self.own.push((name, Cow::Borrowed("")));
        }
    }

    /// One that is a number: `attrs.number("max")`.
    pub fn number(&self, name: &str) -> Option<f32> {
        self.text(name).and_then(|value| value.parse().ok())
    }

    /// One that is a length: `attrs.length("max-height")`.
    pub fn length(&self, name: &str) -> Option<Length> {
        let written = self.text(name)?;
        values::length(name, written).ok()
    }

    /// Whether one that stands alone is there: `attrs.has("checked")`.
    pub fn has(&self, name: &str) -> bool {
        self.text(name).is_some()
    }

    /// Whether it is placed in its parent by its sides, out of the flow.
    pub fn is_placed(&self) -> bool {
        self.top.is_some() || self.left.is_some() || self.right.is_some() || self.bottom.is_some()
    }

    /// What is wrong with these taken together on a `tag`, if anything.
    pub(super) fn problem(&self, tag: Tag) -> Option<String> {
        let numeric = |name: &str| self.text(name).is_none_or(|_| self.number(name).is_some());
        let kind = self.text("type");
        if self.left.is_some() && self.right.is_some() && self.width.is_some() {
            Some("`left` and `right` already give the width".into())
        } else if self.top.is_some() && self.bottom.is_some() && self.height.is_some() {
            Some("`top` and `bottom` already give the height".into())
        } else if (tag == Tag::Progress || kind == Some("range")) && !numeric("value") {
            Some("`value` must be a number here".into())
        } else {
            None
        }
    }
}

/// The attributes every element takes (`id` and `form` are read apart).
pub(super) fn is_global(name: &str) -> bool {
    const GLOBAL: [&str; 18] = [
        "class",
        "title",
        "hidden",
        "disabled",
        "inert",
        "align",
        "valign",
        "top",
        "left",
        "right",
        "bottom",
        "width",
        "height",
        "margin",
        "margin-top",
        "margin-right",
        "margin-bottom",
        "margin-left",
    ];
    GLOBAL.contains(&name)
}

/// Whether `tag` takes an attribute of this name.
pub(super) fn is_known(tag: Tag, name: &str) -> bool {
    is_global(name) || tag.own_attrs().contains(&name)
}

#[cfg(test)]
mod tests;
