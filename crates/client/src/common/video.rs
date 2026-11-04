//! The Video settings that trade looks for speed, kept in `Preferences`, and
//! the presets that set them together. Each is used by the module that draws
//! what it changes; the window's own (display mode, resolution) are in
//! `display`.

use serde::{Deserialize, Serialize};

/// Whether this build runs in a browser, where some choices are the browser's.
pub const IN_BROWSER: bool = cfg!(target_arch = "wasm32");

/// Video settings for speed and clutter.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(default)]
pub struct VideoPrefs {
    /// The most frames drawn each second.
    pub frame_rate: FrameRate,
    /// Waits for the screen before showing a frame (native only).
    pub vsync: bool,
    /// How much of the window's resolution the world is drawn at, in percent
    /// (`RENDER_SCALES`); the interface is always drawn at full resolution.
    pub render_scale: u8,
    pub anti_aliasing: AntiAliasing,
    /// What happens to the drawing while another window is in front.
    pub background: Background,
}

impl Default for VideoPrefs {
    fn default() -> Self {
        Preset::Medium.prefs()
    }
}

impl VideoPrefs {
    /// The preset these are, if any. Choices no preset sets are left out.
    pub fn preset(&self) -> Option<Preset> {
        Preset::ALL
            .into_iter()
            .find(|preset| preset.prefs() == *self)
    }

    /// The render scale as a share of the window, `0.5..=1`.
    pub fn render_share(&self) -> f32 {
        f32::from(self.render_scale.clamp(RENDER_SCALES.0, RENDER_SCALES.1)) / 100.0
    }
}

/// The render scales offered, in percent: least and most.
pub const RENDER_SCALES: (u8, u8) = (50, 100);

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

/// Sets of Video settings, from fastest to best looking. Brightness and
/// Reduce Motion (to come) are the player's own, and never set by these.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Preset {
    Low,
    Medium,
    High,
}

impl Preset {
    pub const ALL: [Self; 3] = [Self::Low, Self::Medium, Self::High];

    pub fn name(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }

    /// The settings it chooses. Medium is the default, as the game was before
    /// there was a choice.
    pub fn prefs(self) -> VideoPrefs {
        let (frame_rate, render_scale, anti_aliasing) = match self {
            Self::Low => (FrameRate::Fps30, 75, AntiAliasing::Off),
            Self::Medium => (FrameRate::Fps30, 100, AntiAliasing::Times4),
            Self::High => (FrameRate::Fps60, 100, AntiAliasing::Times4),
        };
        VideoPrefs {
            frame_rate,
            vsync: true,
            render_scale,
            anti_aliasing,
            background: Background::SlowDown,
        }
    }
}

#[cfg(test)]
mod tests;
