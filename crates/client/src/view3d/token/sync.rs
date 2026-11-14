//! Creating, restyling and removing token views, their icons, and resizing them.

use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::camera::OrbitCamera;
use super::{
    ICON_GAP_PIXELS, ICON_PIXEL, ICON_SHADOW_PIXELS, IconAnchor, ModelAnchor, TokenAssets,
    TokenBase, TokenView, XrayMaterial, base_radius, outline_width, prism, raised_prism,
};
use crate::common::{Icon, Token, TokenModel, TokenShape};

/// Creates, restyles and removes token views to match the entities that have
/// a `Token`.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn sync_tokens(
    mut commands: Commands,
    assets: Res<TokenAssets>,
    added: Query<(Entity, &Token), Added<Token>>,
    tokens: Query<Ref<Token>>,
    mut views: Query<(Entity, &mut TokenView)>,
    mut parts: Query<(&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>)>,
    mut xray_parts: Query<
        (&mut Mesh3d, &mut MeshMaterial3d<XrayMaterial>),
        Without<MeshMaterial3d<StandardMaterial>>,
    >,
    mut anchors: Query<&mut IconAnchor>,
) {
    for (view, mut token_view) in &mut views {
        match tokens.get(token_view.owner) {
            // The owner was despawned or lost its token.
            Err(_) => commands.entity(view).despawn(),
            Ok(token) if token.is_changed() => {
                let (shape, completed) = (token.shape as usize, token.completed as usize);
                let base = token.base as usize;
                token_view.radius = base_radius(token.base);
                if let Ok((mut mesh, mut material)) = parts.get_mut(token_view.body) {
                    mesh.0 = assets.bodies[base][shape].clone();
                    material.0 = assets.colours[completed][token.attitude as usize].clone();
                }
                if let Ok((mut mesh, mut material)) = parts.get_mut(token_view.outline) {
                    mesh.0 = assets.outlines[base][shape].clone();
                    material.0 = assets.outline[completed].clone();
                }
                let attitude = token.attitude as usize;
                if let Ok((mut mesh, mut material)) = xray_parts.get_mut(token_view.xray_body) {
                    mesh.0 = assets.bodies[base][shape].clone();
                    material.0 = assets.xray_fill[attitude].clone();
                }
                if let Ok((mut mesh, mut material)) = xray_parts.get_mut(token_view.xray_outline) {
                    mesh.0 = assets.xray_outlines[base][shape].clone();
                    material.0 = assets.xray_outline[attitude].clone();
                }
                if let Ok(mut anchor) = anchors.get_mut(token_view.anchor) {
                    anchor.scale = token.size.scale();
                    // Shown again, so its x-ray silhouette takes the new colour.
                    anchor.shown = None;
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

pub(super) fn spawn_view(
    commands: &mut Commands,
    assets: &TokenAssets,
    owner: Entity,
    token: &Token,
) {
    let (shape, completed) = (token.shape as usize, token.completed as usize);
    let size = token.base as usize;
    let base = if token.no_base {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    let body = commands
        .spawn((
            Mesh3d(assets.bodies[size][shape].clone()),
            MeshMaterial3d(assets.colours[completed][token.attitude as usize].clone()),
            base,
        ))
        .id();
    let outline = commands
        .spawn((
            Mesh3d(assets.outlines[size][shape].clone()),
            MeshMaterial3d(assets.outline[completed].clone()),
            NotShadowCaster,
            base,
        ))
        .id();
    // Drawn through whatever hides them (`xray`): nothing shows while the
    // token is in plain sight.
    let xray_body = commands
        .spawn((
            Mesh3d(assets.bodies[size][shape].clone()),
            MeshMaterial3d(assets.xray_fill[token.attitude as usize].clone()),
            NotShadowCaster,
            base,
        ))
        .id();
    let xray_outline = commands
        .spawn((
            Mesh3d(assets.xray_outlines[size][shape].clone()),
            MeshMaterial3d(assets.xray_outline[token.attitude as usize].clone()),
            NotShadowCaster,
            base,
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
    let model = commands
        .spawn((
            ModelAnchor::default(),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    // Tilted to the ground by `place_camera_and_tokens`.
    let tilted = commands
        .spawn((TokenBase, Transform::default(), Visibility::default()))
        .add_children(&[body, outline, xray_body, xray_outline])
        .id();
    // Starts at the origin; `place_camera_and_tokens` moves it to the owner.
    commands
        .spawn((
            TokenView {
                owner,
                base: tilted,
                body,
                outline,
                xray_body,
                xray_outline,
                anchor,
                model,
                radius: base_radius(token.base),
                tilt: Quat::IDENTITY,
            },
            Transform::default(),
            Visibility::default(),
        ))
        .add_children(&[tilted, anchor, model]);
}

/// Shows each token owner's `Icon` on its anchor, unless a 3D model stands there
/// (`TokenModel`), replacing or removing the picture when either changes.
#[allow(clippy::too_many_arguments)]
pub fn sync_icons(
    mut commands: Commands,
    mut assets: ResMut<TokenAssets>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut xray: ResMut<Assets<XrayMaterial>>,
    tokens: Query<&Token>,
    icons: Query<(Option<&Icon>, Has<TokenModel>)>,
    views: Query<&TokenView>,
    mut anchors: Query<(Entity, &mut IconAnchor, &ChildOf)>,
) {
    for (anchor, mut shown, child_of) in &mut anchors {
        let Ok(view) = views.get(child_of.parent()) else {
            continue;
        };
        let (icon, model) = icons.get(view.owner).unwrap_or_default();
        // A 3D model stands in place of the icon (`sync_models`).
        let wanted = match icon {
            _ if model => None,
            Some(icon) => Some(icon.0.clone()),
            None => None,
        };
        if shown.shown == wanted {
            continue;
        }
        commands.entity(anchor).despawn_children();
        shown.shown = wanted.clone();
        let Some(key) = wanted else {
            continue;
        };
        let material = assets
            .icon_materials
            .entry(key.clone())
            .or_insert_with_key(|key| {
                // Nearest-pixel sampling keeps the pixel art crisp.
                let image = asset_server
                    .load_builder()
                    .with_settings(|settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::nearest();
                    })
                    .load(key.clone());
                materials.add(StandardMaterial {
                    base_color_texture: Some(image),
                    // Unlit: the picture looks as drawn whatever the sun does.
                    unlit: true,
                    // Blended, after everything solid, so it writes no depth:
                    // drawn into the depth, it would hide its own token's
                    // back half from the x-ray, which would then show
                    // through the picture. Pixel art is all there or not, and
                    // drawn back to front with the rest.
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })
            })
            .clone();
        let quad = assets.icon_quad.clone();
        let scale = 1.0;
        let attitude = tokens
            .get(view.owner)
            .map_or(0, |token| token.attitude as usize);
        let silhouette = assets
            .xray_icons
            .entry((key, attitude))
            .or_insert_with(|| {
                let image = materials
                    .get(&material)
                    .and_then(|usual| usual.base_color_texture.clone())
                    .unwrap_or_default();
                let colour = crate::common::Attitude::ALL[attitude].colour();
                xray.add(XrayMaterial::picture(colour, image))
            })
            .clone();
        commands.entity(anchor).with_children(|icon| {
            icon.spawn((
                Mesh3d(quad.clone()),
                MeshMaterial3d(material),
                NotShadowCaster,
                Transform::from_scale(Vec3::splat(scale)),
            ));
            // Its feet sink into the token, which would hide them from the
            // x-ray and show them over the token: raised clear of it.
            let raised = (ICON_SHADOW_PIXELS - ICON_GAP_PIXELS) * ICON_PIXEL;
            icon.spawn((
                Mesh3d(quad),
                MeshMaterial3d(silhouette),
                NotShadowCaster,
                Transform::from_xyz(0.0, raised, 0.0).with_scale(Vec3::splat(scale)),
            ));
        });
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
    for size in shared::creature::Size::ALL {
        for shape in TokenShape::ALL {
            let (s, t) = (size as usize, shape as usize);
            if let Some(mut mesh) = meshes.get_mut(&assets.outlines[s][t]) {
                *mesh = prism(shape, size, outline);
            }
            if let Some(mut mesh) = meshes.get_mut(&assets.xray_outlines[s][t]) {
                *mesh = raised_prism(shape, size, outline);
            }
        }
    }
    assets.window_height = height;
    assets.camera_distance = distance;
}
