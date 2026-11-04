//! Putting the Video settings chosen into `Preferences`, and saying when the
//! page should be built again to show them.

use bevy::prelude::*;

use super::options::{SCALE_STEP, ScaleText, VideoControl};
use crate::common::{
    AntiAliasing, Background, FrameRate, Preferences, Preset, RENDER_SCALES, VideoPrefs,
};
use crate::widgets::{DropdownChosen, SliderMoved};

/// The Video settings the page shows. Moving Render Scale's slider changes
/// them without building the page again, which would drop the slider being
/// dragged: then `stale` says the preset shown may be wrong, until the slider
/// is let go.
#[derive(Resource, Default)]
pub(in crate::system_menu) struct VideoShown {
    pub(super) prefs: Option<VideoPrefs>,
    pub(super) stale: bool,
}

impl VideoShown {
    /// Whether the page should be built again to show `prefs`. `holding`: a
    /// button or finger is down.
    pub(super) fn needs_building(&self, prefs: &VideoPrefs, holding: bool) -> bool {
        self.prefs != Some(*prefs) || (self.stale && !holding)
    }
}

/// The render scale at this place along the slider, in steps.
fn scale_at(value: f32) -> u8 {
    let (least, most) = RENDER_SCALES;
    let percent = f32::from(least) + value.clamp(0.0, 1.0) * f32::from(most - least);
    let step = f32::from(SCALE_STEP);
    ((percent / step).round() * step) as u8
}

/// What a dropdown's choice does to the settings.
fn choose(prefs: &mut VideoPrefs, control: VideoControl, index: usize) {
    match control {
        VideoControl::Preset => {
            if let Some(&preset) = Preset::ALL.get(index) {
                *prefs = preset.prefs();
            }
        }
        VideoControl::FrameRate => {
            if let Some(&rate) = FrameRate::ALL.get(index) {
                prefs.frame_rate = rate;
            }
        }
        VideoControl::VSync => prefs.vsync = index == 0,
        VideoControl::AntiAliasing => {
            if let Some(&choice) = AntiAliasing::ALL.get(index) {
                prefs.anti_aliasing = choice;
            }
        }
        VideoControl::Background => {
            if let Some(&choice) = Background::ALL.get(index) {
                prefs.background = choice;
            }
        }
        VideoControl::RenderScale => {}
    }
}

/// Takes the choices into `Preferences` as they are made.
pub(in crate::system_menu) fn use_video_options(
    mut chosen: MessageReader<DropdownChosen>,
    mut moved: MessageReader<SliderMoved>,
    controls: Query<&VideoControl>,
    mut preferences: ResMut<Preferences>,
    mut shown: ResMut<VideoShown>,
    mut scale_text: Query<&mut Text, With<ScaleText>>,
) {
    for &DropdownChosen(dropdown, index) in chosen.read() {
        if let Ok(&control) = controls.get(dropdown) {
            let mut prefs = preferences.video;
            choose(&mut prefs, control, index);
            // (Built again by the change, to show the preset.)
            preferences.video = prefs;
        }
    }
    for &SliderMoved(slider, value) in moved.read() {
        if controls.get(slider) != Ok(&VideoControl::RenderScale) {
            continue;
        }
        let scale = scale_at(value);
        if preferences.video.render_scale != scale {
            preferences.video.render_scale = scale;
            shown.prefs = Some(preferences.video);
            shown.stale = true;
            for mut text in &mut scale_text {
                text.0 = format!("{scale}%");
            }
        }
    }
}

#[cfg(test)]
mod tests;
