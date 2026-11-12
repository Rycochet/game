//! Where a widget is, by id, for the debug API's fake mouse (`dev_tools`):
//! `client.click {id}` and the like aim at it without a caller working out
//! window pixels from a screenshot (which a viewer may have shrunk).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use server::debug_api::{BrpResult, refused};

use super::super::logical_rect;
use super::Widgets;

/// Finds widgets on show by id, as `client.ui.*` does.
#[derive(SystemParam)]
pub struct WidgetPlaces<'w, 's> {
    widgets: Widgets<'w, 's>,
}

impl WidgetPlaces<'_, '_> {
    /// A point on the one widget on show called `id`, in the window's logical
    /// pixels (what `client.click` takes): `from_corner` from its top left, or
    /// its centre. Refused as `client.ui.get` refuses: missing, hidden,
    /// covered or more than one.
    pub fn point(&self, id: &str, from_corner: Option<Vec2>) -> BrpResult<Vec2> {
        let entity = self.widgets.find(id)?;
        let Ok((_, _, node, transform, _)) = self.widgets.ids.get(entity) else {
            return Err(refused(format!("no widget is called {id}")));
        };
        let (corner, size) = logical_rect(node, transform);
        let scale = self.widgets.scale.0;
        let inside = from_corner.unwrap_or(size * scale / 2.0);
        Ok(corner * scale + inside)
    }
}

#[cfg(test)]
mod tests;
