//! `button`, and `input type="checkbox"` (a switch with `switch`).

use bevy::prelude::*;

use super::{state_of, words_in};
use crate::dom::attrs::Attrs;
use crate::dom::build::Builder;
use crate::dom::check::{Checked, CheckedNode};
use crate::dom::pictures::picture_for;
use crate::widgets::{
    BUTTON_PADDING, ButtonStyle, HoldButton, Pressable, TextSize, spawn_button, spawn_check_button,
    spawn_icon_button, spawn_small_icon_button, spawn_switch,
};

/// An icon button, across, with no size given.
const ICON_BUTTON: f32 = 32.0;

/// How much a button stands out, by its `class`.
fn style_of(class: Option<&str>) -> Option<ButtonStyle> {
    match class {
        Some("main") => Some(ButtonStyle::Main),
        Some("quiet") => Some(ButtonStyle::Quiet),
        Some("danger") => Some(ButtonStyle::Danger),
        Some("folder") => Some(ButtonStyle::Folder),
        _ => None,
    }
}

impl Builder<'_, '_, '_> {
    /// A button showing its words, or its one picture (`class="small"` for
    /// the small kind on the edge of something). `hold`: pressed only while
    /// held.
    pub(super) fn button(
        &mut self,
        holder: Entity,
        element: &Checked,
        size: TextSize,
        node: Node,
    ) -> Entity {
        let attrs = &element.attrs;
        let class = attrs.class.as_deref();
        let state = state_of(attrs);
        // An icon button is square: as tall as the row, unless it says.
        let side = match node.min_height {
            Val::Px(row) => row,
            _ => ICON_BUTTON,
        };
        let sized = (attrs.width.is_some(), attrs.height.is_some());
        let src = element.children.iter().find_map(|node| match node {
            CheckedNode::Element(picture) => picture.attrs.text("src"),
            CheckedNode::Text(_) => None,
        });
        let picture = src.and_then(|src| picture_for(src, self.pictures, self.assets));
        let words = words_in(element);
        let (hold, style) = (attrs.has("hold"), style_of(class));
        self.spawn_in(holder, |inside| {
            let mut button = match picture {
                Some((icon, _)) if class == Some("small") => {
                    spawn_small_icon_button(inside, state, icon, node)
                }
                Some((icon, _)) => {
                    let node = Node {
                        width: if sized.0 { percent(100) } else { px(side) },
                        height: if sized.1 { percent(100) } else { px(side) },
                        min_height: Val::Auto,
                        min_width: Val::Auto,
                        ..node
                    };
                    spawn_icon_button(inside, state, icon, node)
                }
                None => {
                    let node = Node {
                        padding: BUTTON_PADDING,
                        ..node
                    };
                    spawn_button(inside, state, words, size, node)
                }
            };
            if hold {
                button.insert(HoldButton::default());
            } else {
                button.insert(Pressable);
            }
            if let Some(style) = style {
                button.insert(style);
            }
            button.id()
        })
    }

    /// A checkbox, or a switch.
    pub(super) fn check(
        &mut self,
        holder: Entity,
        attrs: &Attrs,
        size: TextSize,
        node: Node,
    ) -> Entity {
        let (state, on) = (state_of(attrs), attrs.has("checked"));
        let switch = attrs.has("switch");
        // `label`: words in the button, beside its box.
        let label = attrs.text("label").unwrap_or_default().to_owned();
        // Only the box, with no words: its own size.
        let node = match label.is_empty() {
            true => Node::default(),
            false => node,
        };
        self.spawn_in(holder, |inside| {
            if switch {
                return spawn_switch(inside, state, on, Node::default()).id();
            }
            let mut check = spawn_check_button(inside, state, Some(on), label, size, node);
            check.insert(Pressable).id()
        })
    }
}
