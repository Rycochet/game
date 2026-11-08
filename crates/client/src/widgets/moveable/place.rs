//! Keeping panels on screen, also when the window changes.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{MoveablePanel, PanelMoved, PanelPlace};
use crate::common::interface_area;

/// Where a panel of this size goes, wanted at `at`: wholly on the screen if it
/// fits, otherwise with its top left on the screen, for the title bar.
pub(super) fn on_screen(at: Vec2, size: Vec2, window: Vec2) -> Vec2 {
    at.min(window - size).max(Vec2::ZERO)
}

/// Where something at `at` belongs once the window has changed size: the same
/// share of the way across it.
pub(super) fn in_new_window(at: Vec2, old: Vec2, new: Vec2) -> Vec2 {
    at / old * new
}

/// Puts panels where they are wanted, as far as the window allows, and says
/// where one a drag let go of ended up.
pub(in crate::widgets) fn place_panels(
    window: Single<&Window, With<PrimaryWindow>>,
    scale: Res<UiScale>,
    mut moved: MessageWriter<PanelMoved>,
    mut panels: Query<(
        Entity,
        &mut MoveablePanel,
        &ComputedNode,
        &mut Node,
        &mut Visibility,
    )>,
) {
    let window = interface_area(&window, &scale);
    // Minimised: nowhere to be, so everything stays as it is for coming back.
    if window.min_element() < 1.0 {
        return;
    }
    for (panel, mut moveable, computed, mut node, mut visibility) in &mut panels {
        // Not laid out yet, so it can't be placed: it stays hidden until it can.
        if computed.size() == Vec2::ZERO {
            continue;
        }
        visibility.set_if_neq(Visibility::Inherited);
        if let Some(old) = moveable.window.replace(window) {
            moveable.at = in_new_window(moveable.at, old, window);
        }
        let size = computed.size() * computed.inverse_scale_factor;
        if let Some(first) = moveable.first.take() {
            moveable.at = match first {
                PanelPlace::TopLeft(at) => at,
                PanelPlace::BottomRight(gap) => window - size - gap,
                PanelPlace::Share(share) => share * window,
            };
        }
        let at = on_screen(moveable.at, size, window);
        if moveable.at != at {
            moveable.at = at;
        }
        if moveable.dropped {
            moveable.dropped = false;
            let share = at / window;
            moved.write(PanelMoved { panel, share });
        }
        let (left, top) = (px(at.x), px(at.y));
        if node.left != left || node.top != top {
            node.left = left;
            node.top = top;
        }
    }
}
