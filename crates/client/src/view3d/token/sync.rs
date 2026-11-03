//! Creating, restyling and removing token views, their icons, and resizing them.

use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::camera::OrbitCamera;
use super::{IconAnchor, TokenAssets, TokenView, outline_width, prism};
use crate::common::{Icon, Token, TokenShape};

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

pub(super) fn spawn_view(
    commands: &mut Commands,
    assets: &TokenAssets,
    owner: Entity,
    token: &Token,
) {
    let (shape, completed) = (token.shape as usize, token.completed as usize);
    let base = if token.no_base {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    let body = commands
        .spawn((
            Mesh3d(assets.bodies[shape].clone()),
            MeshMaterial3d(assets.colours[completed][token.attitude as usize].clone()),
            base,
        ))
        .id();
    let outline = commands
        .spawn((
            Mesh3d(assets.outlines[shape].clone()),
            MeshMaterial3d(assets.outline[completed].clone()),
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
