//! The picture of an item in writing nobody knows: a grey tile with a "?". Made
//! once at start.

use bevy::asset::{RenderAssetUsages, uuid_handle};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// A grey tile with a "?".
pub(super) const UNKNOWN: Handle<Image> = uuid_handle!("7a2d9c13-8e6f-4b05-a3d1-9f4e2b7c6a08");

/// A "?" five wide and seven tall, a row to a line.
const QUESTION: [&str; 7] = [
    ".###.", "#...#", "....#", "...#.", "..#..", ".....", "..#..",
];
/// How many pixels each of its dots is across.
const DOT: usize = 3;
/// Room round it, in pixels.
const MARGIN: usize = 3;

pub(super) fn make_tiles(mut images: ResMut<Assets<Image>>) {
    let _ = images.insert(&UNKNOWN, question_tile());
}

/// Whether the dot at this pixel of the tile is part of the "?".
fn on_question(x: usize, y: usize) -> bool {
    let dot = |at: usize, count: usize| {
        at.checked_sub(MARGIN)
            .map(|at| at / DOT)
            .filter(|&at| at < count)
    };
    dot(x, 5)
        .zip(dot(y, 7))
        .is_some_and(|(column, row)| QUESTION[row].as_bytes()[column] == b'#')
}

fn question_tile() -> Image {
    const GREY: [u8; 4] = [140, 134, 124, 255];
    const LIGHT: [u8; 4] = [235, 230, 220, 255];
    let (wide, tall) = (5 * DOT + 2 * MARGIN, 7 * DOT + 2 * MARGIN);
    let mut bytes = Vec::with_capacity(wide * tall * 4);
    for y in 0..tall {
        for x in 0..wide {
            bytes.extend_from_slice(if on_question(x, y) { &LIGHT } else { &GREY });
        }
    }
    image(wide as u32, tall as u32, bytes)
}

fn image(width: u32, height: u32, bytes: Vec<u8>) -> Image {
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
