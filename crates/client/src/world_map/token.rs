//! Tokens: how entities with a `common::Token` are drawn on the world map.
//!
//! Each token is a flat prism (its shape extruded upwards) in its attitude's
//! colour, with a black outline, resting on the terrain at its owner's exact
//! position. Above it sits an icon anchor that always faces the camera,
//! showing the entity's 2D `Icon` if it has one.
//!
//! The drawn parts live on a separate "view" entity rather than on the owner,
//! so game entities don't carry rendering components.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, FRAC_PI_8, PI};

use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::light::NotShadowCaster;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use bevy::window::PrimaryWindow;

use super::{CAMERA_DISTANCE, CAMERA_FOV_DEGREES, CAMERA_PITCH_DEGREES};
use crate::common::{Attitude, Icon, Token, TokenShape};

/// Token width in grid units: the diameter of the circle every shape fits in.
pub const DIAMETER: f32 = 16.0;

/// Token thickness, as a fraction of the diameter.
const THICKNESS: f32 = 0.15;

/// Half the token's thickness: how far its centre sits above the ground.
pub const HALF_THICKNESS: f32 = DIAMETER * THICKNESS / 2.0;

/// Outline width on screen.
const OUTLINE_PIXELS: f32 = 2.0;

/// A completed token's brightness (0.7: about 30% darker).
const COMPLETED_SHADE: f32 = 0.7;

/// A completed token's opacity. Alpha to coverage only has coarse steps; 0.9
/// looks about half as see-through as 0.7 did.
const COMPLETED_OPACITY: f32 = 0.9;

/// Icon files are this many pixels square...
const ICON_FILE_PIXELS: f32 = 54.0;

/// ...with the main picture in the central square this size.
const ICON_ART_PIXELS: f32 = 32.0;

/// Width of that central square in grid units. Icons scale with the map, like
/// the token itself, so their pixels aren't whole screen pixels (a blockier,
/// retro look).
const ICON_ART_SIZE: f32 = DIAMETER * 1.5;

/// Gap, in icon pixels, between the top of the token and the bottom of the
/// icon's main picture.
const ICON_GAP_PIXELS: f32 = 1.0;

/// Most pictures end in a shadow a few art pixels above the bottom of the
/// central square; the icon is lowered by this many art pixels so characters
/// stand on the token rather than float above it.
const ICON_SHADOW_PIXELS: f32 = 3.0;

/// The root of a drawn token. It never rotates, so children can rely on
/// world axes. `owner` is the entity with the `Token`.
#[derive(Component)]
pub struct TokenView {
    pub owner: Entity,
    body: Entity,
    outline: Entity,
}

/// A point above each token that is turned to face the camera every frame.
/// The icon is its child: a flat quad in its XY plane, so it always shows the
/// same face to the camera, however the map is rotated. `shown` is the icon
/// path currently displayed.
#[derive(Component, Default)]
pub struct IconAnchor {
    shown: Option<String>,
}

/// Meshes and materials shared by all tokens. The outlines, sized in screen
/// pixels, are rebuilt when the window height changes.
#[derive(Resource)]
pub struct TokenAssets {
    /// The window height (physical pixels) the outlines were made for.
    window_height: f32,
    /// Indexed by `TokenShape as usize`.
    bodies: [Handle<Mesh>; TokenShape::ALL.len()],
    outlines: [Handle<Mesh>; TokenShape::ALL.len()],
    /// Indexed by `[completed as usize][Attitude as usize]`.
    colours: [[Handle<StandardMaterial>; Attitude::ALL.len()]; 2],
    /// Indexed by `completed as usize`.
    outline: [Handle<StandardMaterial>; 2],
    /// One square for every icon.
    icon_quad: Handle<Mesh>,
    /// Icon height above the token's centre, in grid units.
    icon_lift: f32,
    /// One material per icon file, made the first time it is shown.
    icon_materials: HashMap<String, Handle<StandardMaterial>>,
}

/// The outline width in grid units that shows as `OUTLINE_PIXELS` in a window
/// `window_height` physical pixels high.
fn outline_width(window_height: f32) -> f32 {
    // Grid units per pixel at the camera's distance.
    let visible_height = 2.0 * CAMERA_DISTANCE * (CAMERA_FOV_DEGREES.to_radians() / 2.0).tan();
    OUTLINE_PIXELS * visible_height / window_height
}

