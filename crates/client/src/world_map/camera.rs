//! A camera that circles a point at a fixed angle and distance, turned by
//! dragging with the right mouse button. Nothing here is specific to the world
//! map, so other 3D views can use it too.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

/// Turn per pixel of horizontal mouse movement while right-dragging.
const RADIANS_PER_PIXEL: f32 = 0.005;

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
    /// The camera looks this far from the target towards itself (along the
    /// ground), so the near edge of the scene fits on screen.
    pub look_ahead: f32,
}

impl OrbitCamera {
    /// The camera's transform for looking at `target`.
    pub fn transform(&self, target: Vec3) -> Transform {
        let turn = Quat::from_rotation_y(self.yaw);
        let look_at = target + turn * Vec3::new(0.0, 0.0, self.look_ahead);
        let offset = turn * Vec3::new(0.0, self.pitch.sin(), self.pitch.cos()) * self.distance;
        Transform::from_translation(look_at + offset).looking_at(look_at, Vec3::Y)
    }

    /// The compass direction nearest the top of the screen, in quarter turns
    /// clockwise from north (see `common::ViewFacing`).
    pub fn quarter_turns(&self) -> u8 {
        // The camera turning east (positive yaw) puts west at the top.
        (-self.yaw / FRAC_PI_2).round().rem_euclid(4.0) as u8
    }
}

/// Right-drag turns every `OrbitCamera` around its target; Home turns it back
/// to its starting direction (yaw 0).
pub fn turn_with_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    mut cameras: Query<&mut OrbitCamera>,
) {
    for mut camera in &mut cameras {
        if keys.just_pressed(KeyCode::Home) {
            camera.yaw = 0.0;
        } else if buttons.pressed(MouseButton::Right) && motion.delta.x != 0.0 {
            // Dragging right swings the near side of the scene to the right,
            // like grabbing the map, so the camera moves the other way.
            camera.yaw = (camera.yaw - motion.delta.x * RADIANS_PER_PIXEL).rem_euclid(TAU);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera(yaw_degrees: f32) -> OrbitCamera {
        OrbitCamera {
            yaw: yaw_degrees.to_radians(),
            pitch: 30f32.to_radians(),
            distance: 100.0,
            look_ahead: 0.0,
        }
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
}
