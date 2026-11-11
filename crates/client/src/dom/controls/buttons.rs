//! `button`, and `input type="checkbox"` (a switch with `switch`).

use bevy::prelude::*;

use super::{node_in, state_of, words_in};
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
    pub(super) fn button(&mut self, holder: Entity, element: &Checked, size: TextSize) -> Entity {
        let attrs = &element.attrs;
        let class = attrs.class.as_deref();
        let (state, node) = (state_of(attrs), node_in(attrs, false));
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
                    // A picture is as big as its file unless the button says.
                    let side = |given: Val| match given {
                        Val::Auto => px(ICON_BUTTON),
                        given => given,
                    };
                    let node = Node {
                        width: side(node.width),
                        height: side(node.height),
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
    pub(super) fn check(&mut self, holder: Entity, attrs: &Attrs, size: TextSize) -> Entity {
        let (state, on) = (state_of(attrs), attrs.has("checked"));
        let switch = attrs.has("switch");
        // `label`: words in the button, beside its box.
        let label = attrs.text("label").unwrap_or_default().to_owned();
        let node = node_in(attrs, false);
        self.spawn_in(holder, |inside| {
            if switch {
                return spawn_switch(inside, state, on, Node::default()).id();
            }
            let mut check = spawn_check_button(inside, state, Some(on), label, size, node);
            check.insert(Pressable).id()
        })
    }
}