impl TokenAssets {
    /// `window_height` is in physical pixels.
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        window_height: f32,
    ) -> Self {
        let outline = outline_width(window_height);
        // Grid units per icon pixel.
        let icon_pixel = ICON_ART_SIZE / ICON_ART_PIXELS;
        // From the token's centre up to its top, then the gap and half the
        // main picture less its shadow. The quad faces the camera, so heights
        // in it look shortened by the camera's pitch: divide by its cosine.
        let lift_pixels = ICON_GAP_PIXELS + ICON_ART_PIXELS / 2.0 - ICON_SHADOW_PIXELS;
        let icon_lift =
            HALF_THICKNESS + lift_pixels * icon_pixel / CAMERA_PITCH_DEGREES.to_radians().cos();
        Self {
            window_height,
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
            icon_quad: meshes.add(Rectangle::from_length(ICON_FILE_PIXELS * icon_pixel)),
            icon_lift,
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
fn fade(colour: Color, completed: bool) -> StandardMaterial {
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
fn prism(shape: TokenShape, grow: f32) -> Mesh {
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

/// Creates, restyles and removes token views to match the entities that have
/// a `Token`.
pub fn sync_tokens(
    mut commands: Commands,
    assets: Res<TokenAssets>,
    added: Query<(Entity, &Token), Added<Token>>,
    tokens: Query<Ref<Token>>,
    views: Query<(Entity, &TokenView)>,
    mut parts: Query<(&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    for (view, token_view) in &views {
        match tokens.get(token_view.owner) {
            // The owner was despawned or lost its token.
            Err(_) => commands.entity(view).despawn(),
            Ok(token) if token.is_changed() => {
                let (shape, completed) = (token.shape as usize, token.completed as usize);
                if let Ok((mut mesh, mut material)) = parts.get_mut(token_view.body) {
                    mesh.0 = assets.bodies[shape].clone();
                    material.0 = assets.colours[completed][token.attitude as usize].clone();
                }
                if let Ok((mut mesh, mut material)) = parts.get_mut(token_view.outline) {
                    mesh.0 = assets.outlines[shape].clone();
                    material.0 = assets.outline[completed].clone();
                }
            }
            Ok(_) => {}
        }
    }

    for (owner, token) in &added {
        // A token removed and added again between runs keeps its view.
        if views.iter().any(|(_, view)| view.owner == owner) {
            continue;
        }
        spawn_view(&mut commands, &assets, owner, token);
    }
}

fn spawn_view(commands: &mut Commands, assets: &TokenAssets, owner: Entity, token: &Token) {
    let (shape, completed) = (token.shape as usize, token.completed as usize);
    let body = commands
        .spawn((
            Mesh3d(assets.bodies[shape].clone()),
            MeshMaterial3d(assets.colours[completed][token.attitude as usize].clone()),
        ))
        .id();
    let outline = commands
        .spawn((
            Mesh3d(assets.outlines[shape].clone()),
            MeshMaterial3d(assets.outline[completed].clone()),
            NotShadowCaster,
        ))
        .id();
    let anchor = commands
        .spawn((
            IconAnchor::default(),
            Transform::from_xyz(0.0, assets.icon_lift, 0.0),
            Visibility::default(),
        ))
        .id();
    // Starts at the origin; `place_camera_and_tokens` moves it to the owner.
    commands
        .spawn((
            TokenView {
                owner,
                body,
                outline,
            },
            Transform::default(),
            Visibility::default(),
        ))
        .add_children(&[body, outline, anchor]);
}

/// Shows each token owner's `Icon` on its anchor, replacing or removing the
/// picture when the icon changes.
pub fn sync_icons(
    mut commands: Commands,
    mut assets: ResMut<TokenAssets>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    icons: Query<&Icon>,
    views: Query<&TokenView>,
    mut anchors: Query<(Entity, &mut IconAnchor, &ChildOf)>,
) {
    for (anchor, mut shown, child_of) in &mut anchors {
        let Ok(view) = views.get(child_of.parent()) else {
            continue;
        };
        let wanted = icons.get(view.owner).ok().map(|icon| &icon.0);
        if shown.shown.as_ref() == wanted {
            continue;
        }
        commands.entity(anchor).despawn_children();
        shown.shown = wanted.cloned();
        let Some(path) = wanted else {
            continue;
        };
        let material = assets
            .icon_materials
            .entry(path.clone())
            .or_insert_with(|| {
                // Nearest-pixel sampling keeps the pixel art crisp.
                let image = asset_server
                    .load_builder()
                    .with_settings(|settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::nearest();
                    })
                    .load(path.clone());
                materials.add(StandardMaterial {
                    base_color_texture: Some(image),
                    // Unlit: the picture looks as drawn whatever the sun does.
                    unlit: true,
                    // Pixels are either drawn or not, so no sorting problems.
                    alpha_mode: AlphaMode::Mask(0.5),
                    ..default()
                })
            })
            .clone();
        let quad = assets.icon_quad.clone();
        commands.entity(anchor).with_child((
            Mesh3d(quad),
            MeshMaterial3d(material),
            NotShadowCaster,
        ));
    }
}

/// When the window's height changes, rebuilds the outline meshes (keeping
/// their handles, so every token picks them up) so outlines stay
/// `OUTLINE_PIXELS` wide. Icons scale with the map, so they are left alone.
pub fn resize_tokens(
    window: Single<&Window, With<PrimaryWindow>>,
    mut assets: ResMut<TokenAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let height = window.physical_height() as f32;
    // Minimised windows have no height; keep the old sizes.
    if height == 0.0 || height == assets.window_height {
        return;
    }
    let outline = outline_width(height);
    for shape in TokenShape::ALL {
        if let Some(mut mesh) = meshes.get_mut(&assets.outlines[shape as usize]) {
            *mesh = prism(shape, outline);
        }
    }
    assets.window_height = height;
}
