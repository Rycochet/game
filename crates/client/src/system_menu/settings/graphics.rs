//! Settings' Graphics page, as markup (`page`): the preset, then sections.
//! Window (`window`): how the window is shown (Display mode) and full
//! screen's resolution, which change nothing until Apply, which asks whether
//! to keep them (`keep`). Then the settings trading looks for speed or
//! saying what is drawn (`options`), which apply at once (`choose`).

mod choose;
mod options;
mod page;
mod window;

pub(in crate::system_menu) use choose::{GraphicsShown, use_graphics_options};

use bevy::prelude::*;
use bevy::window::{Monitor, OnMonitor, PrimaryWindow};

use super::super::{Action, SystemMenu};
use super::keep::GraphicsTrial;
use crate::common::{DisplayChoice, DisplayInUse, DisplayMode, Preferences, Resolution};
use crate::dom::{Dom, document_in};
use crate::widgets::{ButtonPressed, Dropdown, DropdownChosen, UiId};
use options::GraphicsControl;
use window::CAN_CHANGE;

#[derive(Component)]
pub(in crate::system_menu) struct ModeDropdown;

/// What each entry is: `None` the screen's own resolution.
#[derive(Component)]
pub(in crate::system_menu) struct ResolutionDropdown(Vec<Option<Resolution>>);

#[derive(Component)]
pub(in crate::system_menu) struct ApplyButton;

/// The page's choices, until applied.
#[derive(Resource, Default)]
pub(in crate::system_menu) struct GraphicsForm(DisplayChoice);

/// The page's own document, to be built in Settings' page for it.
pub(super) fn page_in(page: Entity) -> impl Bundle {
    document_in(&page::GRAPHICS, page)
}

/// What a built control of the page is known by, from its id.
pub(super) fn bind_graphics(bound: &mut EntityCommands, id: &UiId) {
    let name = id.parts().1;
    match name {
        "display_mode" => bound.insert(ModeDropdown),
        "resolution" => bound.insert(ResolutionDropdown(vec![None])),
        "apply" => bound.insert((ApplyButton, Action::ApplyGraphics)),
        _ => bound,
    };
    let Some(control) = GraphicsControl::named(name) else {
        return;
    };
    bound.insert(control);
    if control == GraphicsControl::AntiAliasing {
        let mut dropdown = bound.entry::<Dropdown>();
        dropdown.and_modify(options::grey_unusable);
    }
}

/// Shows the page again on opening Settings (dropping choices not applied),
/// when how the window is shown, or the monitor it is on, changes, and when
/// the other Graphics settings change (`GraphicsShown`).
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn show_graphics_page(
    in_use: Res<DisplayInUse>,
    preferences: Res<Preferences>,
    mut shown: ResMut<GraphicsShown>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    menu: Res<SystemMenu>,
    mut form: ResMut<GraphicsForm>,
    window: Single<Option<&OnMonitor>, With<PrimaryWindow>>,
    monitors: Query<&Monitor>,
    mut resolutions: Query<&mut ResolutionDropdown>,
    mut dom: Dom,
    mut last: Local<Option<(SystemMenu, Option<Entity>)>>,
) {
    // Not until the page is built.
    let Ok(mut resolutions) = resolutions.single_mut() else {
        return;
    };
    let on = window.map(|on| on.0);
    let was = last.replace((*menu, on));
    let opening = *menu == SystemMenu::Settings
        && !matches!(
            was,
            Some((
                SystemMenu::Settings | SystemMenu::KeepGraphics | SystemMenu::Ironman,
                _
            ))
        );
    // (Also when the window is on another monitor, with other resolutions.)
    let moved = was.is_some_and(|(_, was_on)| was_on != on);
    let holding = mouse.pressed(MouseButton::Left) || touches.iter().next().is_some();
    let options = shown.needs_showing(&preferences.graphics, holding);
    if was.is_some() && !opening && !moved && !in_use.is_changed() && !options {
        return;
    }
    if was.is_none() {
        window::show_fixed(&mut dom);
        options::show_fixed(&mut dom);
    }
    *shown = GraphicsShown {
        prefs: Some(preferences.graphics),
        stale: false,
    };
    form.0 = in_use.0;
    let monitor = on.and_then(|on| monitors.get(on).ok());
    let (entries, labels) = window::resolution_entries(monitor);
    window::show_window(&mut dom, in_use.0, &entries, labels);
    resolutions.0 = entries;
    options::show_options(&mut dom, &preferences.graphics);
}

/// Takes the choices, applies them on Apply, and shows what can be used.
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn use_graphics_page(
    mut chosen: MessageReader<DropdownChosen>,
    mut pressed: MessageReader<ButtonPressed>,
    resolution_dropdowns: Query<&ResolutionDropdown>,
    mode_dropdowns: Query<(), With<ModeDropdown>>,
    applies: Query<(), With<ApplyButton>>,
    mut form: ResMut<GraphicsForm>,
    mut in_use: ResMut<DisplayInUse>,
    mut trial: ResMut<GraphicsTrial>,
    time: Res<Time<Real>>,
    mut dom: Dom,
    mut shown: Local<Option<[bool; 3]>>,
) {
    // Not until the page is built.
    if applies.is_empty() {
        return;
    }
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
    let usable = [
        CAN_CHANGE,
        CAN_CHANGE && form.0.mode == DisplayMode::FullScreen,
        CAN_CHANGE && form.0 != in_use.0,
    ];
    if *shown != Some(usable) {
        *shown = Some(usable);
        window::show_usable(&mut dom, usable);
    }
}
