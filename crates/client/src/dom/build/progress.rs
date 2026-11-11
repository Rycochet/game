//! `progress`: a bar with words over it when it has a `value`, a spinner
//! when it has none, as in HTML.

use bevy::prelude::*;

use super::Builder;
use crate::dom::check::{Checked, CheckedNode};
use crate::dom::parts::{Bar, TextStyle, Words};
use crate::widgets::{Spinner, spawn_progress_bar};

impl Builder<'_, '_, '_> {
    pub(super) fn progress(&mut self, entity: Entity, element: &Checked, style: TextStyle) {
        if !element.attrs.has("value") {
            self.commands.entity(entity).insert(Spinner);
            return;
        }
        let words = element.children.iter().find_map(|node| match node {
            CheckedNode::Text(words) => Some(words.as_str()),
            CheckedNode::Element(_) => None,
        });
        let words = (
            Text::new(words.unwrap_or_default()),
            style.size.font(),
            TextColor(style.colour),
            Words,
        );
        let fills = Node {
            width: percent(100),
            height: percent(100),
            ..default()
        };
        let colours = self.theme.progress;
        let mut bar = None;
        self.commands.entity(entity).with_children(|inside| {
            bar = Some(spawn_progress_bar(inside, 0.0, colours, fills, words).id());
        });
        if let Some(bar) = bar {
            self.commands.entity(entity).insert(Bar(bar));
        }
    }
}
