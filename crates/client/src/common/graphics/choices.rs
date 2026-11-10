//! What the Graphics settings with more than on and off can be.

use serde::{Deserialize, Serialize};

use super::IN_BROWSER;

/// The most frames drawn each second.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FrameRate {
    Fps30,
    Fps60,
    Fps120,
    Unlimited,
}

impl FrameRate {
    pub const ALL: [Self; 4] = [Self::Fps30, Self::Fps60, Self::Fps120, Self::Unlimited];

    pub fn name(self) -> &'static str {
        match self {
            Self::Fps30 => "30",
            Self::Fps60 => "60",
            Self::Fps120 => "120",
            Self::Unlimited => "Unlimited",
        }
    }

    /// Frames a second, or `None` for as many as can be drawn.
    pub fn per_second(self) -> Option<f64> {
        match self {
            Self::Fps30 => Some(30.0),
            Self::Fps60 => Some(60.0),
            Self::Fps120 => Some(120.0),
            Self::Unlimited => None,
        }
    }
}

/// Smoothing of jagged edges (MSAA).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AntiAliasing {
    Off,
    /// Not in a browser, whose graphics offer only 4× or none.
    Times2,
    Times4,
}

impl AntiAliasing {
    pub const ALL: [Self; 3] = [Self::Off, Self::Times2, Self::Times4];

    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Times2 => "2×",
            Self::Times4 => "4×",
        }
    }

    /// Whether this build can use it.
    pub fn usable(self) -> bool {
        !(IN_BROWSER && self == Self::Times2)
    }

    /// What is used: in a browser 2× becomes 4×.
    pub fn in_use(self) -> Self {
        if self.usable() { self } else { Self::Times4 }
    }
}

/// What happens to the drawing while another window is in front.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Background {
    /// A frame a second, to save power.
    SlowDown,
    /// As when in front: for streaming, or watching on another screen.
    KeepDrawing,
}

impl Background {
    pub const ALL: [Self; 2] = [Self::SlowDown, Self::KeepDrawing];

    pub fn name(self) -> &'static str {
        match self {
            Self::SlowDown => "Slow down",
            Self::KeepDrawing => "Keep drawing",
        }
    }
}

/// The world map's shadows.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shadows {
    Off,
    /// A smaller shadow map: softer edges.
    Low,
    High,
}

impl Shadows {
    pub const ALL: [Self; 3] = [Self::Off, Self::Low, Self::High];

    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Low => "Low",
            Self::High => "High",
        }
    }
}

/// How much of the world map's land is drawn round the player.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewDistance {
    /// Three quarters of the usual window.
    Near,
    /// The usual window, smaller when zoomed in so it keeps its size on screen.
    Normal,
    /// The whole window however far in the camera zooms.
    Far,
}

impl ViewDistance {
    pub const ALL: [Self; 3] = [Self::Near, Self::Normal, Self::Far];

    pub fn name(self) -> &'static str {
        match self {
            Self::Near => "Near",
            Self::Normal => "Normal",
            Self::Far => "Far",
        }
    }
}

/// How much rain, snow and hail is drawn.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Precipitation {
    Off,
    /// Half as many drops.
    Light,
    Full,
}

impl Precipitation {
    pub const ALL: [Self; 3] = [Self::Off, Self::Light, Self::Full];

    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Light => "Light",
            Self::Full => "Full",
        }
    }

    /// The share of drops drawn.
    pub fn share(self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Light => 0.5,
            Self::Full => 1.0,
        }
    }
}

/// How places seen before, but not now, are drawn in sites.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Remembered {
    Show,
    /// Darker, so what is in sight stands out.
    Dim,
    Hide,
}

impl Remembered {
    pub const ALL: [Self; 3] = [Self::Show, Self::Dim, Self::Hide];

    pub fn name(self) -> &'static str {
        match self {
            Self::Show => "Show",
            Self::Dim => "Dim",
            Self::Hide => "Hide",
        }
    }
}
