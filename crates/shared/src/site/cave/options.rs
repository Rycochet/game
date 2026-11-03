//! What a cave poi asks for: how it is entered and its size.

use serde::{Deserialize, Serialize};

/// Which ways in a cave may have.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaveEntrance {
    /// A hole in the ground: stairs down from the world map.
    Hole,
    /// A cave mouth: walk in from outside ground at the edge.
    Opening,
    /// Either, half the time each.
    Either,
}

/// Options for the cave generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaveOptions {
    /// Each level's width and height are picked separately from this
    /// inclusive range.
    pub size: (i64, i64),
    /// How many levels deep, inclusive range.
    pub levels: (u32, u32),
    pub entrance: CaveEntrance,
    /// Dug dungeons have doors where passages enter caves (some secret);
    /// natural caves have none.
    pub doors: bool,
    /// Percentage chance of water pools / a lava pool on each level.
    pub water_percent: u32,
    pub lava_percent: u32,
}
