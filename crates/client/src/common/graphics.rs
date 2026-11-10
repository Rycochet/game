//! The Graphics settings that trade looks for speed or clutter, kept in
//! `Preferences`, and the presets that set them together. Each is used by the
//! module that draws what it changes; the window's own (display mode,
//! resolution) are in `display`. What each can be is in `choices`.

mod choices;
mod max_resolution;

pub use choices::{
    AntiAliasing, Background, FrameRate, Precipitation, Remembered, Shadows, ViewDistance,
};
pub use max_resolution::MaxResolution;

use serde::{Deserialize, Serialize};

/// Whether this build runs in a browser, where some choices are the browser's.
pub const IN_BROWSER: bool = cfg!(target_arch = "wasm32");

/// Graphics settings for speed and clutter.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(default)]
pub struct GraphicsPrefs {
    /// The most frames drawn each second.
    pub frame_rate: FrameRate,
    /// Waits for the screen before showing a frame (native only).
    pub vsync: bool,
    /// How much of the window's resolution the world is drawn at, in percent
    /// (`RENDER_SCALES`); the interface is always drawn at full resolution.
    pub render_scale: u8,
    /// The most lines the world is drawn with, whatever the window's size.
    pub max_resolution: MaxResolution,
    pub anti_aliasing: AntiAliasing,
    /// What happens to the drawing while another window is in front.
    pub background: Background,
    /// The least light on what can be seen, in percent of full daylight
    /// (`BRIGHTNESS_MOST` at most, so the dark still counts). The player's
    /// own: presets leave it alone.
    pub brightness: u8,
    /// The world map's shadows (the brightest sun or moon's).
    pub shadows: Shadows,
    /// The world map lit as at noon by the main sun alone, whatever the time.
    pub midday_sun: bool,
    /// How much of the world map's land is drawn round the player.
    pub view_distance: ViewDistance,
    /// The world map's cloud sheet. Clouds shade the ground only with this and
    /// shadows both on.
    pub clouds: bool,
    /// Rain, snow and hail falling, on the world map and in sites open to the sky.
    pub precipitation: Precipitation,
    /// Trees and low growth painted on the world map.
    pub foliage: bool,
    /// Sites' floors and walls drawn with their pictures, or in plain colours.
    pub floor_textures: bool,
    /// How places seen before, but not now, are drawn in sites.
    pub remembered: Remembered,
    /// Still water, and tokens and the camera jumping to their places rather
    /// than gliding. The player's own: presets leave it alone.
    pub reduce_motion: bool,
}

impl Default for GraphicsPrefs {
    fn default() -> Self {
        Preset::Medium.prefs()
    }
}

impl GraphicsPrefs {
    /// The preset these are, if any. Choices no preset sets are left out.
    pub fn preset(&self) -> Option<Preset> {
        Preset::ALL
            .into_iter()
            .find(|preset| preset.over(*self) == *self)
    }

    /// The render scale as a share of the window, `0.5..=1`.
    pub fn render_share(&self) -> f32 {
        f32::from(self.render_scale.clamp(RENDER_SCALES.0, RENDER_SCALES.1)) / 100.0
    }

    /// The share of a window `height` physical pixels tall the world is drawn
    /// at: Render Scale, or less to keep to Max Resolution.
    pub fn drawn_share(&self, height: u32) -> f32 {
        self.render_share()
            .min(self.max_resolution.share_of(height))
    }

    /// Brightness as a share of full daylight, `0..=0.49`.
    pub fn least_light(&self) -> f32 {
        f32::from(self.brightness.min(BRIGHTNESS_MOST)) / 100.0
    }
}

/// The render scales offered, in percent: least and most.
pub const RENDER_SCALES: (u8, u8) = (50, 100);

/// The most Brightness can be, in percent: below half of daylight, so it helps
/// eyes without lighting up the night (owner).
pub const BRIGHTNESS_MOST: u8 = 49;

/// Sets of Graphics settings, from fastest to best looking. Brightness and
/// Reduce Motion are the player's own, and never set by these.
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

    /// `prefs` with everything this preset sets set, keeping the player's own.
    pub fn over(self, prefs: GraphicsPrefs) -> GraphicsPrefs {
        GraphicsPrefs {
            brightness: prefs.brightness,
            reduce_motion: prefs.reduce_motion,
            ..self.prefs()
        }
    }

    /// The settings it chooses. Medium is the default, as the game was before
    /// there was a choice.
    pub fn prefs(self) -> GraphicsPrefs {
        let low = self == Self::Low;
        let (frame_rate, view_distance) = match self {
            Self::Low => (FrameRate::Fps30, ViewDistance::Near),
            Self::Medium => (FrameRate::Fps30, ViewDistance::Normal),
            Self::High => (FrameRate::Fps60, ViewDistance::Far),
        };
        // A browser draws more slowly (one thread, WebGL), so it is held to
        // fewer lines by default (owner, 2026-11-09).
        let max_resolution = match (self, IN_BROWSER) {
            (Self::Low, true) => MaxResolution::P720,
            (Self::Medium, true) => MaxResolution::P1080,
            (Self::High, true) => MaxResolution::P1440,
            (Self::Low, false) => MaxResolution::P1080,
            (_, false) => MaxResolution::Off,
        };
        GraphicsPrefs {
            frame_rate,
            vsync: true,
            render_scale: if low { 75 } else { 100 },
            max_resolution,
            anti_aliasing: if low {
                AntiAliasing::Off
            } else {
                AntiAliasing::Times4
            },
            background: Background::SlowDown,
            brightness: 0,
            shadows: if low { Shadows::Off } else { Shadows::High },
            midday_sun: low,
            view_distance,
            clouds: !low,
            precipitation: if low {
                Precipitation::Light
            } else {
                Precipitation::Full
            },
            foliage: !low,
            floor_textures: true,
            remembered: Remembered::Show,
            reduce_motion: false,
        }
    }
}

#[cfg(test)]
mod tests;
