//! Building checked markup into entities. Only what never changes is set
//! here; each element's `Node` and looks are `restyle`'s, which sees every
//! new `Element`.

use bevy::prelude::*;

use super::check::{Checked, CheckedNode};
use super::parts::{Canvas, Document, Element, LabelFor, TemplateBody, TextStyle, Words};
use super::pictures::Pictures;
use super::tag::Tag;
use super::theme::Theme;
use crate::widgets::UiId;

mod progress;
mod table;

/// What an element being built is in.
#[derive(Clone, Copy)]
pub(super) struct Inside {
    pub style: TextStyle,
    /// Which copy of a template it is part of: its ids get `.<index>`.
    pub index: Option<usize>,
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
        let text = (Text::new(words), style.size.font(), TextColor(style.colour));
        self.commands.spawn((text, Words, ChildOf(parent))).id()
    }

    fn element(&mut self, parent: Entity, element: &Checked, inside: Inside) {
        let (entity, within) = self.shell(parent, element, inside);
        match element.tag {
            Tag::Template => {
                let body = TemplateBody(element.children.clone());
                self.commands.entity(entity).insert(body);
            }
            Tag::Table => self.table(entity, element, within),
            Tag::Progress => self.progress(entity, element, within.style),
            Tag::Canvas => {
                self.commands.entity(entity).insert(Canvas);
            }
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
        };
        let part = Element {
            tag: element.tag,
            attrs: element.attrs.clone(),
        };
        let mut entity = self
            .commands
            .spawn((part, Node::default(), style, ChildOf(parent)));
        let numbered = |id: UiId| inside.index.map_or(id, |index| id.at(index));
        // A control's id goes on the widget that shows it (`controls`).
        if let Some(id) = element.id.filter(|_| !element.tag.is_control()) {
            entity.insert(numbered(id));
        }
        if let Some(wanted) = element.label_for {
            entity.insert(LabelFor(numbered(wanted)));
        }
        (entity.id(), Inside { style, ..inside })
    }
}

/// Builds each new document from its template. One with something wrong in it
/// is left empty, with the error in the log.
pub(super) fn build_documents(
    mut commands: Commands,
    theme: Res<Theme>,
    pictures: Res<Pictures>,
    assets: Res<AssetServer>,
    new: Query<(Entity, &Document), Added<Document>>,
) {
    for (root, Document(template)) in &new {
        match template.checked() {
            Ok(nodes) => {
                let style = TextStyle {
                    size: theme.text,
                    colour: theme.colour,
                };
                commands.entity(root).insert(style);
                let mut builder = Builder {
                    commands: &mut commands,
                    theme: &theme,
                    pictures: &pictures,
                    assets: &assets,
                };
                builder.nodes(root, &nodes, Inside { style, index: None });
            }
            Err(error) => error!("markup `{}`:{error}", template.name),
        }
    }
}

#[cfg(test)]
pub(super) mod tests;
