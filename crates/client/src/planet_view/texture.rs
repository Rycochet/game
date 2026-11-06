//! The planet's picture: the whole surface, east-west across and pole to pole
//! down. Filled coarsely first, in blocks, then finer and finer, all within a
//! few milliseconds a frame: the whole globe shows at once and sharpens as it
//! turns, rather than the game stopping or the picture filling row by row.
//! Then the relief is worked out from the heights found, the same way.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::time::Duration;

use super::colour::{Colouring, colour_at};
use super::gates::stamp_gates;
use super::relief::relief_row;
use super::showing::{Globe, PlanetView};
use crate::surface_style::SurfaceStyle;

/// The widest the picture gets; smaller worlds get a pixel a grid point.
const MOST_COLUMNS: i64 = 1024;
/// How long a frame may spend filling it.
const FRAME_BUDGET: Duration = Duration::from_millis(6);
/// Blocks a pass fills with one colour, in pixels across: each pass works out
/// only the pixels the ones before haven't.
const PASSES: [u32; 4] = [8, 4, 2, 1];
/// Before anything is filled.
const UNFILLED: [u8; 4] = [24, 24, 28, 255];
/// Grid points to a pixel from which only the broad noise layers are used:
/// finer ones would only speckle the picture with hills smaller than a pixel.
const BROAD_FROM: i64 = 4;

/// How far filling has got: the pass, and the row within it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Fill {
    pass: usize,
    row: u32,
}

/// The pass after the colours': the relief, a row at a time.
const RELIEF_PASS: usize = PASSES.len();

impl Fill {
    pub(super) fn done(self) -> bool {
        self.pass > RELIEF_PASS
    }
}

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

/// The pixels a pass works out in one of its rows (every `block`-th): those
/// an earlier, coarser pass hasn't.
pub(super) fn pass_columns(pass: usize, row: u32, width: u32) -> impl Iterator<Item = u32> {
    let block = PASSES[pass];
    let coarser = pass.checked_sub(1).map(|earlier| PASSES[earlier]);
    (0..width).step_by(block as usize).filter(move |&column| {
        coarser.is_none_or(|c| !column.is_multiple_of(c) || !row.is_multiple_of(c))
    })
}

/// Fills the next rows, as many as fit in the frame's budget.
pub(super) fn fill_texture(
    mut view: ResMut<PlanetView>,
    style: Res<SurfaceStyle>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(showing) = view.showing.as_mut() else {
        return;
    };
    let globe = &mut showing.globe;
    if globe.fill.done() {
        return;
    }
    if globe.fill.pass == RELIEF_PASS {
        fill_relief(globe, &mut images);
        return;
    }
    let Some(mut image) = images.get_mut(&globe.texture) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let size = globe.size;
    let width = globe.planet.surface.width();
    let colouring = Colouring {
        layer: globe.layer,
        water: globe.planet.params.sea_level,
        broad: width >= size.x as i64 * BROAD_FROM,
    };
    let started = Instant::now();
    while globe.fill.pass < RELIEF_PASS && started.elapsed() < FRAME_BUDGET {
        let Fill { pass, row } = globe.fill;
        let block = PASSES[pass];
        for column in pass_columns(pass, row, size.x) {
            let at = ground_at(column, row, size, width);
            let (colour, metres) = colour_at(&globe.planet, &style, colouring, at);
            for down in row..(row + block).min(size.y) {
                for across in column..(column + block).min(size.x) {
                    let pixel = (down * size.x + across) as usize;
                    data[pixel * 4..pixel * 4 + 4].copy_from_slice(&colour);
                    globe.heights[pixel] = metres;
                }
            }
        }
        globe.fill.row += block;
        if globe.fill.row >= size.y {
            globe.fill = Fill {
                pass: pass + 1,
                row: 0,
            };
            if globe.fill.pass == RELIEF_PASS {
                stamp_gates(&globe.planet.gates, size, width, data);
            }
        }
    }
}

/// The relief's next rows, as many as fit in the frame's budget, once the
/// heights are all known.
fn fill_relief(globe: &mut Globe, images: &mut Assets<Image>) {
    let Some(mut image) = images.get_mut(&globe.relief) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let circumference = globe.planet.surface.width() as f32 * 1000.0;
    let started = Instant::now();
    while globe.fill.row < globe.size.y && started.elapsed() < FRAME_BUDGET {
        relief_row(
            &globe.heights,
            globe.size,
            globe.fill.row,
            circumference,
            data,
        );
        globe.fill.row += 1;
    }
    if globe.fill.row >= globe.size.y {
        globe.fill = Fill {
            pass: RELIEF_PASS + 1,
            row: 0,
        };
    }
}
