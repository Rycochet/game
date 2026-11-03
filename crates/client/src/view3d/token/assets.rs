//! Meshes and materials shared by all tokens, and outline widths.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, FRAC_PI_8, PI};

use super::super::CAMERA_FOV_DEGREES;
use super::{
    COMPLETED_OPACITY, COMPLETED_SHADE, DIAMETER, FEET_ABOVE_EDGE_PIXELS, HALF_THICKNESS,
    ICON_ART_PIXELS, ICON_FILE_PIXELS, ICON_GAP_PIXELS, ICON_PIXEL, ICON_SHADOW_PIXELS, ICON_SLIDE,
    OUTLINE_PIXELS,
};
use crate::common::{Attitude, TokenShape};

/// Meshes and materials shared by all tokens. The outlines, sized in screen
/// pixels, are rebuilt when the window height or the camera's zoom changes.
#[derive(Resource)]
pub struct TokenAssets {
    /// The window height (physical pixels) and camera distance the outlines
    /// were made for.
    pub(super) window_height: f32,
    pub(super) camera_distance: f32,
    /// Indexed by `TokenShape as usize`.
    pub(super) bodies: [Handle<Mesh>; TokenShape::ALL.len()],
    pub(super) outlines: [Handle<Mesh>; TokenShape::ALL.len()],
    /// Indexed by `[completed as usize][Attitude as usize]`.
    pub(super) colours: [[Handle<StandardMaterial>; Attitude::ALL.len()]; 2],
    /// Indexed by `completed as usize`.
    pub(super) outline: [Handle<StandardMaterial>; 2],
    /// One square for every icon.
    pub(super) icon_quad: Handle<Mesh>,
    /// One material per icon file, made the first time it is shown.
    pub(super) icon_materials: HashMap<String, Handle<StandardMaterial>>,
}

/// The outline width in grid units that shows as `OUTLINE_PIXELS` in a window
/// `window_height` physical pixels high, seen from `camera_distance`.
pub(super) fn outline_width(window_height: f32, camera_distance: f32) -> f32 {
    // Grid units per pixel at the camera's distance.
    let visible_height = 2.0 * camera_distance * (CAMERA_FOV_DEGREES.to_radians() / 2.0).tan();
    OUTLINE_PIXELS * visible_height / window_height
}

impl TokenAssets {
    /// Where a token's icon anchor (the centre of its main picture) goes,
    /// relative to the token's centre: on the token's top, then up the screen
    /// (along the camera's "up"). In the starting view (`home_pitch`) the
    /// feet stand `ICON_GAP_PIXELS` above the top's centre; as `pitch` rises
    /// to straight down they slide `ICON_SLIDE` of the way to
    /// `FEET_ABOVE_EDGE_PIXELS` above the token's near edge. `scale` is the
    /// icon's size (`TokenSize::scale`).
    pub fn icon_offset(
        &self,
        camera_rotation: Quat,
        pitch: f32,
        home_pitch: f32,
        scale: f32,
    ) -> Vec3 {
        // Grid units per screen pixel at the camera's distance, and per icon
        // art pixel at this size.
        let screen_pixel = outline_width(self.window_height, self.camera_distance) / OUTLINE_PIXELS;
        let icon_pixel = ICON_PIXEL * scale;
        let feet_at_start = ICON_GAP_PIXELS * icon_pixel;
        // The near edge of the token (with its outline) is this far up the
        // screen from the top's centre: below it, and further below the more
        // the camera looks down.
        let near_edge = -(DIAMETER / 2.0 + OUTLINE_PIXELS * screen_pixel) * pitch.sin();
        let feet_looking_down = near_edge + FEET_ABOVE_EDGE_PIXELS * screen_pixel;
        // 0 at the starting tilt (or lower), 1 looking straight down.
        let towards_down =
            ((pitch.sin() - home_pitch.sin()) / (1.0 - home_pitch.sin())).clamp(0.0, 1.0);
        let feet = feet_at_start + (feet_looking_down - feet_at_start) * towards_down * ICON_SLIDE;
        // From the feet up to the centre of the main picture.
        let centre = feet + (ICON_ART_PIXELS / 2.0 - ICON_SHADOW_PIXELS) * icon_pixel;
        // The icon stands over the token's centre, so its lower part would be
        // inside the token and hidden by the token's near half. Bring it
        // towards the camera (the camera's +Z, which barely moves it on
        // screen) until it is in front of the whole token: the near edge of
        // the top, with its outline, is `radius × cos(pitch)` nearer the
        // camera than the top's centre. Plus a little, so looking straight
        // down it lies just above the top rather than in it.
        let radius = DIAMETER / 2.0 + OUTLINE_PIXELS * screen_pixel;
        let forward = radius * pitch.cos() + HALF_THICKNESS;
        Vec3::Y * HALF_THICKNESS + camera_rotation * Vec3::new(0.0, centre, forward)
    }

