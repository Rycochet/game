//! A batch: a page of pictures drawn together by one camera, then given their
//! black edges by another, before each is copied off into its own image.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use std::collections::HashMap;

use super::sheet::Shelves;
use super::{EdgeMaterial, FRAMES_ALIVE, MOST_DETAIL, OUTLINE_PIXELS, PAGE_PIXELS};

/// Room between pictures on a page, more than any edge reaches.
const GAP: u32 = OUTLINE_PIXELS as u32 * MOST_DETAIL * 2 + 2;

/// Everything for one page while it is being drawn, under one entity.
#[derive(Component)]
pub(super) struct Batch {
    /// Which of the `PAGES` it is, for its render layers.
    pub(super) slot: usize,
    pub(super) frames_left: u8,
    pub(super) shelves: Shelves,
    pub(super) size: UVec2,
    /// The models as drawn, then with their edges.
    pub(super) models: Handle<Image>,
    pub(super) page: Handle<Image>,
    /// Where each picture is on the page.
    pub(super) cells: Vec<(URect, Handle<Image>)>,
    /// By the edge's reach in pixels.
    pub(super) edges: HashMap<u32, Handle<EdgeMaterial>>,
    pub(super) quad: Handle<Mesh>,
}

impl Batch {
    /// A page large enough for a picture this size, and at least `PAGE_PIXELS` square.
    pub(super) fn new(
        slot: usize,
        first: UVec2,
        images: &mut Assets<Image>,
        meshes: &mut Assets<Mesh>,
    ) -> Self {
        let size = first.max(UVec2::splat(PAGE_PIXELS));
        Self {
            slot,
            frames_left: FRAMES_ALIVE,
            shelves: Shelves::new(size, GAP),
            size,
            models: images.add(page_image(size)),
            page: images.add(page_image(size)),
            cells: Vec::new(),
            edges: HashMap::new(),
            quad: meshes.add(Rectangle::new(1.0, 1.0)),
        }
    }

    /// The edge material for an edge `reach` pixels thick.
    pub(super) fn edge(
        &mut self,
        reach: u32,
        edges: &mut Assets<EdgeMaterial>,
    ) -> Handle<EdgeMaterial> {
        let models = self.models.clone();
        self.edges
            .entry(reach)
            .or_insert_with(|| edges.add(EdgeMaterial::new(reach as f32, models)))
            .clone()
    }
}

/// An image to draw a page on, which pictures can be copied from. It starts with
/// nothing in it rather than zeroes, which would have to be sent to the graphics card.
fn page_image(size: UVec2) -> Image {
    let mut image = Image::new_target_texture(size.x, size.y, TextureFormat::Rgba8UnormSrgb, None);
    image.data = None;
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    image
}
