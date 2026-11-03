//! View stats, and which way the map view is turned.

use bevy::prelude::*;

/// A line of performance info from the active view, shown by the UI.
#[derive(Resource, Default)]
pub struct ViewStats {
    pub summary: String,
}

/// Which way the map view is turned, as the compass direction nearest the top
/// of the screen, in quarter turns clockwise from north: 0 north (the default
/// view), 1 east, 2 south, 3 west. Written by the active view; read by
/// anything that turns screen directions into grid steps (the keyboard).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewFacing(pub u8);

impl ViewFacing {
    /// Turns a screen direction (`dy` −1 is up the screen, `dx` +1 right)
    /// into a grid step (`y` −1 is north, `x` +1 east).
    pub fn screen_to_grid(self, dx: i64, dy: i64) -> (i64, i64) {
        let (mut x, mut y) = (dx, dy);
        for _ in 0..self.0 % 4 {
            // A quarter turn clockwise: up becomes east, right becomes south.
            (x, y) = (-y, x);
        }
        (x, y)
    }
}
