//! Tokens flashing red for a moment when their owner is harmed: the body
//! and its picture, while the black outline stays (owner, 2026-11-07).

use bevy::prelude::*;

use super::{IconAnchor, TokenAssets, TokenView};
use crate::common::Token;

/// How long a flash lasts: a few frames at 60 a second, by the clock so it
/// looks the same at any frame rate. It is always seen for at least a frame.
pub const FLASH_SECONDS: f32 = 0.1;

/// The red a token flashes.
pub(super) const FLASH_RED: Color = Color::srgb(0.9, 0.05, 0.05);

/// On a token's owner: its token is drawn red until the time is up, and then
/// this is taken off. Adding it again starts the flash again.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct TokenFlash {
    left: f32,
}

impl Default for TokenFlash {
    fn default() -> Self {
        Self {
            left: FLASH_SECONDS,
        }
    }
}

impl TokenFlash {
    /// Whether to draw red this frame, counting the frame off; `false` once
    /// the time is up. A flash is drawn at least once, however long the frame.
    pub(super) fn tick(&mut self, seconds: f32) -> bool {
        if self.left <= 0.0 {
            return false;
        }
        self.left -= seconds;
        true
    }
}

/// Draws each token red or as usual, by whether its owner is flashing, after
/// `sync_tokens` and `sync_icons` have chosen its usual materials.
#[allow(clippy::too_many_arguments)]
pub fn flash_tokens(
    mut commands: Commands,
    time: Res<Time>,
    mut assets: ResMut<TokenAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut owners: Query<(&Token, Option<&mut TokenFlash>)>,
    views: Query<&TokenView>,
    anchors: Query<(&IconAnchor, Option<&Children>)>,
    mut parts: Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    let seconds = time.delta_secs();
    for view in &views {
        let Ok((token, flash)) = owners.get_mut(view.owner) else {
            continue;
        };
        let red = match flash {
            Some(mut flash) => {
                let red = flash.tick(seconds);
                if !red {
                    commands.entity(view.owner).remove::<TokenFlash>();
                }
                red
            }
            None => false,
        };
        let body = match red {
            true => assets.flash_body.clone(),
            false => assets.colours[token.completed as usize][token.attitude as usize].clone(),
        };
        set(&mut parts, view.body, body);
        let Ok((anchor, Some(children))) = anchors.get(view.anchor) else {
            continue;
        };
        let Some(key) = &anchor.shown else {
            continue;
        };
        let Some(usual) = assets.icon_materials.get(key).cloned() else {
            continue;
        };
        let icon = match red {
            true => silhouette(&mut assets, &mut materials, key, &usual),
            false => usual,
        };
        for child in children {
            set(&mut parts, *child, icon.clone());
        }
    }
}

/// Gives a part this material, unless it has it already.
fn set(
    parts: &mut Query<&mut MeshMaterial3d<StandardMaterial>>,
    part: Entity,
    material: Handle<StandardMaterial>,
) {
    if let Ok(mut now) = parts.get_mut(part)
        && now.0 != material
    {
        now.0 = material;
    }
}

/// An icon's red silhouette: its picture's shape (the same alpha cut-out),
/// filled with red. Black lit by nothing, glowing red, so the picture's own
/// colours don't show.
fn silhouette(
    assets: &mut TokenAssets,
    materials: &mut Assets<StandardMaterial>,
    key: &str,
    usual: &Handle<StandardMaterial>,
) -> Handle<StandardMaterial> {
    if let Some(made) = assets.flash_icons.get(key) {
        return made.clone();
    }
    let picture = materials
        .get(usual)
        .and_then(|usual| usual.base_color_texture.clone());
    let made = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        base_color_texture: picture,
        emissive: FLASH_RED.into(),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });
    assets.flash_icons.insert(key.to_string(), made.clone());
    made
}

#[cfg(test)]
mod tests;
