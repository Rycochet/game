//! Where fake mouse input goes: a point in the window's logical pixels, or a
//! widget by id (`client.ui.list`'s ids), so a caller needn't read pixels off
//! a screenshot, which is in physical pixels and may be shrunk when viewed.

use bevy::prelude::*;
use server::debug_api::{BrpResult, refused};

use crate::widgets::WidgetPlaces;

/// A point asked for as `x`, `y` (logical pixels) or as a widget's `id`, with
/// `x`, `y` then measured from the widget's top left (its centre without
/// them). `None`: nothing was asked for.
pub(super) fn aim(
    places: &WidgetPlaces,
    id: Option<&str>,
    x: Option<f32>,
    y: Option<f32>,
) -> BrpResult<Option<Vec2>> {
    let xy = match (x, y) {
        (Some(x), Some(y)) => Some(Vec2::new(x, y)),
        (None, None) => None,
        _ => return Err(refused("give both x and y, or neither")),
    };
    match id {
        Some(id) => places.point(id, xy).map(Some),
        None => Ok(xy),
    }
}

/// As `aim`, when a point is needed.
pub(super) fn aim_needed(
    places: &WidgetPlaces,
    id: Option<&str>,
    x: Option<f32>,
    y: Option<f32>,
) -> BrpResult<Vec2> {
    aim(places, id, x, y)?.ok_or_else(|| refused("give x and y, or an id"))
}
