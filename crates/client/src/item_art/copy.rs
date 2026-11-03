//! Copying each picture off its page into an image of its own, on the graphics
//! card, once the page is drawn. Everything that shows a picture keeps an image
//! of its own: only the drawing is shared.

use bevy::prelude::*;
use bevy::render::Extract;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{CommandEncoderDescriptor, Extent3d, Origin3d};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;

/// The copies to make this frame, after drawing: from where on which page, to which
/// picture. Set again every frame.
#[derive(Resource, Default, Clone)]
pub(super) struct PageCopies(pub(super) Vec<(AssetId<Image>, URect, AssetId<Image>)>);

/// Takes this frame's copies to the world that renders.
pub(super) fn extract_copies(mut commands: Commands, copies: Extract<Res<PageCopies>>) {
    commands.insert_resource(copies.clone());
}

/// Runs after the frame is drawn, so the pages are ready.
pub(super) fn copy_pictures(
    copies: Res<PageCopies>,
    images: Res<RenderAssets<GpuImage>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    if copies.0.is_empty() {
        return;
    }
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
    for (page, rect, picture) in &copies.0 {
        let (Some(page), Some(picture)) = (images.get(*page), images.get(*picture)) else {
            continue;
        };
        let mut from = page.texture.as_image_copy();
        from.origin = Origin3d {
            x: rect.min.x,
            y: rect.min.y,
            z: 0,
        };
        let size = Extent3d {
            width: rect.width(),
            height: rect.height(),
            depth_or_array_layers: 1,
        };
        encoder.copy_texture_to_texture(from, picture.texture.as_image_copy(), size);
    }
    queue.submit([encoder.finish()]);
}
