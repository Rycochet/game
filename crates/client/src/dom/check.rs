//! Markup checked into elements that can be shown. Strict: an element or
//! attribute that isn't known, an `id` used twice, a `for` naming nothing, an
//! element where it can't go are all errors, never ignored.

use std::borrow::Cow;
use std::collections::HashSet;

use super::attrs::Attrs;
use super::error::{MarkupError, Place};
use super::markup::{RawAttr, RawElement, RawNode};
use super::tag::{Holds, Tag};
use crate::widgets::UiId;

/// An element or a run of words, checked.
// Most nodes are elements, so boxing them to shrink the few runs of words
// would only add a step to every one.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum CheckedNode {
    Element(Checked),
    Text(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Checked {
    pub tag: Tag,
    /// Its name, `form/id`, if it has one. A form has none: its `id` is the
    /// form's name, which `form` holds.
    pub id: Option<UiId>,
    /// For a `form`, the name it gives what is in it.
    pub form: Option<&'static str>,
    /// For a label with `for`: the id of what it is for.
    pub label_for: Option<UiId>,
    pub attrs: Attrs,
    pub children: Vec<CheckedNode>,
}

/// Where an element being checked is.
#[derive(Clone, Copy, Default)]
struct Inside {
    form: Option<&'static str>,
    parent: Option<Tag>,
}

/// The ids met so far, and those labels are `for`.
#[derive(Default)]
struct Checker {
    ids: HashSet<String>,
    wanted: Vec<(String, Place)>,
}

/// The checked elements of some markup, or the first thing wrong with it.
pub(super) fn check(raw: Vec<RawNode>) -> Result<Vec<CheckedNode>, MarkupError> {
    let mut checker = Checker::default();
    let nodes = checker.nodes(raw, Inside::default())?;
    checker.labels()?;
    Ok(nodes)
}

impl Checker {
    fn nodes(
        &mut self,
        raw: Vec<RawNode>,
        inside: Inside,
    ) -> Result<Vec<CheckedNode>, MarkupError> {
        let mut nodes = Vec::with_capacity(raw.len());
        for node in raw {
            nodes.push(match node {
                RawNode::Element(element) => CheckedNode::Element(self.element(element, inside)?),
                RawNode::Text(_, at) if inside.parent.is_none() => {
                    return Err(MarkupError::new(at, "words must be inside an element"));
                }
                RawNode::Text(words, _) => CheckedNode::Text(words),
            });
        }
        Ok(nodes)
    }

    fn element(&mut self, raw: RawElement, inside: Inside) -> Result<Checked, MarkupError> {
        let at = raw.at;
        let error = |what: String| MarkupError::new(at, what);
        let name = raw.name;
        let tag = Tag::named(name).ok_or_else(|| error(format!("`<{name}>` is not known")))?;
        let homes = tag.belongs_in();
        if !homes.is_empty() && !inside.parent.is_some_and(|parent| homes.contains(&parent)) {
            let homes: Vec<String> = homes
                .iter()
                .map(|home| format!("`<{}>`", home.name()))
                .collect();
            return Err(error(format!("`<{name}>` goes in {}", homes.join(" or "))));
        }
        fits(tag, &raw.children).map_err(|(at, what)| MarkupError::new(at, what))?;

        let mut attrs = Attrs::default();
        let (mut own_id, mut form) = (None, inside.form);
        for RawAttr { name, value, at } in raw.attrs {
            let set = match name {
                "id" => plain(name, value).map(|id| own_id = Some(id)),
                "form" => plain(name, value).map(|named| form = Some(named)),
                _ => attrs.set(tag, name, value),
            };
            set.map_err(|what| MarkupError::new(at, what))?;
        }
        let moves = tag == Tag::Panel && (attrs.has("moveable") || attrs.has("grip"));
        if (tag.needs_id() || moves) && own_id.is_none() {
            return Err(error(format!("`<{name}>` needs an `id`")));
        }
        if tag == Tag::Input && !attrs.has("type") {
            return Err(error("`<input>` needs a `type`".into()));
        }
        let mut label_for = None;
        if attrs.has("for") {
            let within = form.ok_or_else(|| error("`for` needs a form to look in".into()))?;
            let wanted = attrs.static_text("for");
            let wanted = wanted.ok_or_else(|| error("`for` must be a plain name".into()))?;
            self.wanted.push((format!("{within}/{wanted}"), at));
            label_for = Some(UiId::new(within, wanted));
        }

        let is_form = tag == Tag::Form;
        let id = match own_id {
            Some(_) if is_form => None,
            Some(own) => {
                let needs = "an `id` needs a `<form>` round it, or a `form`";
                let id = UiId::new(form.ok_or_else(|| error(needs.into()))?, own);
                if !self.ids.insert(id.to_string()) {
                    return Err(error(format!("`{id}` is used twice")));
                }
                Some(id)
            }
            None => None,
        };
        let within = Inside {
            form: if is_form { own_id } else { inside.form },
            parent: Some(tag),
        };
        let children = if tag == Tag::Template {
            // A template's ids are its own: each copy gets them with its number.
            let mut copy = Checker::default();
            let children = copy.nodes(raw.children, within)?;
            copy.labels()?;
            children
        } else {
            self.nodes(raw.children, within)?
        };
        Ok(Checked {
            tag,
            id,
            form: own_id.filter(|_| is_form),
            label_for,
            attrs,
            children,
        })
    }

    /// Every label's `for` must name something here.
    fn labels(&self) -> Result<(), MarkupError> {
        match self.wanted.iter().find(|(id, _)| !self.ids.contains(id)) {
            Some((id, at)) => Err(MarkupError::new(
                *at,
                format!("`for` names `{id}`, which isn't here"),
            )),
            None => Ok(()),
        }
    }
}

/// An `id` or a form's name: written plainly, with nothing an id is split by.
fn plain(name: &str, value: Option<Cow<'static, str>>) -> Result<&'static str, String> {
    match value {
        Some(Cow::Borrowed(plain))
            if !plain.is_empty() && !plain.contains([' ', '/', '#', '&']) =>
        {
            Ok(plain)
        }
        _ => Err(format!("`{name}` must be a plain name")),
    }
}

/// Whether these may be inside a `tag` element, or where and why not.
fn fits(tag: Tag, children: &[RawNode]) -> Result<(), (Place, String)> {
    let name = tag.name();
    for child in children {
        let (at, held) = match child {
            RawNode::Element(inner) => (inner.at, format!("`<{}>`", inner.name)),
            RawNode::Text(_, at) => (*at, "words".to_owned()),
        };
        let allowed = match (tag.holds(), child) {
            (Holds::Anything, _) | (Holds::Words | Holds::WordsOrPicture, RawNode::Text(..)) => {
                true
            }
            // One picture, and nothing else beside it.
            (Holds::WordsOrPicture, RawNode::Element(inner)) => {
                inner.name == "img" && children.len() == 1
            }
            (Holds::Only(tags), RawNode::Element(inner)) => {
                Tag::named(inner.name).is_some_and(|inner| tags.contains(&inner))
            }
            (Holds::Nothing, _) => {
                return Err((
                    at,
                    format!("`<{name}>` holds nothing: write it `<{name}/>`"),
                ));
            }
            _ => false,
        };
        if !allowed {
            return Err((at, format!("`<{name}>` can't hold {held}")));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
