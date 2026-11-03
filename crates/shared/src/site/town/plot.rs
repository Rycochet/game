//! Plots that can have something put beside them, and where one stands.

/// A building or garden on a road, which more can be put beside.
#[derive(Clone, Copy, Debug)]
pub(super) struct Plot {
    pub(super) road: usize,
    pub(super) side: i64,
    pub(super) along: (i64, i64),
    pub(super) deep: i64,
}

/// A way to put something beside a plot.
#[derive(Clone, Copy, Debug)]
pub(super) enum Beside {
    /// Further along the road, lined up with it: `1` or `-1`.
    Along(i64),
    /// Across the road, lined up with one end of it.
    Across,
    /// On a new side road past its end: `1` or `-1`.
    Corner(i64),
    /// A house behind it, its door away from it.
    Behind,
}

/// A building being placed.
pub(super) struct Placed {
    pub(super) cells: Vec<(i64, i64)>,
    pub(super) door: (i64, i64),
    pub(super) faces: (i64, i64),
    /// Whether its door opens onto a road, so it can be more than a house.
    pub(super) on_road: bool,
}

/// A plot's corner on a road, and the ways along the road and back from it.
pub(super) struct Frame {
    pub(super) corner: (i64, i64),
    pub(super) along: (i64, i64),
    pub(super) inward: (i64, i64),
}

impl Frame {
    /// The cell `a` along the road and `d` back from it.
    pub(super) fn at(&self, a: i64, d: i64) -> (i64, i64) {
        (
            self.corner.0 + self.along.0 * a + self.inward.0 * d,
            self.corner.1 + self.along.1 * a + self.inward.1 * d,
        )
    }

    /// The way out towards the road.
    pub(super) fn out(&self) -> (i64, i64) {
        (-self.inward.0, -self.inward.1)
    }
}
