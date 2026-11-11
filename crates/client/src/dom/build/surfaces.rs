//! Surfaces: `panel` (moveable by its title, or by a `grip`), `dialog` and
//! `scroll`. The widget that draws each is spawned first, and the element
//! goes inside it, holding what the markup puts in it. Panels that move and
//! dialogs are not laid out where they are written: they stand on their own
//! in front of the rest, in the layer the widgets give them.

use bevy::prelude::*;

use super::{Builder, Inside};
use crate::dom::attrs::{Attrs, Length};
use crate::dom::check::{Checked, CheckedNode};
use crate::dom::parts::{AlwaysBars, Control, ControlOf, SurfaceOf};
use crate::dom::tag::Tag;
use crate::widgets::{
    AutoScroll, PanelPlace, panel, panel_title, spawn_dialog, spawn_fullscreen_layer,
    spawn_grip_panel, spawn_moveable_panel, spawn_scroll_area,
};

/// Where a moving panel starts, from its corner, with no place given.
const START: f32 = 80.0;
/// How tall a scroll area may grow, with no height given: a share of the
/// window's height.
const SCROLL_SHARE: f32 = 80.0;

fn pixels(length: Option<Length>) -> Option<f32> {
    match length {
        Some(Length::Pixels(pixels)) => Some(pixels),
        _ => None,
    }
}

/// Where a panel that moves starts: by its `left` and `top`, or its `right`
/// and `bottom` from the window's far corner.
fn place_of(attrs: &Attrs) -> PanelPlace {
    match (pixels(attrs.right), pixels(attrs.bottom)) {
        (None, None) => {
            let corner = |side| pixels(side).unwrap_or(START);
            PanelPlace::TopLeft(Vec2::new(corner(attrs.left), corner(attrs.top)))
        }
        (right, bottom) => {
            PanelPlace::BottomRight(Vec2::new(right.unwrap_or(0.0), bottom.unwrap_or(0.0)))
        }
    }
}

impl Builder<'_, '_, '_> {
    /// Whether `element` is a surface built here, and not as other elements.
    pub(super) fn is_surface(element: &Checked) -> bool {
        match element.tag {
            Tag::Panel => element.attrs.has("moveable") || element.attrs.has("grip"),
            Tag::Dialog | Tag::Scroll => true,
            _ => false,
        }
    }

    pub(super) fn surface(&mut self, parent: Entity, element: &Checked, inside: Inside) {
        let attrs = &element.attrs;
        let title = attrs.title.as_deref().unwrap_or_default();
        // The widget carrying the id, what the element goes in, and where a
        // dialog's buttons go.
        let (widget, content, buttons) = match element.tag {
            Tag::Panel if attrs.has("grip") => {
                let (root, content) = spawn_grip_panel(self.commands, place_of(attrs));
                (root, content, None)
            }
            Tag::Panel => {
                let at = match place_of(attrs) {
                    PanelPlace::TopLeft(at) => at,
                    _ => Vec2::splat(START),
                };
                let (root, content) = spawn_moveable_panel(self.commands, title, at);
                (root, content, None)
            }
            Tag::Dialog if attrs.has("modal") => {
                let depth = attrs.number("depth").unwrap_or(0.0) as i32;
                let column = Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                };
                let (layer, panel) = spawn_fullscreen_layer(self.commands, depth, column);
                if !title.is_empty() {
                    self.commands.entity(panel).with_child(panel_title(title));
                }
                (layer, panel, None)
            }
            Tag::Dialog => {
                let depth = attrs.number("depth").unwrap_or(0.0) as i32;
                let dialog = spawn_dialog(self.commands, depth, title);
                (dialog.layer, dialog.content, Some(dialog.buttons))
            }
            _ => {
                let area = self.scroll_area(parent, attrs);
                (area, area, None)
            }
        };
        let (entity, within) = self.shell(content, element, inside);
        self.commands.entity(entity).insert(Control(widget));
        let mut made = self.commands.entity(widget);
        made.insert(ControlOf(entity));
        // A scroll area is inside its document, and goes with it by itself.
        if element.tag != Tag::Scroll {
            made.insert(SurfaceOf(inside.document));
        }
        if let Some(id) = element.id {
            made.insert(inside.index.map_or(id, |index| id.at(index)));
        }
        for node in &element.children {
            match (node, buttons) {
                // A dialog's `footer` is its row of buttons.
                (CheckedNode::Element(footer), Some(buttons)) if footer.tag == Tag::Footer => {
                    let (row, within) = self.shell(buttons, footer, within);
                    self.nodes(row, &footer.children, within);
                }
                _ => self.nodes(entity, std::slice::from_ref(node), within),
            }
        }
    }

    /// A scroll area in `parent`, as tall as what is in it up to its
    /// `max-height` (or `height`): pixels, or a share of the window's height.
    fn scroll_area(&mut self, parent: Entity, attrs: &Attrs) -> Entity {
        let most = match attrs.length("max-height").or(attrs.height) {
            Some(Length::Pixels(pixels)) => px(pixels),
            Some(Length::Percent(share)) => Val::Vh(share),
            None => Val::Vh(SCROLL_SHARE),
        };
        let mut area = Entity::PLACEHOLDER;
        self.commands
            .entity(parent)
            .with_children(|inside| area = spawn_scroll_area(inside, most));
        let mut made = self.commands.entity(area);
        if attrs.has("auto") {
            made.insert(AutoScroll::default());
        }
        if attrs.text("scrollbars") == Some("always") {
            made.insert(AlwaysBars);
        }
        area
    }

    /// A panel that stays where it is written: parchment, with its title.
    pub(super) fn fixed_panel(&mut self, entity: Entity, element: &Checked, inside: Inside) {
        self.commands.entity(entity).insert(panel());
        if let Some(title) = element.attrs.title.as_deref() {
            self.commands.entity(entity).with_child(panel_title(title));
        }
        self.nodes(entity, &element.children, inside);
    }
}
