//! Nodes for item grids: at a cell, covering, and where a node is on screen.

use bevy::prelude::*;
use bevy::ui::{CalculatedClip, UiGlobalTransform};

use super::{CELL, CELL_BORDER};

/// A node one cell across at this place in a grid.
pub fn at_cell(x: f32, y: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(x * CELL),
        top: px(y * CELL),
        width: px(CELL),
        height: px(CELL),
        border: UiRect::all(px(CELL_BORDER)),
        ..default()
    }
}

/// A node filling a grid, for its children to be placed in.
pub fn covering() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px(0),
        width: percent(100),
        height: percent(100),
        ..default()
    }
}

/// Where a node is on the screen, in the interface's pixels (`UiPointer`'s, as
/// `Node`s are sized in): its top left and size.
pub fn logical_rect(node: &ComputedNode, transform: &UiGlobalTransform) -> (Vec2, Vec2) {
    let size = node.size() * node.inverse_scale_factor;
    let centre = transform.translation * node.inverse_scale_factor;
    (centre - size / 2.0, size)
}

/// Whether a node is at this point, in the interface's pixels (`UiPointer`'s), and
/// not scrolled out of sight.
pub fn shown_at(
    at: Vec2,
    node: &ComputedNode,
    transform: &UiGlobalTransform,
    clip: Option<&CalculatedClip>,
) -> bool {
    let physical = at / node.inverse_scale_factor;
    node.contains_point(*transform, physical)
        && clip.is_none_or(|clip| clip.contains_point(physical))
}
