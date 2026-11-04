//! The Video settings that trade looks for speed, on Settings' Video page: the
//! preset, then Display and Background. They apply as soon as they are chosen
//! (`choose`); the modules drawing what they change follow `Preferences`.

use bevy::prelude::*;

use super::super::rows::{spawn_dropdown_row, spawn_labelled, spawn_section, tip};
use crate::common::{
    AntiAliasing, Background, FrameRate, IN_BROWSER, Preset, RENDER_SCALES, VideoPrefs,
};
use crate::widgets::{ButtonState, Dropdown, INK, spawn_slider};

const WIDTH: f32 = 200.0;
const SCALE_TEXT_WIDTH: f32 = 60.0;
/// Render Scale moves in steps of this many percent.
pub(super) const SCALE_STEP: u8 = 5;

/// Which setting a control is.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::system_menu) enum VideoControl {
    Preset,
    FrameRate,
    VSync,
    RenderScale,
    AntiAliasing,
    Background,
}

/// The percentage beside Render Scale's slider.
#[derive(Component)]
pub(in crate::system_menu) struct ScaleText;

const PRESET: &str = "Sets the Video settings together, from fastest to best looking. \
    Changing one of them makes this Custom.";
const FRAME_RATE: &str = "The most frames the game draws each second. Lower limits use \
    less power and run cooler, which helps laptops on battery. A big difference.";
const VSYNC: &str = "Waits for the screen before showing each frame, so the picture \
    doesn't tear. It can add a little delay to input. A medium difference.";
const RENDER_SCALE: &str = "Draws the world at a lower resolution and stretches it to \
    fit. The interface stays sharp. Lower is faster, but blurrier. A big difference.";
const ANTI_ALIASING: &str = "Smooths jagged edges. Higher costs more graphics power. \
    With it off the land may shimmer, and faded tokens look rougher. A medium difference.";
const BACKGROUND: &str = "What the game does while another window is in front. Slow \
    down draws a frame a second to save power; Keep drawing suits streaming, or \
    watching it on another screen. A big difference while in the background.";
const BROWSER_DECIDES: &str = "Not in the browser, which decides this itself.";

/// A dropdown for a setting: its choices, the one chosen, and those that can't be.
fn dropdown(names: Vec<String>, chosen: usize, greyed: Vec<bool>) -> Dropdown {
    Dropdown {
        options: names,
        selected: chosen,
        greyed,
        ..default()
    }
}

/// Spawns a setting's dropdown row, marked, and disabled if `why_not`.
fn setting_row(
    parent: &mut ChildSpawnerCommands,
    control: VideoControl,
    label: &str,
    (what, why_not): (&str, Option<&str>),
    dropdown: Dropdown,
) {
    let made = spawn_dropdown_row(parent, label, tip(what, why_not), dropdown, WIDTH);
    let mut commands = parent.commands();
    commands.entity(made).insert(control);
    if why_not.is_some() {
        commands.entity(made).insert(ButtonState::Disabled);
    }
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
    let list = dropdown(names, chosen, greyed);
    setting_row(page, VideoControl::Preset, "Preset", (PRESET, None), list);
}

pub(super) fn spawn_display_section(sections: &mut ChildSpawnerCommands, prefs: &VideoPrefs) {
    spawn_section(sections, "Display", |section| {
        let names = FrameRate::ALL.iter().map(|it| it.name().into()).collect();
        let chosen = FrameRate::ALL.iter().position(|&it| it == prefs.frame_rate);
        let list = dropdown(names, chosen.unwrap_or(0), Vec::new());
        setting_row(
            section,
            VideoControl::FrameRate,
            "Frame Rate Limit",
            (FRAME_RATE, None),
            list,
        );

        let list = dropdown(
            vec!["On".into(), "Off".into()],
            usize::from(!prefs.vsync),
            Vec::new(),
        );
        let why_not = IN_BROWSER.then_some(BROWSER_DECIDES);
        setting_row(
            section,
            VideoControl::VSync,
            "VSync",
            (VSYNC, why_not),
            list,
        );

        spawn_labelled(section, "Render Scale", tip(RENDER_SCALE, None), |row| {
            let (least, most) = RENDER_SCALES;
            let value =
                f32::from(prefs.render_scale.clamp(least, most) - least) / f32::from(most - least);
            spawn_slider(row, value, None, WIDTH).insert(VideoControl::RenderScale);
            row.spawn((
                ScaleText,
                Text::new(format!("{}%", prefs.render_scale)),
                super::super::FONT.font(),
                TextColor(INK),
                Node {
                    width: px(SCALE_TEXT_WIDTH),
                    ..default()
                },
            ));
        });

        let names = AntiAliasing::ALL
            .iter()
            .map(|it| it.name().into())
            .collect();
        let chosen = AntiAliasing::ALL
            .iter()
            .position(|&it| it == prefs.anti_aliasing.in_use());
        let greyed = AntiAliasing::ALL.iter().map(|it| !it.usable()).collect();
        let list = dropdown(names, chosen.unwrap_or(0), greyed);
        let why_not = IN_BROWSER.then_some("2× isn't in the browser.");
        let words = match why_not {
            // Still usable: only one choice is missing.
            Some(why) => format!("{ANTI_ALIASING}\n\n{why}"),
            None => ANTI_ALIASING.into(),
        };
        setting_row(
            section,
            VideoControl::AntiAliasing,
            "Anti-aliasing",
            (&words, None),
            list,
        );
    });
}

pub(super) fn spawn_background_section(sections: &mut ChildSpawnerCommands, prefs: &VideoPrefs) {
    spawn_section(sections, "Background", |section| {
        let names = Background::ALL.iter().map(|it| it.name().into()).collect();
        let chosen = Background::ALL
            .iter()
            .position(|&it| it == prefs.background);
        let list = dropdown(names, chosen.unwrap_or(0), Vec::new());
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
