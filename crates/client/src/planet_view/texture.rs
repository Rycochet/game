//! The planet's picture: the whole surface, east-west across and pole to pole
//! down, coloured as the world map colours it (without the hills too small to see). Filled a few rows a frame, so
//! the globe fills in as it turns rather than the game stopping.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::surface::SurfaceSample;
use std::time::Duration;

use super::open::PlanetView;
use crate::common::CurrentWorld;
use crate::surface_style::{SurfaceKind, SurfaceStyle};

/// The widest the picture gets; smaller worlds get a pixel a grid point.
const MOST_COLUMNS: i64 = 1024;
/// How long a frame may spend filling it.
const FRAME_BUDGET: Duration = Duration::from_millis(6);
/// Rows not filled yet.
const UNFILLED: [u8; 4] = [24, 24, 28, 255];
/// How much darker trees make open land, when the ground is all woods.
const WOODS_DARKER: f32 = 0.3;
/// Grid points to a pixel from which only the broad noise layers are used:
/// finer ones would only speckle the picture with hills smaller than a pixel.
const BROAD_FROM: i64 = 4;

/// The picture's size for a world `width` grid points round.
pub(super) fn texture_size(width: i64) -> UVec2 {
    let columns = width.clamp(2, MOST_COLUMNS) as u32;
    UVec2::new(columns, (columns / 2).max(1))
}

/// The grid point a pixel's middle shows. Rows run from the north pole
/// (`y = -width / 4`) to the south pole (`y = width / 4`), as far as the
/// climate's poles: the world goes on past them, but the globe can't show it.
pub(super) fn ground_at(column: u32, row: u32, size: UVec2, width: i64) -> (i64, i64) {
    let across = (column as f64 + 0.5) / size.x as f64;
    let down = (row as f64 + 0.5) / size.y as f64;
    let x = (across * width as f64).floor() as i64;
    let y = ((down - 0.5) * width as f64 / 2.0).round() as i64;
    (x, y)
}

/// An empty picture of this size, which wraps east-west.
pub(super) fn new_texture(size: UVec2) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &UNFILLED,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

/// A grid point's colour on the globe: its colour on level ground, darker
/// under trees.
fn colour_at(
    world: &CurrentWorld,
    style: &SurfaceStyle,
    at: (i64, i64),
    lines: (f64, f64),
    broad: bool,
) -> [u8; 4] {
    let (x, y) = at;
    let (water, snow) = lines;
    let sample = if broad {
        SurfaceSample {
            height: world.surface.broad_height(x, y),
            jitter: 0.0,
        }
    } else {
        world.surface.sample(x, y)
    };
    let climate = world.climate.sample(x, y, sample.height);
    let mut colour = style.ground_colour(sample, climate, water, snow);
    if style.kind(sample, climate, water, snow) == SurfaceKind::Land {
        let trees = world.vegetation.cover(x, y, sample.height, climate).trees;
        let shade = 1.0 - WOODS_DARKER * trees.clamp(0.0, 1.0);
        for channel in &mut colour[..3] {
            *channel = (*channel as f32 * shade).round() as u8;
        }
    }
    colour
}

/// Fills the next rows, as many as fit in the frame's budget.
pub(super) fn fill_texture(
    mut view: ResMut<PlanetView>,
    world: Option<Res<CurrentWorld>>,
    style: Res<SurfaceStyle>,
    mut images: ResMut<Assets<Image>>,
) {
    let (Some(showing), Some(world)) = (view.showing.as_mut(), world) else {
        return;
    };
    let size = showing.size;
    if showing.rows_done >= size.y {
        return;
    }
    let Some(mut image) = images.get_mut(&showing.texture) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let width = world.surface.width();
    let broad = width >= size.x as i64 * BROAD_FROM;
    let lines = (showing.water, showing.snow_line);
    let started = Instant::now();
    while showing.rows_done < size.y && started.elapsed() < FRAME_BUDGET {
        let row = showing.rows_done;
        for column in 0..size.x {
            let at = ground_at(column, row, size, width);
            let colour = colour_at(&world, &style, at, lines, broad);
            let start = ((row * size.x + column) * 4) as usize;
            data[start..start + 4].copy_from_slice(&colour);
        }
        showing.rows_done += 1;
    }
}
