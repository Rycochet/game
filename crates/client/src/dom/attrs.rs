//! An element's attributes, read into what they mean. There is no CSS: places
//! and sizes are `top`, `left`, `right`, `bottom`, `width` and `height`,
//! spacing is the theme's unless `margin` says otherwise, and `class` only
//! picks one of the theme's looks.

use std::borrow::Cow;

use super::tag::Tag;

mod set;
mod values;

pub use values::{Align, Length, VAlign};

/// The four sides, in the order `margin` values are kept.
pub(super) const SIDES: [&str; 4] = ["top", "right", "bottom", "left"];

/// Everything an element says about itself, but its `id` and `form`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Attrs {
    pub class: Option<Cow<'static, str>>,
    /// Words shown on pointing at it.
    pub title: Option<Cow<'static, str>>,
    /// A label's: the `id` of what it is for.
    pub for_id: Option<Cow<'static, str>>,
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
    pub src: Option<Cow<'static, str>>,
    pub alt: Option<Cow<'static, str>>,
    pub value: Option<f32>,
    pub max: Option<f32>,
    pub colspan: Option<u16>,
    pub cellspacing: Option<f32>,
    pub cellpadding: Option<f32>,
}

impl Attrs {
    /// Whether it is placed in its parent by its sides, out of the flow.
    pub fn is_placed(&self) -> bool {
        self.top.is_some() || self.left.is_some() || self.right.is_some() || self.bottom.is_some()
    }

    /// What is wrong with these taken together, if anything.
    pub(super) fn conflict(&self) -> Option<&'static str> {
        if self.left.is_some() && self.right.is_some() && self.width.is_some() {
            Some("`left` and `right` already give the width")
        } else if self.top.is_some() && self.bottom.is_some() && self.height.is_some() {
            Some("`top` and `bottom` already give the height")
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
