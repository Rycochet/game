//! Where the player is pointing, and what fingers did.

use bevy::prelude::*;

/// Where `PointerPosition` and `TouchGesture` are written, in `PreUpdate`.
/// Systems reading them there run after this.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointerSystems;

/// Inside `PointerSystems`, after the pointer is read from the mouse and
/// fingers: where dev tools may put a pointer of their own in its place, so
/// the rest of the game takes it for the mouse. Empty unless they do.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointerOverride;

/// The point the player is pointing at, in logical window pixels: the finger
/// while one is on the screen, otherwise the mouse. Read this rather than
/// `Window::cursor_position`, which knows nothing of fingers.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct PointerPosition(pub Option<Vec2>);

/// `PointerPosition` in the interface's own pixels, which `UiScale` makes bigger
/// or smaller than the window's: what interface pieces (`Node`s) are placed and
/// sized in. Hit tests and anything placed at the pointer use this.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct UiPointer(pub Option<Vec2>);

/// A point or distance in logical window pixels, in the interface's own pixels.
pub fn in_interface(at: Vec2, scale: &UiScale) -> Vec2 {
    at / scale.0
}

/// The window's size in the interface's own pixels: how much room interface
/// pieces have.
pub fn interface_area(window: &Window, scale: &UiScale) -> Vec2 {
    in_interface(window.size(), scale)
}

/// What fingers did this frame, other than tap. A system using it may set it
/// back to the default so that later ones don't use it as well.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct TouchGesture {
    /// How far one finger slid, in logical pixels.
    pub drag: Vec2,
    /// Where the sliding finger landed, in logical window pixels.
    pub drag_start: Vec2,
    /// How much the gap between two fingers grew: 1 when it didn't.
    pub pinch: f32,
    /// How far two fingers turned, in radians, clockwise on the screen.
    pub twist: f32,
    /// How far two fingers moved down the screen together, in logical pixels.
    pub tilt: f32,
    /// The point between two fingers, in logical window pixels.
    pub centre: Vec2,
    /// Where the first finger of this contact is, while it stays down, however
    /// many have joined it.
    pub first_finger: Option<Vec2>,
    /// Another finger touched and lifted, while the first stayed down.
    pub other_finger_tapped: bool,
    /// This frame's left press is a tap, not the mouse.
    pub tap: bool,
    /// Where the one finger down landed, once it has stayed there a moment.
    pub still_at: Option<Vec2>,
}

impl Default for TouchGesture {
    fn default() -> Self {
        Self {
            drag: Vec2::ZERO,
            drag_start: Vec2::ZERO,
            pinch: 1.0,
            twist: 0.0,
            tilt: 0.0,
            centre: Vec2::ZERO,
            first_finger: None,
            other_finger_tapped: false,
            tap: false,
            still_at: None,
        }
    }
}
