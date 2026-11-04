//! Putting the Video settings chosen into `Preferences`, and saying when the
//! page should be built again to show them.

use bevy::prelude::*;

use super::options::{PercentText, VideoControl};
use crate::common::{
    AntiAliasing, BRIGHTNESS_MOST, Background, FrameRate, Precipitation, Preferences, Preset,
    RENDER_SCALES, Remembered, Shadows, VideoPrefs, ViewDistance,
};
use crate::widgets::{DropdownChosen, SliderMoved};

/// Render Scale moves in steps of this many percent.
const SCALE_STEP: u8 = 5;

/// The Video settings the page shows. Moving a slider changes them without
/// building the page again, which would drop the slider being dragged: then
/// `stale` says the preset shown may be wrong, until the slider is let go.
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

/// What a dropdown's choice does to the settings. On-off dropdowns list On first.
fn choose(prefs: &mut VideoPrefs, control: VideoControl, index: usize) {
    let on = index == 0;
    match control {
        VideoControl::Preset => {
            if let Some(&preset) = Preset::ALL.get(index) {
                *prefs = preset.over(*prefs);
            }
        }
        VideoControl::FrameRate => pick(&FrameRate::ALL, index, &mut prefs.frame_rate),
        VideoControl::VSync => prefs.vsync = on,
        VideoControl::AntiAliasing => pick(&AntiAliasing::ALL, index, &mut prefs.anti_aliasing),
        VideoControl::Background => pick(&Background::ALL, index, &mut prefs.background),
        VideoControl::Shadows => pick(&Shadows::ALL, index, &mut prefs.shadows),
        VideoControl::MiddaySun => prefs.midday_sun = on,
        VideoControl::ViewDistance => pick(&ViewDistance::ALL, index, &mut prefs.view_distance),
        VideoControl::Clouds => prefs.clouds = on,
        VideoControl::Precipitation => pick(&Precipitation::ALL, index, &mut prefs.precipitation),
        VideoControl::Foliage => prefs.foliage = on,
        VideoControl::FloorTextures => prefs.floor_textures = on,
        VideoControl::Remembered => pick(&Remembered::ALL, index, &mut prefs.remembered),
        VideoControl::ReduceMotion => prefs.reduce_motion = on,
        VideoControl::RenderScale | VideoControl::Brightness => {}
    }
}

/// What a slider's place sets, if it is one: the setting and its new percent.
fn slide(prefs: &mut VideoPrefs, control: VideoControl, value: f32) -> Option<u8> {
    let (setting, range, step) = match control {
        VideoControl::RenderScale => (&mut prefs.render_scale, RENDER_SCALES, SCALE_STEP),
        VideoControl::Brightness => (&mut prefs.brightness, (0, BRIGHTNESS_MOST), 1),
        _ => return None,
    };
    let percent = percent_at(value, range, step);
    (*setting != percent).then(|| {
        *setting = percent;
        percent
    })
}

/// Takes the choices into `Preferences` as they are made.
pub(in crate::system_menu) fn use_video_options(
    mut chosen: MessageReader<DropdownChosen>,
    mut moved: MessageReader<SliderMoved>,
    controls: Query<&VideoControl>,
    mut preferences: ResMut<Preferences>,
    mut shown: ResMut<VideoShown>,
    mut texts: Query<(&PercentText, &mut Text)>,
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
        let Ok(&control) = controls.get(slider) else {
            continue;
        };
        let mut prefs = preferences.video;
        let Some(percent) = slide(&mut prefs, control, value) else {
            continue;
        };
        preferences.video = prefs;
        shown.prefs = Some(prefs);
        shown.stale = true;
        for (text, mut words) in &mut texts {
            if text.0 == control {
                words.0 = format!("{percent}%");
            }
        }
    }
}

#[cfg(test)]
mod tests;
