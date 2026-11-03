//! The trading panels open, who keeps each shop, and who the player is walking to.

use bevy::prelude::*;
use shared::id::Id;
use shared::shop::DOOR_REACH;
use shared::site::BuildingLook;
use std::collections::BTreeMap;

use crate::common::MapPosition;
use crate::widgets::StockGrid;

/// A trading panel, with who it is with: never two for the same one.
pub(super) struct OpenTrade {
    pub(super) panel: Entity,
    /// What the player must stay near.
    pub(super) place: (i64, i64),
    /// What the grid goes in.
    pub(super) holder: Entity,
    pub(super) grid: StockGrid,
}

/// The panels open, in the order they were opened.
#[derive(Resource, Default)]
pub(super) struct OpenTrades(pub(super) Vec<OpenTrade>);

impl OpenTrades {
    pub(super) fn panel_with(&self, with: Id) -> Option<Entity> {
        let open = self.0.iter().find(|open| open.grid.with == with);
        open.map(|open| open.panel)
    }
}

/// Who keeps each shop on this level, by building.
#[derive(Resource, Default)]
pub(super) struct Keepers(pub(super) BTreeMap<u16, Id>);

/// The keeper the player is walking to, to open on arrival, and the cell in front
/// of their door.
#[derive(Resource, Default)]
pub(super) struct Heading(pub(super) Option<(Id, (i64, i64))>);

/// Whether someone here is near enough to this door. The server checks this too.
pub(super) fn beside(from: MapPosition, door: (i64, i64)) -> bool {
    (from.x - door.0).abs().max((from.y - door.1).abs()) <= DOOR_REACH
}

pub(super) fn front_of(building: &BuildingLook) -> (i64, i64) {
    (
        building.door.0 + building.faces.0,
        building.door.1 + building.faces.1,
    )
}
