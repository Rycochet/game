//! The camera, window size, interface and cursor.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, params, refused};

use super::super::{Forced, turn_camera};
use super::done;
use crate::common::{InterfaceSize, Preferences};
use crate::view3d::OrbitCamera;

#[derive(Deserialize)]
pub(super) struct ViewParams {
    /// Degrees: 90 puts the camera in the east looking west.
    yaw: Option<f32>,
    /// Degrees above the horizon.
    pitch: Option<f32>,
    /// Divides the camera's present distance.
    zoom: Option<f32>,
}

/// `client.view { "yaw": 90, "pitch": 30, "zoom": 1.5 }` (`HOW_DEBUG_VIEW`):
/// each only if given. Zoom divides the distance it is at now, so it adds up
/// over calls.
pub(super) fn view(
    In(request): In<Option<Value>>,
    mut cameras: Query<&mut OrbitCamera>,
) -> BrpResult {
    let ViewParams { yaw, pitch, zoom } = params(request)?;
    for mut camera in &mut cameras {
        turn_camera(&mut camera, yaw, pitch, zoom);
    }
    done()
}

#[derive(Deserialize)]
pub(super) struct SizeParams {
    width: f32,
    height: f32,
}

/// `client.window_size { "width": 1280, "height": 720 }`, in logical pixels
/// (`HOW_DEBUG_WINDOW_SIZE`).
pub(super) fn window_size(
    In(request): In<Option<Value>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) -> BrpResult {
    let SizeParams { width, height } = params(request)?;
    window.set_maximized(false);
    window.resolution.set(width, height);
    done()
}

#[derive(Deserialize)]
pub(super) struct InterfaceParams {
    /// `"auto"`, or a percentage.
    size: Option<Value>,
    /// "Enable touch".
    touch: Option<bool>,
}

/// `client.interface { "size": 200, "touch": false }`: the interface size and
/// "Enable touch", as Settings sets them (and saves them: run with
/// `HOW_PREFERENCES` elsewhere to keep the player's own). `"size": "auto"` for Auto.
pub(super) fn interface(
    In(request): In<Option<Value>>,
    mut preferences: ResMut<Preferences>,
) -> BrpResult {
    let InterfaceParams { size, touch } = params(request)?;
    if let Some(size) = size {
        preferences.interface_size = match &size {
            Value::String(auto) if auto.eq_ignore_ascii_case("auto") => InterfaceSize::Auto,
            Value::Number(percent) => percent
                .as_u64()
                .and_then(|percent| u16::try_from(percent).ok())
                .map(InterfaceSize::Percent)
                .ok_or_else(|| refused(format!("not a percentage: {percent}")))?,
            other => {
                return Err(refused(format!(
                    "size is \"auto\" or a percentage, not {other}"
                )));
            }
        };
    }
    if let Some(touch) = touch {
        preferences.touch = touch;
    }
    done()
}

#[derive(Deserialize)]
pub(super) struct CursorParams {
    x: Option<f32>,
    y: Option<f32>,
}

/// `client.cursor { "x": 400, "y": 300 }`: holds the mouse cursor there, in
/// logical pixels, until `client.cursor {}` lets it go (`HOW_DEBUG_CURSOR`).
pub(super) fn cursor(In(request): In<Option<Value>>, mut forced: ResMut<Forced>) -> BrpResult {
    let CursorParams { x, y } = params(request)?;
    forced.cursor = x.zip(y).map(|(x, y)| Vec2::new(x, y));
    done()
}
