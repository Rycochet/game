//! The Graphics settings that trade looks for speed or say what is drawn, on
//! Settings' Graphics page: which control is which, and showing them as the
//! preferences have them. They apply as soon as they are chosen (`choose`);
//! the modules drawing what they change follow `Preferences`.

use bevy::prelude::*;

use super::super::rows::tip_words;
use super::choose::slider_range;
use crate::common::{AntiAliasing, GraphicsPrefs, IN_BROWSER, Preset};
use crate::dom::Dom;
use crate::widgets::Dropdown;

/// Which setting a control is.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::system_menu) enum GraphicsControl {
    Preset,
    FrameRate,
    VSync,
    RenderScale,
    MaxResolution,
    AntiAliasing,
    Background,
    Brightness,
    Shadows,
    MiddaySun,
    ViewDistance,
    Clouds,
    Precipitation,
    Foliage,
    FloorTextures,
    Remembered,
    ReduceMotion,
}

impl GraphicsControl {
    pub(super) const ALL: [Self; 17] = [
        Self::Preset,
        Self::FrameRate,
        Self::VSync,
        Self::RenderScale,
        Self::MaxResolution,
        Self::AntiAliasing,
        Self::Background,
        Self::Brightness,
        Self::Shadows,
        Self::MiddaySun,
        Self::ViewDistance,
        Self::Clouds,
        Self::Precipitation,
        Self::Foliage,
        Self::FloorTextures,
        Self::Remembered,
        Self::ReduceMotion,
    ];

    /// Its control's name in the page's markup.
    pub(super) fn ui_name(self) -> &'static str {
        match self {
            Self::Preset => "preset",
            Self::FrameRate => "frame_rate",
            Self::VSync => "vsync",
            Self::RenderScale => "render_scale",
            Self::MaxResolution => "max_resolution",
            Self::AntiAliasing => "anti_aliasing",
            Self::Background => "background",
            Self::Brightness => "brightness",
            Self::Shadows => "shadows",
            Self::MiddaySun => "midday_sun",
            Self::ViewDistance => "view_distance",
            Self::Clouds => "clouds",
            Self::Precipitation => "precipitation",
            Self::Foliage => "foliage",
            Self::FloorTextures => "floor_textures",
            Self::Remembered => "remembered",
            Self::ReduceMotion => "reduce_motion",
        }
    }

    /// The control with this name in the markup, if there is one.
    pub(super) fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|it| it.ui_name() == name)
    }

    fn id(self) -> String {
        format!("settings.graphics/{}", self.ui_name())
    }
}

// The tips that say more in a browser; the rest are in the markup.
const VSYNC: &str = "Waits for the screen before showing each frame, so the picture \
    doesn't tear. It can add a little delay to input. A medium difference.";
const ANTI_ALIASING: &str = "Smooths jagged edges. Higher costs more graphics power. \
    With it off the land may shimmer, and faded tokens look rougher. A medium difference.";
const BACKGROUND: &str = "What the game does while another window is in front. Slow \
    down draws a frame a second to save power; Keep drawing suits streaming, or \
    watching it on another screen. A big difference while in the background.";

/// Anti-aliasing's choices that can't be used here are shown greyed.
pub(super) fn grey_unusable(mut dropdown: Mut<Dropdown>) {
    dropdown.greyed = AntiAliasing::ALL.iter().map(|it| !it.usable()).collect();
}

/// What stays as it is while the game runs: the sliders' ranges, and what
/// a browser decides for itself, disabled with a tip saying so.
pub(super) fn show_fixed(dom: &mut Dom) {
    for control in GraphicsControl::ALL {
        if let Some((least, most, step)) = slider_range(control) {
            dom.set(&control.id(), "min", &least.to_string());
            dom.set(&control.id(), "max", &most.to_string());
            dom.set(&control.id(), "step", &step.to_string());
        }
    }
    let why_not = IN_BROWSER.then_some("Not in the browser, which decides this itself.");
    let tip = tip_words(VSYNC, why_not);
    dom.set("settings.graphics/vsync_row", "title", &tip);
    dom.set_flag(&GraphicsControl::VSync.id(), "disabled", IN_BROWSER);
    // Still usable in a browser: only one choice is missing.
    let why_not = IN_BROWSER.then_some("2× isn't in the browser.");
    let tip = tip_words(ANTI_ALIASING, why_not);
    dom.set("settings.graphics/anti_aliasing_row", "title", &tip);
    let why_not = IN_BROWSER.then_some("Not in the browser, which slows hidden tabs itself.");
    let tip = tip_words(BACKGROUND, why_not);
    dom.set("settings.graphics/background_row", "title", &tip);
    dom.set_flag(&GraphicsControl::Background.id(), "disabled", IN_BROWSER);
}

/// The percentage beside a slider.
pub(super) fn show_percent(dom: &mut Dom, control: GraphicsControl, percent: u8) {
    let id = format!("{}_percent", control.id());
    dom.set_text(&id, format!("{percent}%"));
}

/// Shows every setting as `prefs` have it. The preset is Custom while they
/// are no preset.
pub(super) fn show_options(dom: &mut Dom, prefs: &GraphicsPrefs) {
    use GraphicsControl as Control;
    let preset = prefs.preset().map_or("Custom", Preset::name);
    let chosen = [
        (Control::Preset, preset),
        (Control::FrameRate, prefs.frame_rate.name()),
        (Control::MaxResolution, prefs.max_resolution.name()),
        (Control::AntiAliasing, prefs.anti_aliasing.in_use().name()),
        (Control::Background, prefs.background.name()),
        (Control::Shadows, prefs.shadows.name()),
        (Control::ViewDistance, prefs.view_distance.name()),
        (Control::Precipitation, prefs.precipitation.name()),
        (Control::Remembered, prefs.remembered.name()),
    ];
    for (control, name) in chosen {
        dom.set(&control.id(), "value", name);
    }
    let switched = [
        (Control::VSync, prefs.vsync),
        (Control::MiddaySun, prefs.midday_sun),
        (Control::Clouds, prefs.clouds),
        (Control::Foliage, prefs.foliage),
        (Control::FloorTextures, prefs.floor_textures),
        (Control::ReduceMotion, prefs.reduce_motion),
    ];
    for (control, on) in switched {
        dom.set_flag(&control.id(), "checked", on);
    }
    let slid = [
        (Control::RenderScale, prefs.render_scale),
        (Control::Brightness, prefs.brightness),
    ];
    for (control, percent) in slid {
        let (least, most, _) = slider_range(control).unwrap_or((0, 100, 1));
        let percent = percent.clamp(least, most);
        dom.set(&control.id(), "value", &percent.to_string());
        show_percent(dom, control, percent);
    }
}
