//! An element's Bevy `Node`, from its attributes and its look.

use bevy::prelude::*;

use super::attrs::{Align, Attrs, Length, VAlign};
use super::parts::InTable;
use super::tag::Tag;
use super::theme::Look;
use crate::widgets::PANEL_EDGE;

/// A progress bar's height, and a spinner's across, with no size given.
const PROGRESS: f32 = 24.0;
/// Between a fixed panel's edge and what is in it.
const PANEL_PADDING: f32 = 8.0;
/// The least a progress bar is across.
const PROGRESS_LEAST: f32 = 120.0;

/// What an element needs laying out by, beside its attributes.
pub(super) struct Around<'a> {
    pub look: &'a Look,
    pub spot: Option<&'a InTable>,
    /// It is a cell of a hidden row.
    pub row_hidden: bool,
    /// How tall a line of its writing is, in pixels.
    pub line: f32,
}

pub(super) fn node_for(tag: Tag, attrs: &Attrs, around: &Around) -> Node {
    // A dialog's size is its widget's.
    let sized = tag != Tag::Dialog;
    let mut node = Node {
        width: val(attrs.width.filter(|_| sized)),
        height: val(attrs.height.filter(|_| sized)),
        margin: margin(attrs, around.look),
        ..default()
    };
    // A panel that moves and a dialog are placed by their widgets.
    let moves = tag == Tag::Panel && (attrs.has("moveable") || attrs.has("grip"));
    let fixed_panel = tag == Tag::Panel && !moves;
    if attrs.is_placed() && !moves && tag != Tag::Dialog {
        node.position_type = PositionType::Absolute;
        node.top = val(attrs.top);
        node.left = val(attrs.left);
        node.right = val(attrs.right);
        node.bottom = val(attrs.bottom);
    }
    match tag {
        Tag::Table => {
            let columns = match around.spot {
                Some(InTable::Table { columns }) => *columns,
                _ => 1,
            };
            node.display = Display::Grid;
            node.grid_template_columns = RepeatedGridTrack::auto(columns);
            let spacing = px(attrs.number("cellspacing").unwrap_or(0.0));
            (node.row_gap, node.column_gap) = (spacing, spacing);
        }
        // As wide as what they are in, so they take a whole line of a flow.
        Tag::Br => (node.width, node.height) = (percent(100), px(0)),
        Tag::Hr => (node.width, node.height) = (percent(100), px(1)),
        Tag::Img if attrs.height.is_none() => node.height = px(around.line),
        Tag::Progress if attrs.has("value") => {
            node.min_width = px(PROGRESS_LEAST);
            node.flex_grow = 1.0;
            node.height = attrs
                .height
                .map_or(px(PROGRESS), |height| val(Some(height)));
        }
        Tag::Progress => {
            node.width = attrs.width.map_or(px(PROGRESS), |width| val(Some(width)));
            node.height = attrs
                .height
                .map_or(px(PROGRESS), |height| val(Some(height)));
        }
        // A control is as big as the widget in it; one that fills its room
        // takes what there is.
        _ if tag.is_control() => {
            let fills = match tag {
                Tag::Input => attrs.text("type") == Some("text"),
                _ => matches!(tag, Tag::Select | Tag::KeyBox | Tag::ColourPicker),
            };
            if fills && attrs.width.is_none() {
                node.flex_grow = 1.0;
                node.min_width = px(PROGRESS_LEAST);
            } else if attrs.width.is_none() {
                // Its own size: never squeezed by a neighbour that fills.
                node.flex_shrink = 0.0;
            }
        }
        // A dialog's buttons: in a row, at the right.
        Tag::Footer => {
            node.column_gap = px(around.look.gap);
            node.justify_content = JustifyContent::FlexEnd;
            node.align_items = AlignItems::Center;
        }
        _ if tag.is_inline() => flow_along(&mut node, attrs, around.look),
        _ => stack(&mut node, attrs, around.look),
    }
    // A dialog given a height has what is in it fill that height.
    if attrs.fills || (tag == Tag::Dialog && attrs.height.is_some()) {
        node.flex_grow = 1.0;
        node.min_height = px(0);
    }
    if fixed_panel {
        node.border = UiRect::all(px(PANEL_EDGE));
        node.padding = UiRect::all(px(PANEL_PADDING));
    }
    match around.spot {
        Some(InTable::Row { row, columns, .. }) => {
            node.grid_row = GridPlacement::start(*row as i16 + 1);
            node.grid_column = GridPlacement::start_span(1, *columns);
        }
        Some(InTable::Cell {
            row,
            column,
            span,
            padding,
            ..
        }) => {
            node.grid_row = GridPlacement::start(*row as i16 + 1);
            node.grid_column = GridPlacement::start_span(*column as i16 + 1, *span);
            node.padding = UiRect::all(px(*padding));
        }
        _ => {}
    }
    // A dialog or a panel that moves is hidden by its widget, which other
    // code may also show: what is in it is always laid out.
    let own_hidden = attrs.hidden && !moves && tag != Tag::Dialog;
    if own_hidden || around.row_hidden || tag == Tag::Template {
        node.display = Display::None;
    }
    node
}

/// A block: what is in it stacks from the top.
fn stack(node: &mut Node, attrs: &Attrs, look: &Look) {
    node.flex_direction = FlexDirection::Column;
    node.row_gap = px(look.gap);
    node.align_items = match attrs.align {
        Some(Align::Left) => AlignItems::FlexStart,
        Some(Align::Center) => AlignItems::Center,
        Some(Align::Right) => AlignItems::FlexEnd,
        None => AlignItems::Stretch,
    };
    node.justify_content = match attrs.valign {
        Some(VAlign::Middle) => JustifyContent::Center,
        Some(VAlign::Bottom) => JustifyContent::FlexEnd,
        Some(VAlign::Top) | None => JustifyContent::FlexStart,
    };
}

/// Inline: what is in it flows from the left, wrapping, as words do.
fn flow_along(node: &mut Node, attrs: &Attrs, look: &Look) {
    node.flex_direction = FlexDirection::Row;
    node.flex_wrap = match attrs.nowrap {
        true => FlexWrap::NoWrap,
        false => FlexWrap::Wrap,
    };
    // Never wider than what holds it, so long words wrap and don't run out.
    node.max_width = percent(100);
    node.column_gap = px(look.gap);
    node.justify_content = match attrs.align {
        Some(Align::Center) => JustifyContent::Center,
        Some(Align::Right) => JustifyContent::FlexEnd,
        Some(Align::Left) | None => JustifyContent::FlexStart,
    };
    node.align_items = match attrs.valign {
        Some(VAlign::Top) => AlignItems::FlexStart,
        Some(VAlign::Bottom) => AlignItems::FlexEnd,
        Some(VAlign::Middle) | None => AlignItems::Center,
    };
}

/// The room round an element: the theme's, but for the sides it gives itself.
fn margin(attrs: &Attrs, look: &Look) -> UiRect {
    let side = |at: usize| px(attrs.margin[at].unwrap_or(look.margin[at]));
    UiRect {
        top: side(0),
        right: side(1),
        bottom: side(2),
        left: side(3),
    }
}

fn val(length: Option<Length>) -> Val {
    match length {
        Some(Length::Pixels(pixels)) => px(pixels),
        Some(Length::Percent(share)) => percent(share),
        None => Val::Auto,
    }
}

#[cfg(test)]
mod tests;
