//! A warning at the startup menu when the game is drawn without a graphics
//! card (software drawing, such as a browser with hardware acceleration off):
//! it can still be played, just slowly. "Don't Warn Again" is kept in
//! `Preferences::quiet_slow_drawing`.

use bevy::prelude::*;
use bevy::render::render_resource::WgpuAdapterInfo as AdapterInfo;
use bevy::render::renderer::RenderAdapterInfo;
use wgpu_types::DeviceType;

use super::{ABOVE_MENU_LAYER, Action, BUTTON_ROW, FONT, Layer, SystemMenu};
use crate::common::{Preferences, Session};
use crate::widgets::{ButtonState, ButtonStyle, Pressable, spawn_button, spawn_dialog};

const WORDS_WIDTH: f32 = 460.0;

#[cfg(target_arch = "wasm32")]
const WORDS: &str = "Your browser isn't using your graphics card, so the game will \
    run slowly. Turn on hardware acceleration in your browser's settings (in Chrome, \
    Edge and Brave: System, \"Use graphics acceleration when available\"), then \
    restart the browser.";
#[cfg(not(target_arch = "wasm32"))]
const WORDS: &str = "This computer is drawing the game without its graphics card, so \
    the game will run slowly. Updating the graphics card's drivers may help.";

/// Names of software drawing, in lower case: what browsers and drivers call
/// it when there is no graphics card to draw with.
const SOFTWARE_NAMES: [&str; 4] = [
    "swiftshader",
    "llvmpipe",
    "softpipe",
    "microsoft basic render",
];

/// Whether this adapter draws without a graphics card. `page_says`: the web
/// page found the browser would only give a slow WebGL (always false natively).
pub(super) fn is_slow(info: &AdapterInfo, page_says: bool) -> bool {
    let name = info.name.to_lowercase();
    page_says
        || info.device_type == DeviceType::Cpu
        || SOFTWARE_NAMES.iter().any(|soft| name.contains(soft))
}

/// What `index.html` found: WebGL refused with "fail if major performance
/// caveat", which browsers do when they would draw without a graphics card.
#[cfg(target_arch = "wasm32")]
fn page_says_slow() -> bool {
    web_sys::window()
        .and_then(|window| js_sys::Reflect::get(&window, &"howSlowDrawing".into()).ok())
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

#[cfg(not(target_arch = "wasm32"))]
fn page_says_slow() -> bool {
    false
}

pub(super) fn spawn_slow_drawing(commands: &mut Commands) {
    let dialog = spawn_dialog(commands, ABOVE_MENU_LAYER, "Slow drawing");
    commands
        .entity(dialog.layer)
        .insert((Layer(&[SystemMenu::SlowDrawing]), Action::Back));
    commands.entity(dialog.content).with_child((
        Text::new(WORDS),
        FONT.font(),
        Node {
            width: px(WORDS_WIDTH),
            ..default()
        },
    ));
    commands.entity(dialog.buttons).with_children(|row| {
        let node = Node {
            width: px(WORDS_WIDTH / 2.5),
            height: px(BUTTON_ROW),
            ..default()
        };
        spawn_button(row, ButtonState::Normal, "Play Anyway", FONT, node.clone()).insert((
            Action::Back,
            Action::Back.ui_id("slow_drawing"),
            ButtonStyle::Main,
            Pressable,
        ));
        spawn_button(row, ButtonState::Normal, "Don't Warn Again", FONT, node).insert((
            Action::QuietSlowDrawing,
            Action::QuietSlowDrawing.ui_id("slow_drawing"),
            Pressable,
        ));
    });
}

/// Once a run, at the startup menu: the warning, unless it was turned off.
pub(super) fn warn_of_slow_drawing(
    info: Option<Res<RenderAdapterInfo>>,
    preferences: Res<Preferences>,
    session: Res<Session>,
    mut menu: ResMut<SystemMenu>,
    mut warned: Local<bool>,
) {
    if *warned || preferences.quiet_slow_drawing || session.is_playing() {
        return;
    }
    let Some(info) = info else {
        return;
    };
    if *menu != SystemMenu::Menu {
        return;
    }
    *warned = true;
    if is_slow(&info, page_says_slow()) {
        info!(
            "Drawing without a graphics card: {} ({:?})",
            info.name, info.device_type
        );
        *menu = SystemMenu::SlowDrawing;
    }
}

#[cfg(test)]
mod tests;
