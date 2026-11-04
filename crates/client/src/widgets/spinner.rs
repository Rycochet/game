//! Spinners: a ring with a gap, turning, for something under way that can't
//! say how far it has got. Give a node `Spinner` (and a size), and it gets its
//! picture and turns until it is hidden or despawned.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::INK;

const PIXELS: usize = 64;
/// Turns a second.
const TURNS: f32 = 1.25;
/// How much of the ring is missing, as a share of a turn.
const GAP: f32 = 0.3;

/// A node shown as a turning spinner.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Spinner;

/// The picture every spinner uses, made once.
#[derive(Resource)]
pub(super) struct SpinnerPicture(Handle<Image>);

pub(super) fn make_spinner_picture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let ink = INK.to_srgba().to_u8_array();
    let centre = PIXELS as f32 / 2.0;
    let mut bytes = vec![0; PIXELS * PIXELS * 4];
    for y in 0..PIXELS {
        for x in 0..PIXELS {
            let (dx, dy) = (x as f32 + 0.5 - centre, y as f32 + 0.5 - centre);
            let ring = (centre * 0.6..=centre * 0.95).contains(&dx.hypot(dy));
            // 0 to 1 round the ring, starting at the top.
            let along = (dx.atan2(-dy) / TAU).rem_euclid(1.0);
            if ring && along > GAP {
                let alpha = (along - GAP) / (1.0 - GAP);
                let pixel = [ink[0], ink[1], ink[2], (alpha * 255.0) as u8];
                bytes[(y * PIXELS + x) * 4..][..4].copy_from_slice(&pixel);
            }
        }
    }
    let image = Image::new(
        Extent3d {
            width: PIXELS as u32,
            height: PIXELS as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    commands.insert_resource(SpinnerPicture(images.add(image)));
}

/// New spinners get their picture.
pub(super) fn picture_spinners(
    new: Query<Entity, Added<Spinner>>,
    picture: Res<SpinnerPicture>,
    mut commands: Commands,
) {
    for spinner in &new {
        commands
            .entity(spinner)
            .insert(ImageNode::new(picture.0.clone()));
    }
}

/// Turns the spinners on show, on the real clock (a paused game still waits).
pub(super) fn turn_spinners(
    time: Res<Time<Real>>,
    mut spinners: Query<(&mut UiTransform, &InheritedVisibility), With<Spinner>>,
) {
    let angle = -(time.elapsed_secs() * TURNS).fract() * TAU;
    for (mut transform, visible) in &mut spinners {
        if visible.get() {
            transform.rotation = Rot2::radians(angle);
        }
    }
}

#[cfg(test)]
mod tests;
