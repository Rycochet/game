//! The Video settings that trade looks for speed, on Settings' Video page: the
//! preset, then Display and Background (and, in `sections`, the rest), with
//! the pieces their rows are made of. They apply as soon as they are chosen
//! (`choose`); the modules drawing what they change follow `Preferences`.

use bevy::prelude::*;

use super::super::rows::{
    spawn_dropdown_row, spawn_labelled, spawn_section, spawn_switch_row, tip,
};
use crate::common::{
    AntiAliasing, Background, FrameRate, IN_BROWSER, MaxResolution, Preset, RENDER_SCALES,
    VideoPrefs,
};
use crate::widgets::{ButtonState, Dropdown, INK, UiId, spawn_slider};

const WIDTH: f32 = 200.0;
const PERCENT_WIDTH: f32 = 60.0;

/// Which setting a control is.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::system_menu) enum VideoControl {
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

impl VideoControl {
    fn ui_name(self) -> &'static str {
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
}

/// The percentage beside a slider, by the slider's setting.
#[derive(Component)]
pub(in crate::system_menu) struct PercentText(pub(super) VideoControl);

const PRESET: &str = "Sets the Video settings together, from fastest to best looking. \
    Brightness and Reduce Motion are yours, and stay as they are. Changing any other \
    makes this Custom.";
const FRAME_RATE: &str = "The most frames the game draws each second. Lower limits use \
    less power and run cooler, which helps laptops on battery. A big difference.";
const VSYNC: &str = "Waits for the screen before showing each frame, so the picture \
    doesn't tear. It can add a little delay to input. A medium difference.";
const RENDER_SCALE: &str = "Draws the world at a lower resolution and stretches it to \
    fit. The interface stays sharp. Lower is faster, but blurrier. A big difference.";
const MAX_RESOLUTION: &str = "The most lines the world is drawn with, however big the     window or screen. Bigger windows are then stretched, so full screen costs no more     to draw. The interface stays sharp. A big difference on large screens.";
const ANTI_ALIASING: &str = "Smooths jagged edges. Higher costs more graphics power. \
    With it off the land may shimmer, and faded tokens look rougher. A medium difference.";
const BACKGROUND: &str = "What the game does while another window is in front. Slow \
    down draws a frame a second to save power; Keep drawing suits streaming, or \
    watching it on another screen. A big difference while in the background.";
const BROWSER_DECIDES: &str = "Not in the browser, which decides this itself.";

/// A dropdown for a setting: its choices' names, the one chosen, and those that
/// can't be (none if empty).
pub(super) fn dropdown<T: PartialEq + Copy>(
    all: &[T],
    name: impl Fn(T) -> &'static str,
    chosen: T,
    greyed: Vec<bool>,
) -> Dropdown {
    Dropdown {
        options: all.iter().map(|&it| name(it).into()).collect(),
        selected: all.iter().position(|&it| it == chosen).unwrap_or(0),
        greyed,
        ..default()
    }
}

/// Spawns a setting's dropdown row, marked, and disabled if `why_not`.
pub(super) fn setting_row(
    parent: &mut ChildSpawnerCommands,
    control: VideoControl,
    label: &str,
    (what, why_not): (&str, Option<&str>),
    dropdown: Dropdown,
) {
    let made = spawn_dropdown_row(parent, label, tip(what, why_not), dropdown, WIDTH);
    let mut commands = parent.commands();
    commands
        .entity(made)
        .insert((control, UiId::new("settings.video", control.ui_name())));
    if why_not.is_some() {
        commands.entity(made).insert(ButtonState::Disabled);
    }
}

/// Spawns a setting's switch row, `on` or off, marked, and disabled if `why_not`.
pub(super) fn switch_row(
    parent: &mut ChildSpawnerCommands,
    control: VideoControl,
    label: &str,
    (what, why_not): (&str, Option<&str>),
    on: bool,
) {
    let state = match (why_not.is_some(), on) {
        (false, _) => ButtonState::Normal,
        (true, false) => ButtonState::Disabled,
        (true, true) => ButtonState::DisabledSelected,
    };
    let made = spawn_switch_row(parent, label, tip(what, why_not), state, on);
    parent
        .commands()
        .entity(made)
        .insert((control, UiId::new("settings.video", control.ui_name())));
}

