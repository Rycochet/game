//! Settings' Video page: the preset, then sections. Window: how the window is
//! shown (Display mode) and full screen's resolution, which change nothing
//! until Apply, which asks whether to keep them (`keep`); in a browser they are
//! shown, disabled, as the browser looks after the window. Then the settings
//! trading looks for speed (`options`) or showing what is drawn (`sections`),
//! which apply at once (`choose`).

mod choose;
mod options;
mod sections;

pub(in crate::system_menu) use choose::{VideoShown, use_video_options};

use bevy::prelude::*;
use bevy::window::{Monitor, OnMonitor, PrimaryWindow};

use super::super::{Action, SystemMenu};
use super::keep::VideoTrial;
use super::rows::{spawn_dropdown_row, spawn_section, spawn_sections, tip};
use super::{FONT, ROW_HEIGHT};
use crate::common::{
    DisplayChoice, DisplayInUse, DisplayMode, Preferences, Resolution, resolutions,
};
use crate::widgets::{
    ButtonPressed, ButtonState, ButtonStyle, Dropdown, DropdownChosen, INK, Pressable, TextSize,
    spawn_button,
};

const DROPDOWN_WIDTH: f32 = 300.0;
const DISPLAY_MODE: &str = "How the game fills the screen: in a window, in a window     without edges covering the screen, or full screen.";
const RESOLUTION: &str = "The screen's resolution in full screen.";
const BROWSER_WINDOW: &str = "The browser looks after the window.";
const APPLY_WIDTH: f32 = 120.0;
/// Whether this build can change the window.
const CAN_CHANGE: bool = cfg!(not(target_arch = "wasm32"));

#[derive(Component)]
pub(in crate::system_menu) struct VideoPage;

#[derive(Component)]
pub(in crate::system_menu) struct ModeDropdown;

/// What each entry is: `None` the screen's own resolution.
#[derive(Component)]
pub(in crate::system_menu) struct ResolutionDropdown(Vec<Option<Resolution>>);

#[derive(Component)]
pub(in crate::system_menu) struct ApplyButton;

/// The page's choices, until applied.
#[derive(Resource, Default)]
pub(in crate::system_menu) struct VideoForm(DisplayChoice);

/// Which entry is this resolution: the same one, else the same size, else the
/// screen's own.
fn resolution_index(entries: &[Option<Resolution>], wanted: Option<Resolution>) -> usize {
    let size = |entry: &Option<Resolution>| entry.map(|seen| (seen.width, seen.height));
    entries
        .iter()
        .position(|&entry| entry == wanted)
        .or_else(|| {
            entries
                .iter()
                .position(|entry| size(entry) == size(&wanted))
        })
        .unwrap_or(0)
}

/// The resolution dropdown's entries for this monitor, with their labels.
fn resolution_entries(monitor: Option<&Monitor>) -> (Vec<Option<Resolution>>, Vec<String>) {
    let Some(monitor) = monitor else {
        return (vec![None], vec!["Desktop".into()]);
    };
    let (width, height) = (monitor.physical_width, monitor.physical_height);
    let found = resolutions(&monitor.video_modes);
    let entries = std::iter::once(None).chain(found.iter().copied().map(Some));
    let labels = std::iter::once(format!("Desktop ({width} × {height})"))
        .chain(found.iter().map(|resolution| resolution.label()));
    (entries.collect(), labels.collect())
}

/// Builds the page again on opening Settings (dropping choices not applied),
/// when how the window is shown, or the monitor it is on, changes, and when
/// the other Video settings change (`VideoShown`).
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn fill_video_page(
    mut commands: Commands,
    in_use: Res<DisplayInUse>,
    preferences: Res<Preferences>,
    mut shown: ResMut<VideoShown>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    menu: Res<SystemMenu>,
    mut form: ResMut<VideoForm>,
    window: Single<Option<&OnMonitor>, With<PrimaryWindow>>,
    monitors: Query<&Monitor>,
    page: Single<Entity, With<VideoPage>>,
    mut last: Local<Option<(SystemMenu, Option<Entity>)>>,
) {
    let on = window.map(|on| on.0);
    let was = last.replace((*menu, on));
    let opening = *menu == SystemMenu::Settings
        && !matches!(was, Some((SystemMenu::Settings | SystemMenu::KeepVideo, _)));
    // (Also when the window is on another monitor, with other resolutions.)
    let moved = was.is_some_and(|(_, was_on)| was_on != on);
    let holding = mouse.pressed(MouseButton::Left) || touches.iter().next().is_some();
    let options = shown.needs_building(&preferences.video, holding);
    if was.is_some() && !opening && !moved && !in_use.is_changed() && !options {
        return;
    }
    *shown = VideoShown {
        prefs: Some(preferences.video),
        stale: false,
    };
    form.0 = in_use.0;
    let monitor = on.and_then(|on| monitors.get(on).ok());
    let (entries, labels) = resolution_entries(monitor);
    let page = *page;
    commands.entity(page).despawn_related::<Children>();
    let video = preferences.video;
    commands.entity(page).with_children(|page| {
        options::spawn_preset_row(page, &video);
        spawn_sections(page, |sections| {
            spawn_section(sections, "Window", |section| {
                spawn_window_rows(section, in_use.0, entries, labels);
            });
            options::spawn_display_section(sections, &video);
            sections::spawn_lighting_section(sections, &video);
            sections::spawn_world_map_section(sections, &video);
            sections::spawn_sites_section(sections, &video);
            sections::spawn_motion_section(sections, &video);
            options::spawn_background_section(sections, &video);
        });
    });
}

