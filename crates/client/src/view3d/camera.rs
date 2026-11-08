//! A camera that circles a point, turned and tilted by dragging with the right
//! mouse button and zoomed with the wheel, or by two fingers, or turned and
//! zoomed by keys (`keys.rs`). Shared by every 3D view.

mod keys;

pub use keys::control_with_keys;

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use crate::common::{Action, Preferences, TouchGesture};

/// Turn or tilt per pixel of mouse movement while right-dragging.
const RADIANS_PER_PIXEL: f32 = 0.005;

/// Lowest and highest camera angle above the horizon. Straight down (90°)
/// is avoided: the camera's "up" would be undefined there.
const MIN_PITCH_DEGREES: f32 = 10.0;
pub(super) const MAX_PITCH_DEGREES: f32 = 85.0;

/// Closest zoom, as a fraction of the starting distance (owner's choice). The
/// starting view is also the furthest out.
const MIN_ZOOM: f32 = 0.6;

/// Distance multiplier per wheel notch (scrolling towards the screen zooms
/// in).
const ZOOM_PER_NOTCH: f32 = 0.9;

/// Touchpads scroll in pixels rather than notches; this many make one notch.
const PIXELS_PER_NOTCH: f32 = 100.0;

/// Where a camera sits relative to the point it looks at. Put it on a camera
/// entity and call `transform` each frame with the point to look at.
#[derive(Component, Clone, Copy, Debug)]
pub struct OrbitCamera {
    /// Turn around the vertical axis, in radians. 0 is south of the target
    /// looking north; positive turns the camera anticlockwise seen from above
    /// (towards the east).
    pub yaw: f32,
    /// Angle above the horizon, in radians: the player's setting. What is
    /// shown may be higher (`shown_pitch`), to keep over raised ground.
    pub pitch: f32,
    /// Distance from the point looked at.
    pub distance: f32,
    /// How far the camera looks from the target towards itself (along the
    /// ground), as a fraction of the distance at the horizon. It shrinks as
    /// the camera zooms in or looks down, so the target stays in view.
    look_ahead: f32,
    /// The starting view, which `reset` returns to: (yaw, pitch, distance).
    home: (f32, f32, f32),
    /// How far out it may go, as a multiple of the starting distance: 1, or
    /// more where a view wants to show further (`set_reach`).
    reach: f32,
    /// The lowest it may tilt, in radians: `MIN_PITCH_DEGREES`, or more where
    /// a view wants it (`set_lowest_pitch`).
    lowest: f32,
    /// How much higher it is shown than `pitch`, in radians, so the line to
    /// the player clears raised wilderness ground (`clearance`); 0 elsewhere.
    raise: f32,
}

impl OrbitCamera {
    /// A camera looking north from `pitch` (radians) above the horizon.
    /// `look_ahead` is in the same units as `distance`, at this starting
    /// view.
    pub fn new(pitch: f32, distance: f32, look_ahead: f32) -> Self {
        Self {
            yaw: 0.0,
            pitch,
            distance,
            look_ahead: look_ahead / (distance * pitch.cos()),
            home: (0.0, pitch, distance),
            reach: 1.0,
            lowest: MIN_PITCH_DEGREES.to_radians(),
            raise: 0.0,
        }
    }

    /// Back to the starting view, as far out as it may go.
    pub fn reset(&mut self) {
        (self.yaw, self.pitch, self.distance) = self.home;
        self.distance *= self.reach;
        self.raise = 0.0;
    }

    /// Lets it go `reach` times as far out as the starting view, and goes as
    /// far out as that (the Hall of Worlds shows further); 1 is usual.
    pub fn set_reach(&mut self, reach: f32) {
        self.reach = reach;
        self.distance = self.home.2 * reach;
    }

    /// How far out it may go, as a multiple of the starting distance.
    pub fn reach(&self) -> f32 {
        self.reach
    }

    /// Lets it tilt no lower than `degrees` above the horizon (`None`: the
    /// usual lowest), raising it if it is lower now. The Hall of Worlds keeps
    /// the floor running off the top of the screen this way.
    pub fn set_lowest_pitch(&mut self, degrees: Option<f32>) {
        self.lowest = degrees.unwrap_or(MIN_PITCH_DEGREES).to_radians();
        self.pitch = self.pitch.max(self.lowest);
    }

    /// The camera's transform for looking at `target`, at the pitch shown.
    pub fn transform(&self, target: Vec3) -> Transform {
        self.transform_at(self.shown_pitch(), target)
    }

    /// The same at any pitch, in radians.
    pub(super) fn transform_at(&self, pitch: f32, target: Vec3) -> Transform {
        let turn = Quat::from_rotation_y(self.yaw);
        let ahead = self.look_ahead * self.distance * pitch.cos();
        let look_at = target + turn * Vec3::new(0.0, 0.0, ahead);
        let offset = turn * Vec3::new(0.0, pitch.sin(), pitch.cos()) * self.distance;
        Transform::from_translation(look_at + offset).looking_at(look_at, Vec3::Y)
    }

