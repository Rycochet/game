//! What a building poi asks for: its outline and sizes.

use serde::{Deserialize, Serialize};

/// The shape of a building seen from above.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Outline {
    Rectangle,
    /// Corners cut off at 45°.
    CutCorners,
    /// Corners rounded off.
    RoundCorners,
    Circle,
}

/// Options for the building generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildingOptions {
    /// The ground floor's size, outer walls included, as inclusive ranges.
    /// A circle uses `width` for its diameter.
    pub width: (i64, i64),
    pub height: (i64, i64),
    /// Picked from evenly (repeat one to make it likelier).
    pub outlines: &'static [Outline],
    /// Square: the height is the width.
    pub square: bool,
    /// At least this many floors, whatever its size.
    pub min_floors: u32,
    /// At most this many floors.
    pub max_floors: u32,
    /// Every floor is the same size as the ground floor. Above `min_floors`,
    /// the top one may be about half the size.
    pub same_size: bool,
    /// Floors go on up, each smaller by this many cells per side (an inclusive
    /// range), until no room fits at the top: instead of the other floor rules.
    pub taper: Option<(i64, i64)>,
    /// Bigger buildings have more floors: up to one per this many cells of
    /// the ground floor's shorter side.
    pub size_per_floor: i64,
    /// How ruined it is, 0 (a normal building) to 5. The damage has its own
    /// seed, so the floorplan is the same at every level.
    pub ruin: u8,
    /// Percentage chance of a basement under the ground floor.
    pub basement_percent: u32,
    /// How big a basement is, if not the size of the ground floor: walls
    /// included, inclusive ranges, never smaller than the ground floor.
    pub basement_size: Option<((i64, i64), (i64, i64))>,
    /// Of the places in outside walls a window could go, what percentage have
    /// one, on every floor but a basement.
    pub windows_percent: u32,
    /// Whether the ground floor has them too.
    pub ground_windows: bool,
    /// Raised on pillars: the ground floor is only covered ground and lone
    /// pillars, with one staircase up to the floors above.
    pub stilts: bool,
}
