//! Where a document over the game's view goes: along an edge of the window,
//! with no height of its own.

use bevy::prelude::*;

/// The node for a document whose pieces are placed from the top of the
/// window (`top`, `left`, `right` on them). It is as wide as the window and
/// takes no room, so nothing of it is ever in front of the game's view:
/// only the pieces themselves are.
pub fn top_strip() -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: px(0),
        ..strip()
    }
}

/// The same along the bottom of the window, for pieces placed from there
/// (`bottom`).
pub fn bottom_strip() -> Node {
    Node {
        position_type: PositionType::Absolute,
        bottom: px(0),
        ..strip()
    }
}

/// As wide as the window, stacked, so a form in it is as wide too.
fn strip() -> Node {
    Node {
        left: px(0),
        right: px(0),
        flex_direction: FlexDirection::Column,
        ..default()
    }
}