/// Spawns a setting's slider row, from `least` to `most` percent, at `percent`.
pub(super) fn slider_row(
    parent: &mut ChildSpawnerCommands,
    control: VideoControl,
    label: &str,
    what: &str,
    (least, most, percent): (u8, u8, u8),
) {
    spawn_labelled(parent, label, tip(what, None), |row| {
        let value = f32::from(percent.clamp(least, most) - least) / f32::from(most - least);
        spawn_slider(row, value, None, WIDTH)
            .insert((control, UiId::new("settings.video", control.ui_name())));
        row.spawn((
            PercentText(control),
            Text::new(format!("{percent}%")),
            super::super::FONT.font(),
            TextColor(INK),
            Node {
                width: px(PERCENT_WIDTH),
                ..default()
            },
        ));
    });
}

/// Preset, at the top of the page: Custom while the settings are no preset.
pub(super) fn spawn_preset_row(page: &mut ChildSpawnerCommands, prefs: &VideoPrefs) {
    let mut names: Vec<String> = Preset::ALL.iter().map(|it| it.name().into()).collect();
    names.push("Custom".into());
    let chosen = prefs
        .preset()
        .and_then(|preset| Preset::ALL.iter().position(|&it| it == preset))
        .unwrap_or(Preset::ALL.len());
    // Custom is only shown, not chosen.
    let mut greyed = vec![false; Preset::ALL.len()];
    greyed.push(true);
    let list = Dropdown {
        options: names,
        selected: chosen,
        greyed,
        ..default()
    };
    setting_row(page, VideoControl::Preset, "Preset", (PRESET, None), list);
}

pub(super) fn spawn_display_section(sections: &mut ChildSpawnerCommands, prefs: &VideoPrefs) {
    spawn_section(sections, "Display", |section| {
        let list = dropdown(
            &FrameRate::ALL,
            FrameRate::name,
            prefs.frame_rate,
            Vec::new(),
        );
        let row = (FRAME_RATE, None);
        setting_row(
            section,
            VideoControl::FrameRate,
            "Frame Rate Limit",
            row,
            list,
        );

        let why_not = IN_BROWSER.then_some(BROWSER_DECIDES);
        let row = (VSYNC, why_not);
        switch_row(section, VideoControl::VSync, "VSync", row, prefs.vsync);

        let (least, most) = RENDER_SCALES;
        let at = (least, most, prefs.render_scale);
        slider_row(
            section,
            VideoControl::RenderScale,
            "Render Scale",
            RENDER_SCALE,
            at,
        );

        let list = dropdown(
            &MaxResolution::ALL,
            MaxResolution::name,
            prefs.max_resolution,
            Vec::new(),
        );
        let row = (MAX_RESOLUTION, None);
        setting_row(
            section,
            VideoControl::MaxResolution,
            "Max Resolution",
            row,
            list,
        );

        let greyed = AntiAliasing::ALL.iter().map(|it| !it.usable()).collect();
        let chosen = prefs.anti_aliasing.in_use();
        let list = dropdown(&AntiAliasing::ALL, AntiAliasing::name, chosen, greyed);
        // Still usable in a browser: only one choice is missing.
        let words = match IN_BROWSER {
            true => format!("{ANTI_ALIASING}\n\n2× isn't in the browser."),
            false => ANTI_ALIASING.into(),
        };
        let row = (words.as_str(), None);
        setting_row(
            section,
            VideoControl::AntiAliasing,
            "Anti-aliasing",
            row,
            list,
        );
    });
}

pub(super) fn spawn_background_section(sections: &mut ChildSpawnerCommands, prefs: &VideoPrefs) {
    spawn_section(sections, "Background", |section| {
        let list = dropdown(
            &Background::ALL,
            Background::name,
            prefs.background,
            Vec::new(),
        );
        let why_not = IN_BROWSER.then_some("Not in the browser, which slows hidden tabs itself.");
        let row = (BACKGROUND, why_not);
        setting_row(
            section,
            VideoControl::Background,
            "When Not in Focus",
            row,
            list,
        );
    });
}
