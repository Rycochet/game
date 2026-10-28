//! A camera that circles a point, turned and tilted by dragging with the right
//! mouse button and zoomed with the wheel. Nothing here is specific to the
//! world map, so other 3D views can use it too.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

/// Turn or tilt per pixel of mouse movement while right-dragging.
const RADIANS_PER_PIXEL: f32 = 0.005;

/// Lowest and highest camera angle above the horizon. Straight down (90°)
/// is avoided: the camera's "up" would be undefined there.
const MIN_PITCH_DEGREES: f32 = 10.0;
const MAX_PITCH_DEGREES: f32 = 85.0;

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
    /// Angle above the horizon, in radians.
    pub pitch: f32,
    /// Distance from the point looked at.
    pub distance: f32,
    /// How far the camera looks from the target towards itself (along the
    /// ground), as a fraction of the distance at the horizon. It shrinks as
    /// the camera zooms in or looks down, so the target stays in view.
    look_ahead: f32,
    /// The starting view, which `reset` returns to: (yaw, pitch, distance).
    home: (f32, f32, f32),
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
        }
    }

    /// Back to the starting view.
    pub fn reset(&mut self) {
        (self.yaw, self.pitch, self.distance) = self.home;
    }

    /// The camera's transform for looking at `target`.
    pub fn transform(&self, target: Vec3) -> Transform {
        let turn = Quat::from_rotation_y(self.yaw);
        let ahead = self.look_ahead * self.distance * self.pitch.cos();
        let look_at = target + turn * Vec3::new(0.0, 0.0, ahead);
        let offset = turn * Vec3::new(0.0, self.pitch.sin(), self.pitch.cos()) * self.distance;
        Transform::from_translation(look_at + offset).looking_at(look_at, Vec3::Y)
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
        self.pitch = (self.pitch + pixels.y * RADIANS_PER_PIXEL).clamp(
            MIN_PITCH_DEGREES.to_radians(),
            MAX_PITCH_DEGREES.to_radians(),
        );
    }

    fn zoom_by(&mut self, notches: f32) {
        let home = self.home.2;
        self.distance = (self.distance * ZOOM_PER_NOTCH.powf(notches)).clamp(home * MIN_ZOOM, home);
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
/// the wheel zooms, and Home returns to the starting view.
pub fn control_with_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut cameras: Query<&mut OrbitCamera>,
) {
    let dragging = buttons.pressed(MouseButton::Right) && motion.delta != Vec2::ZERO;
    let notches = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / PIXELS_PER_NOTCH,
    };
    let reset = keys.just_pressed(KeyCode::Home);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn camera(yaw_degrees: f32) -> OrbitCamera {
        let mut camera = OrbitCamera::new(30f32.to_radians(), 100.0, 0.0);
        camera.yaw = yaw_degrees.to_radians();
        camera
    }

    #[test]
    fn default_view_is_from_the_south() {
        let transform = camera(0.0).transform(Vec3::ZERO);
        assert!(transform.translation.z > 0.0);
        assert!(transform.translation.x.abs() < 1e-3);
        assert!((transform.translation.length() - 100.0).abs() < 1e-3);
    }

    #[test]
    fn quarter_turns_match_the_side_the_camera_is_on() {
        // Camera east of the target, looking west: west is at the top.
        assert!(camera(90.0).transform(Vec3::ZERO).translation.x > 0.0);
        assert_eq!(camera(90.0).quarter_turns(), 3);
        assert_eq!(camera(0.0).quarter_turns(), 0);
        assert_eq!(camera(-90.0).quarter_turns(), 1);
        assert_eq!(camera(180.0).quarter_turns(), 2);
        assert_eq!(camera(350.0).quarter_turns(), 0);
    }

    #[test]
    fn ground_axes_point_right_and_towards_the_camera() {
        let camera = camera(90.0);
        let (right, towards) = camera.ground_axes();
        let position = camera.transform(Vec3::ZERO).translation;
        assert!(towards.dot(Vec2::new(position.x, position.z)) > 0.0);
        // Looking west, right is north (−z).
        assert!((right - Vec2::new(0.0, -1.0)).length() < 1e-5);
    }

    #[test]
    fn tilt_and_zoom_stay_in_range_and_reset() {
        let mut camera = camera(0.0);
        camera.turn(Vec2::new(0.0, 10_000.0));
        assert_eq!(camera.pitch, MAX_PITCH_DEGREES.to_radians());
        camera.zoom_by(-100.0);
        assert_eq!(camera.distance, 100.0);
        camera.zoom_by(100.0);
        assert!((camera.zoom() - MIN_ZOOM).abs() < 1e-6);
        camera.reset();
        assert_eq!((camera.pitch, camera.distance), (30f32.to_radians(), 100.0));
    }
}
