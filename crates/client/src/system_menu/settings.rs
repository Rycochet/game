//! The pages of Settings, under its tabs: Gameplay (`gameplay`), Interface
//! (its size, and "Enable touch") and Video (`video`). Interface choices go
//! straight into `Preferences`.

mod gameplay;
mod keep;
mod rows;
mod video;

pub(super) use gameplay::{fill_gameplay_page, use_gameplay_page};
pub(super) use keep::{VideoTrial, run_trial, spawn_keep_requester};
pub(super) use video::{VideoForm, VideoShown, fill_video_page, use_video_options, use_video_page};

use bevy::prelude::*;

use crate::common::{InterfaceSize, Preferences};
use crate::widgets::{
    ButtonPressed, ButtonState, Dropdown, DropdownChosen, INK, INTERFACE_PERCENTS, Pressable,
    TabChosen, TextSize, TouchOnly, spawn_check_button, spawn_dropdown, spawn_tabs,
};

const PAGES: [&str; 3] = ["Gameplay", "Interface", "Video"];
const FONT: TextSize = TextSize::Large;
const ROW_GAP: f32 = 12.0;
const LABEL_GAP: f32 = 16.0;
const ROW_HEIGHT: f32 = 36.0;
const DROPDOWN_WIDTH: f32 = 160.0;

/// The tabs' row.
#[derive(Component)]
pub(super) struct PageTabs;

/// A page, by its place among the tabs.
#[derive(Component)]
pub(super) struct Page(usize);

/// The Interface page's contents, built again when what they show changes.
#[derive(Component)]
pub(super) struct InterfacePage;

#[derive(Component)]
pub(super) struct SizeDropdown;

#[derive(Component)]
pub(super) struct TouchButton;

/// The tabs and their pages, in Settings' panel.
pub(super) fn spawn_pages(panel: &mut ChildSpawnerCommands) {
    let tabs = spawn_tabs(panel, &PAGES, 0, FONT);
    panel.commands().entity(tabs).insert(PageTabs);
    for (index, _) in PAGES.iter().enumerate() {
        // Pages not chosen take no room, so the chosen one is always at the top.
        let mut page = panel.spawn((
            Page(index),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(ROW_GAP),
                display: if index == 0 {
                    Display::Flex
                } else {
                    Display::None
                },
                ..default()
            },
        ));
        match PAGES[index] {
            "Gameplay" => {
                page.insert(gameplay::GameplayPage);
            }
            "Interface" => {
                page.insert(InterfacePage);
            }
            "Video" => {
                page.insert(video::VideoPage);
            }
            _ => {}
        }
    }
}

/// Which entry of the dropdown a size is.
fn size_index(size: InterfaceSize) -> usize {
    match size {
        InterfaceSize::Auto => 0,
        InterfaceSize::Percent(percent) => {
            INTERFACE_PERCENTS
                .iter()
                .position(|&offered| offered >= percent)
                .unwrap_or(INTERFACE_PERCENTS.len() - 1)
                + 1
        }
    }
}

/// The size an entry of the dropdown is.
fn size_at(index: usize) -> InterfaceSize {
    index
        .checked_sub(1)
        .and_then(|index| INTERFACE_PERCENTS.get(index))
        .map_or(InterfaceSize::Auto, |&percent| {
            InterfaceSize::Percent(percent)
        })
}

fn size_options() -> Vec<String> {
    std::iter::once("Auto".to_string())
        .chain(
            INTERFACE_PERCENTS
                .iter()
                .map(|percent| format!("{percent}%")),
        )
        .collect()
}

/// Builds the Interface page again when the preferences change (and at the start).
pub(super) fn fill_interface_page(
    mut commands: Commands,
    preferences: Res<Preferences>,
    touch_only: Res<TouchOnly>,
    page: Single<Entity, With<InterfacePage>>,
    mut built: Local<bool>,
) {
    if *built && !preferences.is_changed() && !touch_only.is_changed() {
        return;
    }
    *built = true;
    let page = *page;
    commands.entity(page).despawn_related::<Children>();
    commands.entity(page).with_children(|page| {
        page.spawn(Node {
            column_gap: px(LABEL_GAP),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((Text::new("Interface size"), FONT.font(), TextColor(INK)));
            let dropdown = Dropdown {
                options: size_options(),
                selected: size_index(preferences.interface_size),
                searchable: false,
                ..default()
            };
            let node = Node {
                width: px(DROPDOWN_WIDTH),
                height: px(ROW_HEIGHT),
                ..default()
            };
            spawn_dropdown(row, dropdown, FONT, node).insert(SizeDropdown);
        });
        // A device with only a touch screen has it on, and can't turn it off.
        let (state, checked) = if touch_only.0 {
            (ButtonState::DisabledSelected, true)
        } else {
            (ButtonState::Normal, preferences.touch)
        };
        let node = Node {
            height: px(ROW_HEIGHT),
            align_self: AlignSelf::FlexStart,
            ..default()
        };
        spawn_check_button(page, state, Some(checked), "Enable touch", FONT, node)
            .insert((TouchButton, Pressable));
        page.spawn((
            Text::new("Keeps the interface at least its own size, for fingers."),
            TextSize::Small.font(),
            TextColor(INK),
        ));
    });
}

/// Shows the page whose tab was chosen, and puts choices into the preferences.
#[allow(clippy::too_many_arguments)]
pub(super) fn use_pages(
    mut tabs: MessageReader<TabChosen>,
    mut chosen: MessageReader<DropdownChosen>,
    mut pressed: MessageReader<ButtonPressed>,
    page_tabs: Query<(), With<PageTabs>>,
    mut pages: Query<(&Page, &mut Node)>,
    size_dropdowns: Query<(), With<SizeDropdown>>,
    touch_buttons: Query<(), With<TouchButton>>,
    mut preferences: ResMut<Preferences>,
) {
    for &TabChosen(row, index) in tabs.read() {
        if page_tabs.contains(row) {
            for (page, mut node) in &mut pages {
                let display = if page.0 == index {
                    Display::Flex
                } else {
                    Display::None
                };
                if node.display != display {
                    node.display = display;
                }
            }
        }
    }
    for &DropdownChosen(dropdown, index) in chosen.read() {
        if size_dropdowns.contains(dropdown) {
            let size = size_at(index);
            if preferences.interface_size != size {
                preferences.interface_size = size;
            }
        }
    }
    for press in pressed.read() {
        if touch_buttons.contains(press.0) {
            preferences.touch = !preferences.touch;
        }
    }
}

#[cfg(test)]
mod tests;
