//! The system menu: Esc or the gear button top right opens a small panel with
//! Continue, Settings, Credits, Logout and Quit, and while it is open single
//! player is paused. The same menu, with Play, is the startup menu. See
//! `docs/context/client/system_menu.md`.

mod settings;

use bevy::prelude::*;

use crate::widgets::{TextSize, WidgetSystems};

// Split into files (`cargo xtask lint` keeps files small).
mod connect;
mod credits;
#[cfg(not(target_arch = "wasm32"))]
mod full_screen_key;
mod icons;
mod input;
#[cfg(not(target_arch = "wasm32"))]
mod online;
mod panel;
mod refusal;
#[cfg(not(target_arch = "wasm32"))]
mod server_options;
mod show;
mod spawn;
mod state;

use input::{take_input, takes_input};
use show::{ask_for_pause, show_buttons, show_ended_page, show_menu};
use spawn::spawn_menu;
pub use state::SystemMenu;
use state::{Action, GearButton, Layer, MenuButton, PausedText};

const GEAR_BUTTON: f32 = 34.0;
const CORNER_GAP: f32 = 8.0;

const MENU_WIDTH: f32 = 260.0;
const MARGIN: f32 = 20.0;
/// Above the first button while the game is paused; no room is kept for it
/// otherwise.
const PAUSED_ROW: f32 = 48.0;
const BUTTON_ROW: f32 = 38.0;
const GAP: f32 = 8.0;
/// Between the menu's sections.
const SECTION_GAP: f32 = 24.0;
const FONT: TextSize = TextSize::Large;
const SETTINGS_TITLE_ROW: f32 = 36.0;
/// Round the close button, still on it, for fingers.
const CLOSE_MARGIN: f32 = 4.0;
/// How much of the window's width and height Settings takes (owner's choice).
const SETTINGS_WIDTH: f32 = 50.0;
const SETTINGS_HEIGHT: f32 = 75.0;

const MENU_LAYER: i32 = 0;
const ABOVE_MENU_LAYER: i32 = 1;

pub struct SystemMenuPlugin;

impl Plugin for SystemMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SystemMenu>()
            .init_resource::<connect::ConnectForm>()
            .init_resource::<connect::AddressCheck>()
            .init_resource::<settings::VideoForm>()
            .init_resource::<settings::VideoShown>()
            .init_resource::<settings::VideoTrial>()
            .add_systems(Startup, spawn_menu)
            .add_systems(
                PreUpdate,
                take_input.in_set(WidgetSystems::Front).run_if(takes_input),
            )
            .add_systems(
                Update,
                (
                    show_menu,
                    show_buttons,
                    credits::start_credits,
                    refusal::show_refusal,
                    (
                        connect::open_connect,
                        connect::check_address,
                        connect::use_connect,
                        connect::show_connect,
                        connect::show_mark,
                    )
                        .chain(),
                    connect::fill_server_list,
                    ask_for_pause,
                    settings::fill_interface_page,
                    settings::use_pages,
                    settings::fill_video_page,
                    (settings::use_video_page, settings::run_trial).chain(),
                    settings::use_video_options,
                    settings::fill_gameplay_page,
                    settings::use_gameplay_page,
                ),
            );
        // Alt+Enter, and this game's own server opened to others: native only.
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            PreUpdate,
            full_screen_key::toggle_full_screen
                .after(bevy::input::InputSystems)
                .before(WidgetSystems::Presses),
        )
        .add_systems(
            Update,
            (
                online::use_online,
                online::show_online,
                online::show_online_failure,
                server_options::run_server_options,
                server_options::give_copy_its_font,
            ),
        );
    }
}

#[cfg(test)]
mod tests;
