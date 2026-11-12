//! How high the wilderness's cells are, for walking and drawing, for the level
//! the player is on (`shared::movement::WildHeights`).

use bevy::prelude::*;
use shared::movement::{FlatGround, WildHeights};

/// How high a 16th of an elevation is drawn, in cells: one step of height
/// (owner's choice, 2026-11-08: 1/16 from screenshots, then 1/8 after playing).
pub const DRAWN_PER_16TH: f32 = 0.125;

/// The heights of the wilderness level's cells, worked out by `player` from the
/// world map's ground and the flats the server told of, whenever the level is
/// the wilderness and its window moves; `None` anywhere else. The drawn ground
/// (`DrawnGround`, which moves are priced on) is made from them, and moves
/// there are slowed by their terrain, as the server's are.
#[derive(Resource, Default)]
pub struct WildCellHeights {
    /// The level's bounds they were worked out for.
    pub bounds: (i64, i64, i64, i64),
    /// The flats they were worked out with (`CurrentDungeon::flats`).
    pub flats: Vec<FlatGround>,
    pub heights: Option<WildHeights>,
    /// The height drawn at 0, in 16ths: the window's middle cell's, so what is
    /// drawn stays near the origin's height (far up, the depth buffer and
    /// floats lose precision and the ground shows through tokens).
    pub reference: i32,
}

impl WildCellHeights {
    /// The terrain of a cell (`WildHeights::terrain`), if known.
    pub fn terrain(&self, (x, y): (i64, i64)) -> Option<shared::movement::Terrain> {
        self.heights.as_ref()?.terrain(x, y)
    }
}
