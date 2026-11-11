//! What the panels' contents share.

use bevy::prelude::*;

/// Takes a button out of its row, or puts it back: for those the player has
/// no permission for, which are hidden rather than greyed (owner).
pub(super) fn show_if(node: &mut Node, shown: bool) {
    let display = if shown { Display::Flex } else { Display::None };
    if node.display != display {
        node.display = display;
    }
}
