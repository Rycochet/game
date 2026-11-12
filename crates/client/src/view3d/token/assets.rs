//! Meshes and materials shared by all tokens, and outline widths.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, FRAC_PI_8, PI};

use super::super::CAMERA_FOV_DEGREES;
use super::{
    COMPLETED_OPACITY, COMPLETED_SHADE, DIAMETER, HALF_THICKNESS, ICON_ART_PIXELS,
    ICON_FILE_PIXELS, ICON_GAP_PIXELS, ICON_PIXEL, ICON_SHADOW_PIXELS, OUTLINE_PIXELS,
    XrayMaterial,
};
use crate::common::{Attitude, TokenShape};
use shared::creature::Size;

/// Meshes by size, then by shape.
type BySize<T> = [[T; TokenShape::ALL.len()]; Size::ALL.len()];

/// A token base's radius in 3D units for a creature of `size`: to scale by
/// `Size::radius`, Medium's being `DIAMETER / 2`.
pub(super) fn base_radius(size: Size) -> f32 {
    DIAMETER / 2.0 * size.radius() as f32 / Size::Medium.radius() as f32
}

/// Meshes and materials shared by all tokens. The outlines, sized in screen
/// pixels, are rebuilt when the window height or the camera's zoom changes.
#[derive(Resource)]
pub struct TokenAssets {
    /// The window height (physical pixels) and camera distance the outlines
    /// were made for.
    pub(super) window_height: f32,
    pub(super) camera_distance: f32,
    /// Indexed by `[Size as usize][TokenShape as usize]` (`Token::base`).
    pub(super) bodies: BySize<Handle<Mesh>>,
    pub(super) outlines: BySize<Handle<Mesh>>,
    /// The outline drawn through what hides it: as `outlines`, but not
    /// below the ground, where the ground would hide it and the x-ray would
    /// show it as a line under every token.
    pub(super) xray_outlines: BySize<Handle<Mesh>>,
    /// Indexed by `[completed as usize][Attitude as usize]`.
    pub(super) colours: [[Handle<StandardMaterial>; Attitude::ALL.len()]; 2],
    /// Indexed by `completed as usize`.
    pub(super) outline: [Handle<StandardMaterial>; 2],
    /// One square for every icon.
    pub(super) icon_quad: Handle<Mesh>,
    /// One material per icon file, made the first time it is shown.
    pub(super) icon_materials: HashMap<String, Handle<StandardMaterial>>,
    /// A body flashing red (`TokenFlash`).
    pub(super) flash_body: Handle<StandardMaterial>,
    /// Each icon's red silhouette, made the first time it flashes.
    pub(super) flash_icons: HashMap<String, Handle<StandardMaterial>>,
    /// Hidden tokens drawn through what hides them, by attitude: the body's
    /// faint fill and the outline.
    pub(super) xray_fill: [Handle<XrayMaterial>; Attitude::ALL.len()],
    pub(super) xray_outline: [Handle<XrayMaterial>; Attitude::ALL.len()],
    /// Each icon's x-ray silhouette by attitude, made the first time it shows.
    pub(super) xray_icons: HashMap<(String, usize), Handle<XrayMaterial>>,
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
    /// relative to the token's centre: standing upright on the top's centre,
    /// its feet `ICON_GAP_PIXELS` above it. `scale` is the icon's size
    /// (`TokenSize::scale`).
    pub fn icon_offset(scale: f32) -> Vec3 {
        let icon_pixel = ICON_PIXEL * scale;
        let feet = HALF_THICKNESS + ICON_GAP_PIXELS * icon_pixel;
        // From the feet up to the centre of the main picture.
        Vec3::Y * (feet + (ICON_ART_PIXELS / 2.0 - ICON_SHADOW_PIXELS) * icon_pixel)
    }

    /// How an icon anchor turns: upright on the map, turned about the vertical
    /// to face the camera (whose rotation this is). Nothing in the game faces
    /// a way, so pictures always face the screen, but never lean back as the
    /// camera tilts.
    pub fn icon_rotation(camera_rotation: Quat) -> Quat {
        // The quad faces +Z; turn that towards the camera along the ground.
        // The camera never looks straight down, so this is never level.
        let towards_camera = camera_rotation * Vec3::Z;
        Quat::from_rotation_y(towards_camera.x.atan2(towards_camera.z))
    }

    /// `window_height` is in physical pixels.
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        xray: &mut Assets<XrayMaterial>,
        window_height: f32,
        camera_distance: f32,
    ) -> Self {
        let outline = outline_width(window_height, camera_distance);
        Self {
            window_height,
            camera_distance,
            bodies: Size::ALL
                .map(|size| TokenShape::ALL.map(|shape| meshes.add(prism(shape, size, 0.0)))),
            outlines: Size::ALL
                .map(|size| TokenShape::ALL.map(|shape| meshes.add(prism(shape, size, outline)))),
            xray_outlines: Size::ALL.map(|size| {
                TokenShape::ALL.map(|shape| meshes.add(raised_prism(shape, size, outline)))
            }),
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
            flash_body: materials.add(StandardMaterial {
                base_color: super::flash::FLASH_RED,
                unlit: true,
                ..default()
            }),
            flash_icons: HashMap::default(),
            xray_fill: Attitude::ALL
                .map(|attitude| xray.add(XrayMaterial::fill(attitude.colour()))),
            xray_outline: Attitude::ALL
                .map(|attitude| xray.add(XrayMaterial::outline(attitude.colour()))),
            xray_icons: HashMap::default(),
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

/// `prism(shape, grow)` moved up by `grow`, so its bottom is the token's.
pub(super) fn raised_prism(shape: TokenShape, size: Size, grow: f32) -> Mesh {
    prism(shape, size, grow).translated_by(Vec3::Y * grow)
}

/// The token's shape extruded to its thickness, standing on the XZ plane and
/// centred on the origin, as wide as `size`'s base (`base_radius`) and as thick
/// as any. `grow` makes it that much bigger all round (for the outline).
pub(super) fn prism(shape: TokenShape, size: Size, grow: f32) -> Mesh {
    let radius = base_radius(size);
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
