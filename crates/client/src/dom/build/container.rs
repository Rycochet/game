//! `container`: the frame round a grid of things, the same wherever one is:
//! its `title` at the top left with its `header` (controls) at the top right,
//! then what it holds, then its `footer` (the weight at the left, slots at
//! the right).

use bevy::prelude::*;

use super::{Builder, Inside};
use crate::dom::check::{Checked, CheckedNode};
use crate::dom::parts::TextStyle;
use crate::dom::tag::Tag;
use crate::widgets::TextSize;

/// Between the title and the controls.
const TOP_GAP: f32 = 8.0;

impl Builder<'_, '_, '_> {
    /// Fills a container's element: the top row if it has a title or a
    /// header, what it holds, and its footer last, wherever they were written.
    pub(super) fn container(&mut self, entity: Entity, element: &Checked, inside: Inside) {
        let part = |tag: Tag| {
            element.children.iter().find_map(|node| match node {
                CheckedNode::Element(child) if child.tag == tag => Some(child),
                _ => None,
            })
        };
        let (header, footer) = (part(Tag::Header), part(Tag::Footer));
        let title = element.attrs.title.as_deref();
        if title.is_some() || header.is_some() {
            // The title at the left and the controls at the right; alone,
            // the controls keep to the right.
            let row = Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: px(TOP_GAP),
                ..default()
            };
            let top = self.commands.spawn((row, ChildOf(entity))).id();
            if let Some(title) = title {
                let style = TextStyle {
                    size: TextSize::Medium,
                    ..inside.style
                };
                self.words(top, title, style);
            }
            if let Some(header) = header {
                self.nodes(top, &[CheckedNode::Element(header.clone())], inside);
            }
        }
        for node in &element.children {
            let own_part = matches!(node, CheckedNode::Element(child)
                if matches!(child.tag, Tag::Header | Tag::Footer));
            if !own_part {
                self.nodes(entity, std::slice::from_ref(node), inside);
            }
        }
        if let Some(footer) = footer {
            self.nodes(entity, &[CheckedNode::Element(footer.clone())], inside);
        }
    }
}