    /// The angle above the horizon shown, in radians: the setting, raised
    /// over high ground.
    pub fn shown_pitch(&self) -> f32 {
        (self.pitch + self.raise).min(MAX_PITCH_DEGREES.to_radians())
    }

    /// How much higher it is shown than the setting, in radians.
    pub(super) fn raise(&self) -> f32 {
        self.raise
    }

    pub(super) fn set_raise(&mut self, raise: f32) {
        self.raise = raise.max(0.0);
    }

    /// The camera's right and towards-the-camera directions along the ground,
    /// as (x, z) pairs. A square lined up with these stays square on screen.
    pub fn ground_axes(&self) -> (Vec2, Vec2) {
        let (sin, cos) = self.yaw.sin_cos();
        (Vec2::new(cos, -sin), Vec2::new(sin, cos))
    }

    /// The compass direction nearest the top of the screen, in quarter turns
    /// clockwise from north (see `common::ViewFacing`).
    pub fn quarter_turns(&self) -> u8 {
        // The camera turning east (positive yaw) puts west at the top.
        (-self.yaw / FRAC_PI_2).round().rem_euclid(4.0) as u8
    }

    fn turn(&mut self, pixels: Vec2) {
        // Dragging right swings the near side of the scene to the right, like
        // grabbing the map, so the camera moves the other way. Dragging down
        // pulls the near side down, so the camera looks down more steeply.
        self.yaw = (self.yaw - pixels.x * RADIANS_PER_PIXEL).rem_euclid(TAU);
        if pixels.y != 0.0 {
            // Tilting starts from what is shown, which becomes the setting.
            self.pitch = (self.shown_pitch() + pixels.y * RADIANS_PER_PIXEL)
                .clamp(self.lowest, MAX_PITCH_DEGREES.to_radians());
            self.raise = 0.0;
        }
    }

    fn zoom_by(&mut self, notches: f32) {
        let home = self.home.2;
        self.distance = (self.distance * ZOOM_PER_NOTCH.powf(notches))
            .clamp(home * MIN_ZOOM, home * self.reach);
    }

    /// Moves in by a pinch: fingers twice as far apart halve the distance.
    fn zoom_by_pinch(&mut self, pinch: f32) {
        let home = self.home.2;
        self.distance = (self.distance / pinch).clamp(home * MIN_ZOOM, home * self.reach);
    }

    /// Turns by an angle in radians, clockwise on the screen: the map follows
    /// the fingers. (Positive yaw swings the camera anticlockwise seen from
    /// above, which turns the scene clockwise.)
    fn twist(&mut self, radians: f32) {
        self.yaw = (self.yaw + radians).rem_euclid(TAU);
    }

    /// The distance as a fraction of the starting one: 1 in the starting
    /// view (the furthest out), smaller when zoomed in.
    pub fn zoom(&self) -> f32 {
        self.distance / self.home.2
    }

    /// The starting pitch, in radians.
    pub fn home_pitch(&self) -> f32 {
        self.home.1
    }
}

/// Right-drag turns (sideways) and tilts (up and down) every `OrbitCamera`,
/// the wheel zooms, and Reset View (Home) returns to the starting view.
pub fn control_with_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut cameras: Query<&mut OrbitCamera>,
) {
    let dragging = buttons.pressed(MouseButton::Right) && motion.delta != Vec2::ZERO;
    let notches = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / PIXELS_PER_NOTCH,
    };
    let reset = preferences.controls.just_pressed(&keys, Action::ResetView);
    if !(dragging || notches != 0.0 || reset) {
        // Leave the cameras untouched, so their change detection stays quiet.
        return;
    }
    for mut camera in &mut cameras {
        if reset {
            camera.reset();
            continue;
        }
        if dragging {
            camera.turn(motion.delta);
        }
        if notches != 0.0 {
            camera.zoom_by(notches);
        }
    }
}

/// Two fingers do what the mouse does: pinching zooms, twisting turns, and
/// moving both up or down tilts, in the same direction as right-dragging.
pub fn control_with_touch(gesture: Res<TouchGesture>, mut cameras: Query<&mut OrbitCamera>) {
    if gesture.pinch == 1.0 && gesture.twist == 0.0 && gesture.tilt == 0.0 {
        // Leave the cameras untouched, so their change detection stays quiet.
        return;
    }
    for mut camera in &mut cameras {
        camera.zoom_by_pinch(gesture.pinch);
        camera.twist(gesture.twist);
        camera.turn(Vec2::new(0.0, gesture.tilt));
    }
}

#[cfg(test)]
mod tests;
