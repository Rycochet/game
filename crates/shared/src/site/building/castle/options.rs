//! What a castle asks for round its keep.

use super::super::super::wall::WallMaterial;

/// Options for a castle round a building, its keep (`BuildingOptions::castle`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastleOptions {
    /// The rectangle its outer wall runs round, the wall included, as
    /// inclusive ranges. The keep is what is left inside, past the gaps.
    pub width: (i64, i64),
    pub height: (i64, i64),
    /// Ground between the wall and the keep, rolled for each side (an
    /// inclusive range).
    pub gap: (i64, i64),
    /// How big the pond inside the wall is, each way (an inclusive range).
    pub pond: (i64, i64),
    /// Percentage chance of a moat round the outside, and how wide it is (an
    /// inclusive range), one wider for castles whose wall's width and height
    /// add up to at least `wide_moat_from`.
    pub moat_percent: u32,
    pub moat: (i64, i64),
    pub wide_moat_from: i64,
    /// How many floors its towers have (an inclusive range); those by the
    /// gate have the fewest.
    pub tower_floors: (u32, u32),
    /// What its wall may be made of, and how likely each is.
    pub materials: &'static [(WallMaterial, u32)],
}
