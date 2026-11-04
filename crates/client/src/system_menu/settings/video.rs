//! Settings' Video page: how the window is shown (Display mode) and full
//! screen's resolution. Choosing changes nothing until Apply, which asks
//! whether to keep them (`keep`). In a browser they are shown, disabled, as the
//! browser looks after the window.

use bevy::prelude::*;
use bevy::window::{Monitor, OnMonitor, PrimaryWindow};

use super::super::{Action, SystemMenu};
use super::keep::VideoTrial;
use super::{FONT, LABEL_GAP, ROW_GAP, ROW_HEIGHT};
use crate::common::{DisplayChoice, DisplayInUse, DisplayMode, Resolution, resolutions};
use crate::widgets::{
    ButtonPressed, ButtonState, ButtonStyle, Dropdown, DropdownChosen, INK, Pressable, TextSize,
    spawn_button, spawn_dropdown,
};

const LABEL_WIDTH: f32 = 140.0;
const DROPDOWN_WIDTH: f32 = 300.0;
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

fn spawn_row(page: &mut ChildSpawnerCommands, label: &str, dropdown: Dropdown) -> Entity {
    let mut made = Entity::PLACEHOLDER;
    page.spawn(Node {
        column_gap: px(LABEL_GAP),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            Text::new(label),
            FONT.font(),
            TextColor(INK),
            Node {
                width: px(LABEL_WIDTH),
                ..default()
            },
        ));
        let node = Node {
            width: px(DROPDOWN_WIDTH),
            height: px(ROW_HEIGHT),
            ..default()
        };
        made = spawn_dropdown(row, dropdown, FONT, node).id();
    });
    made
}

/// Builds the page again on opening Settings (dropping choices not applied),
/// and when how the window is shown, or the monitor it is on, changes.
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn fill_video_page(
    mut commands: Commands,
    in_use: Res<DisplayInUse>,
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
    if was.is_some() && !opening && !moved && !in_use.is_changed() {
        return;
    }
    form.0 = in_use.0;
    let monitor = on.and_then(|on| monitors.get(on).ok());
    let (entries, labels) = resolution_entries(monitor);
    let page = *page;
    commands.entity(page).despawn_related::<Children>();
    commands.entity(page).with_children(|page| {
        // As wide as the widest row, so Apply is at its right.
        page.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(ROW_GAP),
            align_self: AlignSelf::FlexStart,
            ..default()
        })
        .with_children(|page| {
            let modes = Dropdown {
                options: DisplayMode::ALL
                    .iter()
                    .map(|mode| mode.name().into())
                    .collect(),
                selected: DisplayMode::ALL
                    .iter()
                    .position(|&mode| mode == in_use.0.mode)
                    .unwrap_or(0),
                ..default()
            };
            let mode_dropdown = spawn_row(page, "Display mode", modes);
            page.commands().entity(mode_dropdown).insert(ModeDropdown);
            let sizes = Dropdown {
                options: labels,
                selected: resolution_index(&entries, in_use.0.resolution),
                ..default()
            };
            let size_dropdown = spawn_row(page, "Resolution", sizes);
            page.commands()
                .entity(size_dropdown)
                .insert(ResolutionDropdown(entries));
            if !CAN_CHANGE {
                page.spawn((
                    Text::new("The browser looks after the window."),
                    TextSize::Small.font(),
                    TextColor(INK),
                ));
            }
            page.spawn(Node {
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
        });
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
