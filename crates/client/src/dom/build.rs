//! Building checked markup into entities. Only what never changes is set
//! here; each element's `Node` and looks are `restyle`'s, which sees every
//! new `Element`.

use bevy::prelude::*;

use super::attrs::Align;
use super::check::{Checked, CheckedNode};
use super::parts::{
    Canvas, Document, DocumentOf, Element, LabelFor, TemplateBody, TextStyle, Words,
};
use super::pictures::Pictures;
use super::tag::Tag;
use super::theme::Theme;
use crate::widgets::UiId;

mod progress;
mod surfaces;
pub(super) mod table;
mod title_row;

/// What an element being built is in.
#[derive(Clone, Copy)]
pub(super) struct Inside {
    pub style: TextStyle,
    /// Which copy of a template it is part of: its ids get `.<index>`.
    pub index: Option<usize>,
    /// The root of the document it is in.
    pub document: Entity,
    /// It is in a dialog's `footer`: its row of buttons.
    pub footer: bool,
    /// Which of several documents alike it is in: its ids get `#<id>`.
    pub which: Option<shared::id::Id>,
}

impl Inside {
    /// An id as it is here: in this one of several documents alike, and
    /// numbered in a copy of a template.
    pub fn named(&self, id: UiId) -> UiId {
        let id = self.which.map_or(id, |which| id.in_one(which));
        self.index.map_or(id, |index| id.at(index))
    }
}

pub(super) struct Builder<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub theme: &'a Theme,
    pub pictures: &'a Pictures,
    pub assets: &'a AssetServer,
}

impl Builder<'_, '_, '_> {
    /// Builds `nodes` as the last children of `parent`.
    pub fn nodes(&mut self, parent: Entity, nodes: &[CheckedNode], inside: Inside) {
        for node in nodes {
            match node {
                CheckedNode::Element(element) => self.element(parent, element, inside),
                CheckedNode::Text(words) => {
                    self.words(parent, words, inside.style);
                }
            }
        }
    }

    pub fn words(&mut self, parent: Entity, words: &str, style: TextStyle) -> Entity {
        let text = (
            Text::new(words),
            style.size.font(),
            TextColor(style.colour),
            TextLayout::justify(style.justify),
        );
        self.commands.spawn((text, Words, ChildOf(parent))).id()
    }

    fn element(&mut self, parent: Entity, element: &Checked, inside: Inside) {
        if Self::is_surface(element) {
            return self.surface(parent, element, inside);
        }
        let (entity, within) = self.shell(parent, element, inside);
        match element.tag {
            Tag::Template => {
                let body = TemplateBody(element.children.clone());
                self.commands.entity(entity).insert(body);
            }
            Tag::Table => self.table(entity, element, within),
            Tag::Progress => self.progress(entity, element, within.style),
            // A grid or slot of things is its owner's to draw in, as a canvas.
            Tag::Canvas | Tag::ObjectGrid | Tag::ObjectSlot => {
                self.commands.entity(entity).insert(Canvas);
            }
            Tag::Object => {
                let shown = crate::dom::objects::ObjectShown::default();
                self.commands.entity(entity).insert(shown);
            }
            Tag::Panel => self.fixed_panel(entity, element, within),
            tag if tag.is_control() => self.control(entity, element, within),
            _ => self.nodes(entity, &element.children, within),
        }
    }

    /// An element with nothing in it yet, and what things in it are inside.
    fn shell(&mut self, parent: Entity, element: &Checked, inside: Inside) -> (Entity, Inside) {
        let class = element.attrs.class.as_deref();
        let look = self.theme.look(element.tag, class);
        let style = TextStyle {
            size: look.text.unwrap_or(inside.style.size),
            colour: look.colour.unwrap_or(inside.style.colour),
            justify: match element.attrs.align {
                Some(Align::Left) => Justify::Left,
                Some(Align::Center) => Justify::Center,
                Some(Align::Right) => Justify::Right,
                None => inside.style.justify,
            },
        };
        let part = Element {
            tag: element.tag,
            attrs: element.attrs.clone(),
        };
        let mut entity = self
            .commands
            .spawn((part, Node::default(), style, ChildOf(parent)));
        let numbered = |id: UiId| inside.named(id);
        // A control's or surface's id goes on the widget that shows it.
        let on_widget = element.tag.is_control() || Self::is_surface(element);
        if let Some(id) = element.id.filter(|_| !on_widget) {
            entity.insert(numbered(id));
        }
        if let Some(wanted) = element.label_for {
            entity.insert(LabelFor(numbered(wanted)));
        }
        let footer = inside.footer || element.tag == Tag::Footer;
        let within = Inside {
            style,
            footer,
            ..inside
        };
        (entity.id(), within)
    }
}

/// Builds each new document from its template. One with something wrong in it
/// is left empty, with the error in the log.
/// A document to build: its root, its template, what it is in (if it is
/// inside another's element) and which of several alike it is.
type NewDocument = (
    Entity,
    &'static Document,
    Option<&'static ChildOf>,
    Option<&'static DocumentOf>,
);

pub(super) fn build_documents(
    mut commands: Commands,
    theme: Res<Theme>,
    pictures: Res<Pictures>,
    assets: Res<AssetServer>,
    new: Query<NewDocument, Added<Document>>,
    styles: Query<&TextStyle>,
) {
    for (root, Document(template), parent, which) in &new {
        match template.checked() {
            Ok(nodes) => {
                // Inside another document's element, it writes as that does.
                let around = parent.and_then(|parent| styles.get(parent.parent()).ok());
                let style = around.copied().unwrap_or(TextStyle {
                    size: theme.text,
                    colour: theme.colour,
                    justify: Justify::Left,
                });
                commands.entity(root).insert(style);
                let mut builder = Builder {
                    commands: &mut commands,
                    theme: &theme,
                    pictures: &pictures,
                    assets: &assets,
                };
                let inside = Inside {
                    style,
                    index: None,
                    document: root,
                    footer: false,
                    which: which.map(|DocumentOf(which)| *which),
                };
                builder.nodes(root, &nodes, inside);
            }
            Err(error) => error!("markup `{}`:{error}", template.name),
        }
    }
}

#[cfg(test)]
pub(super) mod tests;
