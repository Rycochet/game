//! Tokens: how entities with a `common::Token` are drawn in the 3D views.
//!
//! Each token is a flat prism (its shape extruded upwards) in its attitude's
//! colour, with a black outline, standing on the ground at its owner's exact
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

use super::camera::OrbitCamera;
use super::{CAMERA_FOV_DEGREES, ViewCamera};
use crate::common::{Attitude, Clickable, Icon, Token, TokenShape};

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

/// Width of that central square in grid units, at `TokenSize::Default` (each
/// token's size scales it). Icons scale with the map, like the token itself,
/// so their pixels aren't whole screen pixels (a blockier, retro look).
const ICON_ART_SIZE: f32 = DIAMETER * 1.5;

/// Gap, in icon pixels, between the top of the token and the bottom of the
/// icon's main picture.
const ICON_GAP_PIXELS: f32 = 1.0;

/// Most pictures end in a shadow a few art pixels above the bottom of the
/// central square; the icon is lowered by this many art pixels so characters
/// stand on the token rather than float above it.
const ICON_SHADOW_PIXELS: f32 = 3.0;

/// Tilting from the starting view to straight down slides the icon's feet
/// (the bottom of the main picture, less its shadow) smoothly down towards a
/// point this many screen pixels above the near edge of the token...
const FEET_ABOVE_EDGE_PIXELS: f32 = 2.0;

/// ...but only this fraction of the way: the full slide looked about double
/// what was needed (owner's judgement).
const ICON_SLIDE: f32 = 0.5;

/// Grid units per icon art pixel, at `TokenSize::Default`.
const ICON_PIXEL: f32 = ICON_ART_SIZE / ICON_ART_PIXELS;

/// The root of a drawn token. It never rotates, so children can rely on
/// world axes. `owner` is the entity with the `Token`.
#[derive(Component)]
pub struct TokenView {
    pub owner: Entity,
    body: Entity,
    outline: Entity,
    anchor: Entity,
}

/// A point above each token that is turned to face the camera every frame.
/// The icon is its child: a flat quad in its XY plane, so it always shows the
/// same face to the camera, however the map is rotated. `shown` is the icon
/// path currently displayed; `scale` the owner's `TokenSize` scale.
#[derive(Component)]
pub struct IconAnchor {
    shown: Option<String>,
    scale: f32,
}

impl IconAnchor {
    /// How big the icon is drawn, relative to `TokenSize::Default`.
    pub fn scale(&self) -> f32 {
        self.scale
    }
}

/// Meshes and materials shared by all tokens. The outlines, sized in screen
/// pixels, are rebuilt when the window height or the camera's zoom changes.
#[derive(Resource)]
pub struct TokenAssets {
    /// The window height (physical pixels) and camera distance the outlines
    /// were made for.
    window_height: f32,
    camera_distance: f32,
    /// Indexed by `TokenShape as usize`.
    bodies: [Handle<Mesh>; TokenShape::ALL.len()],
    outlines: [Handle<Mesh>; TokenShape::ALL.len()],
    /// Indexed by `[completed as usize][Attitude as usize]`.
    colours: [[Handle<StandardMaterial>; Attitude::ALL.len()]; 2],
    /// Indexed by `completed as usize`.
    outline: [Handle<StandardMaterial>; 2],
    /// One square for every icon.
    icon_quad: Handle<Mesh>,
    /// One material per icon file, made the first time it is shown.
    icon_materials: HashMap<String, Handle<StandardMaterial>>,
}

/// The outline width in grid units that shows as `OUTLINE_PIXELS` in a window
/// `window_height` physical pixels high, seen from `camera_distance`.
fn outline_width(window_height: f32, camera_distance: f32) -> f32 {
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
    mut anchors: Query<&mut IconAnchor>,
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
                if let Ok(mut anchor) = anchors.get_mut(token_view.anchor) {
                    anchor.scale = token.size.scale();
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
            IconAnchor {
                shown: None,
                scale: token.size.scale(),
            },
            // Placed each frame by `place_camera_and_tokens`.
            Transform::default(),
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
                anchor,
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

/// When the window's height or the camera's zoom changes, rebuilds the
/// outline meshes (keeping their handles, so every token picks them up) so
/// outlines stay `OUTLINE_PIXELS` wide. Icons scale with the map, so they are
/// left alone.
pub fn resize_tokens(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<&OrbitCamera>,
    mut assets: ResMut<TokenAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let height = window.physical_height() as f32;
    let distance = camera.distance;
    // Minimised windows have no height; keep the old sizes.
    if height == 0.0 || (height == assets.window_height && distance == assets.camera_distance) {
        return;
    }
    let outline = outline_width(height, distance);
    for shape in TokenShape::ALL {
        if let Some(mut mesh) = meshes.get_mut(&assets.outlines[shape as usize]) {
            *mesh = prism(shape, outline);
        }
    }
    assets.window_height = height;
    assets.camera_distance = distance;
}

/// The `Clickable` token the left mouse button was just pressed on, if any
/// (nearest the camera when several overlap). Set every frame in
/// `View3dSystems::Click`; views ignore a click on the ground under it.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClickedToken(pub Option<Entity>);

/// Finds the clickable token under the cursor on a left click: within the
/// token's circle on screen, or its icon's main picture.
pub fn click_tokens(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    views: Query<(&TokenView, &GlobalTransform, &InheritedVisibility)>,
    anchors: Query<(&IconAnchor, &GlobalTransform)>,
    clickable: Query<(), With<Clickable>>,
    mut clicked: ResMut<ClickedToken>,
) {
    clicked.set_if_neq(ClickedToken(None));
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = *camera;
    let right = camera_transform.right();
    let eye = camera_transform.translation();
    // Whether the cursor is within `half` (3D units, across the screen) of a
    // point: as a circle or a square.
    let near = |point: Vec3, half: f32, square: bool| {
        let (Ok(centre), Ok(side)) = (
            camera.world_to_viewport(camera_transform, point),
            camera.world_to_viewport(camera_transform, point + right * half),
        ) else {
            return false;
        };
        let reach = centre.distance(side);
        let offset = cursor - centre;
        if square {
            offset.x.abs() <= reach && offset.y.abs() <= reach
        } else {
            offset.length() <= reach
        }
    };

    let mut best: Option<(f32, Entity)> = None;
    for (view, transform, visibility) in &views {
        if !visibility.get() || clickable.get(view.owner).is_err() {
            continue;
        }
        let base = transform.translation();
        let mut hit = near(base, DIAMETER / 2.0, false);
        if let Ok((anchor, anchor_transform)) = anchors.get(view.anchor) {
            hit |= anchor.shown.is_some()
                && near(
                    anchor_transform.translation(),
                    ICON_ART_SIZE * anchor.scale / 2.0,
                    true,
                );
        }
        let distance = eye.distance(base);
        if hit && best.is_none_or(|(d, _)| distance < d) {
            best = Some((distance, view.owner));
        }
    }
    clicked.0 = best.map(|(_, owner)| owner);
}
