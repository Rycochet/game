//! Max Resolution: the most lines the world is drawn with, whatever the
//! window's size, so a big window or full screen doesn't cost more to draw.

use serde::{Deserialize, Serialize};

/// The most lines (picture height) the world is drawn with. The interface is
/// always drawn at the window's full resolution.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MaxResolution {
    /// As big as the window.
    Off,
    P720,
    P1080,
    P1440,
}

impl MaxResolution {
    pub const ALL: [Self; 4] = [Self::Off, Self::P720, Self::P1080, Self::P1440];

    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::P720 => "720p",
            Self::P1080 => "1080p",
            Self::P1440 => "1440p",
        }
    }

    /// The most lines, or `None` for no limit.
    pub fn lines(self) -> Option<u32> {
        match self {
            Self::Off => None,
            Self::P720 => Some(720),
            Self::P1080 => Some(1080),
            Self::P1440 => Some(1440),
        }
    }

    /// The share of a window `height` physical pixels tall that keeps to this
    /// (1 when it already does).
    pub fn share_of(self, height: u32) -> f32 {
        match self.lines() {
            Some(lines) if height > lines => lines as f32 / height as f32,
            _ => 1.0,
        }
    }
}