/// Display mode and Resolution, with Apply at the right below them.
fn spawn_window_rows(
    section: &mut ChildSpawnerCommands,
    in_use: DisplayChoice,
    entries: Vec<Option<Resolution>>,
    labels: Vec<String>,
) {
    let why_not = (!CAN_CHANGE).then_some(BROWSER_WINDOW);
    let modes = Dropdown {
        options: DisplayMode::ALL
            .iter()
            .map(|mode| mode.name().into())
            .collect(),
        selected: DisplayMode::ALL
            .iter()
            .position(|&mode| mode == in_use.mode)
            .unwrap_or(0),
        ..default()
    };
    let mode_tip = tip(DISPLAY_MODE, why_not);
    let mode_dropdown =
        spawn_dropdown_row(section, "Display mode", mode_tip, modes, DROPDOWN_WIDTH);
    section
        .commands()
        .entity(mode_dropdown)
        .insert(ModeDropdown);
    let sizes = Dropdown {
        options: labels,
        selected: resolution_index(&entries, in_use.resolution),
        ..default()
    };
    let size_tip = tip(RESOLUTION, Some(why_not.unwrap_or("Only for Full screen.")));
    let size_dropdown = spawn_dropdown_row(section, "Resolution", size_tip, sizes, DROPDOWN_WIDTH);
    section
        .commands()
        .entity(size_dropdown)
        .insert(ResolutionDropdown(entries));
    if let Some(why) = why_not {
        section.spawn((Text::new(why), TextSize::Small.font(), TextColor(INK)));
    }
    section
        .spawn(Node {
            justify_content: JustifyContent::FlexEnd,
            ..default()
        })
        .with_children(|row| {
            let node = Node {
                width: px(APPLY_WIDTH),
                height: px(ROW_HEIGHT),
                ..default()
            };
            spawn_button(row, ButtonState::Disabled, "Apply", FONT, node).insert((
                ApplyButton,
                Action::ApplyVideo,
                ButtonStyle::Main,
                Pressable,
            ));
        });
}

/// A button on the page, and which.
type PageButton = (
    &'static mut ButtonState,
    Has<ModeDropdown>,
    Has<ResolutionDropdown>,
    Has<ApplyButton>,
);

/// Takes the choices, applies them on Apply, and shows what can be used.
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn use_video_page(
    mut chosen: MessageReader<DropdownChosen>,
    mut pressed: MessageReader<ButtonPressed>,
    resolution_dropdowns: Query<&ResolutionDropdown>,
    mode_dropdowns: Query<(), With<ModeDropdown>>,
    applies: Query<(), With<ApplyButton>>,
    mut form: ResMut<VideoForm>,
    mut in_use: ResMut<DisplayInUse>,
    mut trial: ResMut<VideoTrial>,
    time: Res<Time<Real>>,
    mut states: Query<PageButton>,
) {
    for &DropdownChosen(dropdown, index) in chosen.read() {
        if mode_dropdowns.contains(dropdown) {
            form.0.mode = DisplayMode::ALL[index.min(DisplayMode::ALL.len() - 1)];
        } else if let Ok(entries) = resolution_dropdowns.get(dropdown) {
            form.0.resolution = entries.0.get(index).copied().flatten();
        }
    }
    let changed = form.0 != in_use.0;
    if pressed.read().any(|press| applies.contains(press.0)) && CAN_CHANGE && changed {
        trial.start(in_use.0, time.elapsed_secs_f64());
        in_use.0 = form.0;
    }
    let usable = |yes: bool| {
        if yes {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        }
    };
    for (mut state, mode, resolution, apply) in &mut states {
        let wanted = if mode {
            usable(CAN_CHANGE)
        } else if resolution {
            usable(CAN_CHANGE && form.0.mode == DisplayMode::FullScreen)
        } else if apply {
            usable(CAN_CHANGE && form.0 != in_use.0)
        } else {
            continue;
        };
        // (Leaving a usable one as it is: hovered, or open.)
        if wanted.is_disabled() != state.is_disabled() {
            *state = wanted;
        }
    }
}

#[cfg(test)]
mod tests;
