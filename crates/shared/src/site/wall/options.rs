//! What a building asks of a wall round it.

use super::WallMaterial;

/// Options for a wall round a building (`BuildingOptions::wall`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WallOptions {
    /// Percentage chance that there is one.
    pub percent: u32,
    /// Ground left between the building and the wall, rolled for each side
    /// (an inclusive range).
    pub gap: (i64, i64),
    /// How big a pond inside the wall is, each way (an inclusive range), or
    /// none.
    pub pond: Option<(i64, i64)>,
    /// What it may be made of, and how likely each is.
    pub materials: &'static [(WallMaterial, u32)],
}