    /// `window_height` is in physical pixels.
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        window_height: f32,
        camera_distance: f32,
    ) -> Self {
        let outline = outline_width(window_height, camera_distance);
        Self {
            window_height,
            camera_distance,
            bodies: TokenShape::ALL.map(|shape| meshes.add(prism(shape, 0.0))),
            outlines: TokenShape::ALL.map(|shape| meshes.add(prism(shape, outline))),
            colours: [false, true].map(|completed| {
                Attitude::ALL.map(|attitude| {
                    // Lit, so the sun shades the top and sides differently and
                    // it reads as 3D.
                    materials.add(StandardMaterial {
                        perceptual_roughness: 0.6,
                        ..fade(attitude.colour(), completed)
                    })
                })
            }),
            // Unlit (always black), and only the inside faces are drawn: the
            // "inverted hull" shows as a rim round the coloured shape.
            outline: [false, true].map(|completed| {
                materials.add(StandardMaterial {
                    unlit: true,
                    cull_mode: Some(Face::Front),
                    ..fade(Color::BLACK, completed)
                })
            }),
            icon_quad: meshes.add(Rectangle::from_length(ICON_FILE_PIXELS * ICON_PIXEL)),
            icon_materials: HashMap::default(),
        }
    }
}

/// A material in `colour`, darkened and made see-through if `completed`.
///
/// Completed tokens use alpha to coverage: with multisampling, alpha decides
/// how many of each pixel's samples are drawn, and drawn samples still hide
/// what is behind them. Ordinary blending would show the outline's inside
/// (directly behind the token) through it, turning it black.
pub(super) fn fade(colour: Color, completed: bool) -> StandardMaterial {
    if !completed {
        return StandardMaterial {
            base_color: colour,
            ..default()
        };
    }
    // Darken the sRGB values (as on screen), so it looks about 30% darker.
    let Srgba {
        red, green, blue, ..
    } = colour.to_srgba();
    let dark = Color::srgba(
        red * COMPLETED_SHADE,
        green * COMPLETED_SHADE,
        blue * COMPLETED_SHADE,
        COMPLETED_OPACITY,
    );
    StandardMaterial {
        base_color: dark,
        alpha_mode: AlphaMode::AlphaToCoverage,
        ..default()
    }
}

/// The token's shape extruded to its thickness, standing on the XZ plane and
/// centred on the origin. `grow` makes it that much bigger all round (for the
/// outline).
pub(super) fn prism(shape: TokenShape, grow: f32) -> Mesh {
    let radius = DIAMETER / 2.0;
    let height = 2.0 * (HALF_THICKNESS + grow);
    // Sides, and a turn about the vertical. Polygons start with a corner
    // pointing north: the square turns to line up with the grid, the hexagon
    // and octagon to put a flat side towards the camera.
    let (sides, turn) = match shape {
        TokenShape::Circle => return Cylinder::new(radius + grow, height).into(),
        TokenShape::Triangle => (3, 0.0),
        TokenShape::Square => (4, FRAC_PI_4),
        TokenShape::Hexagon => (6, FRAC_PI_6),
        TokenShape::Octagon => (8, FRAC_PI_8),
    };
    // Grow the corners further than `grow` so the flat sides move out by
    // `grow`, keeping the outline the same width all round.
    let circumradius = radius + grow / (PI / sides as f32).cos();
    Mesh::from(Extrusion::new(
        RegularPolygon::new(circumradius, sides),
        height,
    ))
    // Extrusions are along Z: stand it up (Z becomes Y), then turn it.
    .rotated_by(Quat::from_rotation_y(turn) * Quat::from_rotation_x(-FRAC_PI_2))
}
