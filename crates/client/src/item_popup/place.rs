//! Keeping item popups on the screen: one opened near an edge moves in, once
//! its size is known.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use bevy::window::PrimaryWindow;

use super::open::FORM;
use crate::common::interface_area;
use crate::dom::{Dom, Popup};
use crate::widgets::{UiId, logical_rect};

/// Moves any popup sticking out past the window's right or bottom edge back in.
pub(super) fn keep_on_screen(
    popups: Query<(&UiId, &ComputedNode, &UiGlobalTransform), With<Popup>>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut dom: Dom,
) {
    let area = interface_area(&window, &ui_scale);
    for (id, node, transform) in &popups {
        if id.parts().0 != FORM {
            continue;
        }
        let (corner, size) = logical_rect(node, transform);
        if size == Vec2::ZERO {
            continue;
        }
        let inside = corner.min(area - size).max(Vec2::ZERO).round();
        if inside != corner.round() {
            let form = id.form();
            dom.set(&format!("{form}/box"), "left", &inside.x.to_string());
            dom.set(&format!("{form}/box"), "top", &inside.y.to_string());
        }
    }
}
