//! Whether a map needs painting this frame, and how much of it: the land
//! under the marks is the costly part, so it is kept and reused when only the
//! sky round the minimap has moved on.

use super::DrawnFrom;

/// How often a map still filling in (its chunks' colours or rivers worked out
/// a little each frame) is painted again to show what has been added.
pub(super) const FILLING_SECONDS: f32 = 0.1;

/// What a frame's repaint is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Redraw {
    /// Nothing has changed.
    No,
    /// Only more of the map has been worked out: fill in more, and paint it
    /// now and then (`FILLING_SECONDS`).
    Filling,
    /// Only the clock: the minimap's suns and moons, or in sites what is
    /// remembered (which changes without telling).
    Clock,
    /// What it shows has changed: paint it all.
    All,
}

impl Redraw {
    /// Why this frame might paint, from what it was last drawn from (`last`)
    /// and what it would be drawn from now.
    pub(super) fn of(
        last: Option<&DrawnFrom>,
        now: &DrawnFrom,
        (filling, world_changed, clock_due): (bool, bool, bool),
    ) -> Self {
        if last != Some(now) || world_changed {
            Self::All
        } else if clock_due {
            Self::Clock
        } else if filling {
            Self::Filling
        } else {
            Self::No
        }
    }
}

/// The minimap's picture before its frame and sky went on, and what it was
/// drawn from, for a repaint on the clock with nothing else changed.
#[derive(Default)]
pub(super) struct KeptLand {
    from: Option<DrawnFrom>,
    pixels: Vec<u8>,
}

impl KeptLand {
    pub(super) fn keep(&mut self, from: DrawnFrom, pixels: &[u8]) {
        self.from = Some(from);
        self.pixels.clear();
        self.pixels.extend_from_slice(pixels);
    }

    /// Puts the kept land back into `pixels`, if it was drawn from `from` and
    /// is the same size: false if it can't.
    pub(super) fn restore(&self, from: &DrawnFrom, pixels: &mut [u8]) -> bool {
        let same = self.from.as_ref() == Some(from) && self.pixels.len() == pixels.len();
        if same {
            pixels.copy_from_slice(&self.pixels);
        }
        same
    }

    pub(super) fn forget(&mut self) {
        self.from = None;
    }
}

#[cfg(test)]
mod tests;
