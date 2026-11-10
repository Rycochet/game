//! Giving the window its icon, once winit has made the window.
//!
//! Bevy has no setting for a window's icon, so this asks winit directly. On
//! Windows the icon comes from the program file itself (put there by
//! `build.rs`), so Windows can pick the best of its sizes for the title bar
//! and the taskbar. Elsewhere it is a PNG built into the program. macOS
//! ignores window icons (only an app bundle has one).

use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WINIT_WINDOWS;
use winit::window::Icon;

/// Sets the icon on the first frame the window exists, then never again.
pub(super) fn set_window_icon(
    window: Option<Single<Entity, With<PrimaryWindow>>>,
    mut done: Local<bool>,
    // On the main thread, where winit's windows are.
    _main_thread: NonSendMarker,
) {
    let Some(window) = window.filter(|_| !*done) else {
        return;
    };
    WINIT_WINDOWS.with_borrow(|windows| {
        let Some(winit) = windows.get_window(*window) else {
            return;
        };
        *done = true;
        match icon() {
            Ok(icon) => {
                #[cfg(windows)]
                {
                    use winit::platform::windows::WindowExtWindows;
                    winit.set_taskbar_icon(Some(icon.clone()));
                }
                winit.set_window_icon(Some(icon));
            }
            Err(error) => warn!("No window icon: {error}"),
        }
    });
}

/// The icon from the program file's resources (id 1, as `build.rs` gives it).
#[cfg(windows)]
fn icon() -> Result<Icon, String> {
    use winit::platform::windows::IconExtWindows;
    Icon::from_resource(1, None).map_err(|e| e.to_string())
}

/// The icon from a PNG built into the program.
#[cfg(not(windows))]
fn icon() -> Result<Icon, String> {
    use bevy::asset::RenderAssetUsages;
    use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};

    const PNG: &[u8] = include_bytes!("../../assets/app_icon/icon-128.png");
    let image = Image::from_buffer(
        PNG,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::Default,
        RenderAssetUsages::MAIN_WORLD,
    )
    .map_err(|e| e.to_string())?;
    let rgba = image
        .try_into_dynamic()
        .map_err(|e| e.to_string())?
        .into_rgba8();
    let (width, height) = rgba.dimensions();
    Icon::from_rgba(rgba.into_raw(), width, height).map_err(|e| e.to_string())
}
