//! A finished town's buildings: where each stands and how it looks.

use serde::{Deserialize, Serialize};

use super::{BuildingKind, RoofKind, WallKind};

/// A rectangle of cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

/// What anyone can see of a building (players know a town's layout at once).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingLook {
    /// Its `RoomId` on the level.
    pub id: u16,
    pub kind: BuildingKind,
    pub walls: WallKind,
    pub roof: RoofKind,
    /// The cells it covers, as rectangles that don't overlap.
    pub rects: Vec<Rect>,
    /// Its door's cell, and the way the door faces, as a grid step.
    pub door: (i64, i64),
    pub faces: (i64, i64),
}

/// A building of a town. The numbers stay on the server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TownBuilding {
    pub look: BuildingLook,
    /// How good it is, from 0 to 1000 (for 0 to 1).
    pub quality: u16,
    /// How many trades it has made when generated, from 10 to 1000, higher
    /// in bigger towns. Players' trades add to it (kept by the server).
    pub trades: u32,
}
