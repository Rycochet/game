//! Putting the Graphics settings chosen into `Preferences`, and saying when the
//! page should be built again to show them.

use bevy::prelude::*;

use super::options::{GraphicsControl, show_percent};
use crate::common::{
    AntiAliasing, BRIGHTNESS_MOST, Background, FrameRate, GraphicsPrefs, MaxResolution,
    Precipitation, Preferences, Preset, RENDER_SCALES, Remembered, Shadows, ViewDistance,
};
use crate::dom::Dom;
use crate::widgets::{DropdownChosen, SliderMoved, SwitchFlipped};

/// Render Scale moves in steps of this many percent.
const SCALE_STEP: u8 = 5;

/// The Graphics settings the page shows. Moving a slider changes them without
/// showing the page again, which would move the slider being dragged: then
/// `stale` says the preset shown may be wrong, until the slider is let go.
#[derive(Resource, Default)]
pub(in crate::system_menu) struct GraphicsShown {
    pub(super) prefs: Option<GraphicsPrefs>,
    pub(super) stale: bool,
}

impl GraphicsShown {
    /// Whether the page should show `prefs` again. `holding`: a button or
    /// finger is down.
    pub(super) fn needs_showing(&self, prefs: &GraphicsPrefs, holding: bool) -> bool {
        self.prefs != Some(*prefs) || (self.stale && !holding)
    }
}

/// The percentage at this place along a slider from `least` to `most`, in
/// whole `step`s.
fn percent_at(value: f32, (least, most): (u8, u8), step: u8) -> u8 {
    let percent = f32::from(least) + value.clamp(0.0, 1.0) * f32::from(most - least);
    let step = f32::from(step);
    ((percent / step).round() * step).min(f32::from(most)) as u8
}

/// Picks the `index`th of `all` into `to`, if there is one.
fn pick<T: Copy>(all: &[T], index: usize, to: &mut T) {
    if let Some(&chosen) = all.get(index) {
        *to = chosen;
    }
}

/// What a dropdown's choice does to the settings.
fn choose(prefs: &mut GraphicsPrefs, control: GraphicsControl, index: usize) {
    match control {
        GraphicsControl::Preset => {
            if let Some(&preset) = Preset::ALL.get(index) {
                *prefs = preset.over(*prefs);
            }
        }
        GraphicsControl::FrameRate => pick(&FrameRate::ALL, index, &mut prefs.frame_rate),
        GraphicsControl::MaxResolution => {
            pick(&MaxResolution::ALL, index, &mut prefs.max_resolution)
        }
        GraphicsControl::AntiAliasing => pick(&AntiAliasing::ALL, index, &mut prefs.anti_aliasing),
        GraphicsControl::Background => pick(&Background::ALL, index, &mut prefs.background),
        GraphicsControl::Shadows => pick(&Shadows::ALL, index, &mut prefs.shadows),
        GraphicsControl::ViewDistance => pick(&ViewDistance::ALL, index, &mut prefs.view_distance),
        GraphicsControl::Precipitation => {
            pick(&Precipitation::ALL, index, &mut prefs.precipitation)
        }
        GraphicsControl::Remembered => pick(&Remembered::ALL, index, &mut prefs.remembered),
        _ => {}
    }
}

/// The setting a switch is, if it is one.
fn switched(prefs: &mut GraphicsPrefs, control: GraphicsControl) -> Option<&mut bool> {
    match control {
        GraphicsControl::VSync => Some(&mut prefs.vsync),
        GraphicsControl::MiddaySun => Some(&mut prefs.midday_sun),
        GraphicsControl::Clouds => Some(&mut prefs.clouds),
        GraphicsControl::Foliage => Some(&mut prefs.foliage),
        GraphicsControl::FloorTextures => Some(&mut prefs.floor_textures),
        GraphicsControl::ReduceMotion => Some(&mut prefs.reduce_motion),
        _ => None,
    }
}

/// A slider's least, most and step, in percent, if this is one.
pub(super) fn slider_range(control: GraphicsControl) -> Option<(u8, u8, u8)> {
    match control {
        GraphicsControl::RenderScale => Some((RENDER_SCALES.0, RENDER_SCALES.1, SCALE_STEP)),
        GraphicsControl::Brightness => Some((0, BRIGHTNESS_MOST, 1)),
        _ => None,
    }
}

/// What a slider's place sets, if it is one: the setting and its new percent.
fn slide(prefs: &mut GraphicsPrefs, control: GraphicsControl, value: f32) -> Option<u8> {
    let (least, most, step) = slider_range(control)?;
    let setting = match control {
        GraphicsControl::RenderScale => &mut prefs.render_scale,
        _ => &mut prefs.brightness,
    };
    let percent = percent_at(value, (least, most), step);
    (*setting != percent).then(|| {
        *setting = percent;
        percent
    })
}

/// Takes the choices into `Preferences` as they are made.
pub(in crate::system_menu) fn use_graphics_options(
    mut chosen: MessageReader<DropdownChosen>,
    mut moved: MessageReader<SliderMoved>,
    mut flipped: MessageReader<SwitchFlipped>,
    controls: Query<&GraphicsControl>,
    mut preferences: ResMut<Preferences>,
    mut shown: ResMut<GraphicsShown>,
    mut dom: Dom,
) {
    for &DropdownChosen(dropdown, index) in chosen.read() {
        if let Ok(&control) = controls.get(dropdown) {
            let mut prefs = preferences.graphics;
            choose(&mut prefs, control, index);
            // (Shown again by the change, with the preset.)
            preferences.graphics = prefs;
        }
    }
    for &SwitchFlipped(switch, on) in flipped.read() {
        if let Ok(&control) = controls.get(switch) {
            let mut prefs = preferences.graphics;
            if let Some(setting) = switched(&mut prefs, control) {
                *setting = on;
                // (Shown again by the change.)
                preferences.graphics = prefs;
            }
        }
    }
    for &SliderMoved(slider, value) in moved.read() {
        let Ok(&control) = controls.get(slider) else {
            continue;
        };
        let mut prefs = preferences.graphics;
        let Some(percent) = slide(&mut prefs, control, value) else {
            continue;
        };
        preferences.graphics = prefs;
        shown.prefs = Some(prefs);
        shown.stale = true;
        show_percent(&mut dom, control, percent);
    }
}

#[cfg(test)]
mod tests;
