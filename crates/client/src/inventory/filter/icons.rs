//! The filter's pictures, one for All and one for each category, drawn here so
//! there are no files to ship for them.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::item::ItemCategory;

const SIZE: usize = 32;
const INK_BYTES: [u8; 4] = [60, 40, 20, 255];

/// The picture for All (`None`), or for a category.
pub(super) fn filter_icon(category: Option<ItemCategory>) -> Image {
    use ItemCategory::*;
    match category {
        None => draw(funnel),
        Some(Weapons) => draw(sword),
        Some(Armour) => draw(shield),
        Some(Clothing) => draw(shirt),
        Some(Jewellery) => draw(ring),
        Some(Containers) => draw(bag),
        Some(Food) => draw(apple),
        Some(Valuables) => draw(coin),
        Some(Materials) => draw(stones),
        Some(Other) => draw(dots),
    }
}

/// An image with ink wherever `paint` says, given each pixel's middle.
fn draw(paint: fn(f32, f32) -> bool) -> Image {
    let mut bytes = vec![0; SIZE * SIZE * 4];
    for y in 0..SIZE {
        for x in 0..SIZE {
            if paint(x as f32 + 0.5, y as f32 + 0.5) {
                bytes[(y * SIZE + x) * 4..][..4].copy_from_slice(&INK_BYTES);
            }
        }
    }
    Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Within a box, from its left and top to its right and bottom.
fn inside(x: f32, y: f32, left: f32, top: f32, right: f32, bottom: f32) -> bool {
    (left..right).contains(&x) && (top..bottom).contains(&y)
}

/// Within an ellipse with this middle and these half widths.
fn oval(x: f32, y: f32, middle: (f32, f32), across: f32, down: f32) -> bool {
    let (dx, dy) = ((x - middle.0) / across, (y - middle.1) / down);
    dx * dx + dy * dy <= 1.0
}

fn funnel(x: f32, y: f32) -> bool {
    let across = (x - 16.0).abs();
    let cone = (4.0..18.0).contains(&y) && across < 13.0 - (y - 4.0) * 0.7;
    cone || inside(x, y, 13.0, 18.0, 19.0, 28.0)
}

fn sword(x: f32, y: f32) -> bool {
    let across = (x - 16.0).abs();
    let blade = (5.0..21.0).contains(&y) && across < 2.0;
    let tip = (1.0..5.0).contains(&y) && across < (y - 1.0) / 2.0;
    let guard = inside(x, y, 8.0, 21.0, 24.0, 24.0);
    let grip = (24.0..28.0).contains(&y) && across < 1.5;
    blade || tip || guard || grip || oval(x, y, (16.0, 29.5), 2.5, 2.5)
}

fn shield(x: f32, y: f32) -> bool {
    let across = (x - 16.0).abs();
    let top = (4.0..16.0).contains(&y) && across < 11.0;
    let point = (16.0..29.0).contains(&y) && across < 11.0 * (29.0 - y) / 13.0;
    top || point
}

fn shirt(x: f32, y: f32) -> bool {
    let body = inside(x, y, 9.0, 7.0, 23.0, 28.0);
    let sleeves = inside(x, y, 3.0, 7.0, 29.0, 15.0) && !inside(x, y, 3.0, 12.0, 7.0, 15.0)
        || inside(x, y, 3.0, 7.0, 7.0, 13.0)
        || inside(x, y, 25.0, 7.0, 29.0, 13.0);
    let neck = oval(x, y, (16.0, 6.0), 4.0, 4.0);
    (body || sleeves) && !neck
}

fn ring(x: f32, y: f32) -> bool {
    let band = oval(x, y, (16.0, 20.0), 10.0, 10.0) && !oval(x, y, (16.0, 20.0), 7.0, 7.0);
    let stone = (x - 16.0).abs() + (y - 7.0).abs() < 5.0;
    band || stone
}

fn bag(x: f32, y: f32) -> bool {
    let body = oval(x, y, (16.0, 20.0), 11.0, 9.0);
    let neck = inside(x, y, 13.0, 8.0, 19.0, 12.0);
    let tie = inside(x, y, 10.0, 5.0, 22.0, 8.0);
    body || neck || tie
}

fn apple(x: f32, y: f32) -> bool {
    let fruit = oval(x, y, (12.0, 19.0), 8.0, 9.0) || oval(x, y, (20.0, 19.0), 8.0, 9.0);
    let dip = oval(x, y, (16.0, 9.0), 3.0, 2.5);
    let stem = inside(x, y, 15.0, 3.0, 17.0, 10.0);
    let leaf = oval(x, y, (21.0, 6.0), 4.0, 2.0);
    (fruit && !dip) || stem || leaf
}

fn coin(x: f32, y: f32) -> bool {
    let face = oval(x, y, (16.0, 16.0), 13.0, 13.0);
    let groove = oval(x, y, (16.0, 16.0), 10.0, 10.0) && !oval(x, y, (16.0, 16.0), 8.5, 8.5);
    face && !groove
}

fn stones(x: f32, y: f32) -> bool {
    oval(x, y, (13.0, 21.0), 11.0, 7.0) || oval(x, y, (23.0, 12.0), 6.0, 5.0)
}

fn dots(x: f32, y: f32) -> bool {
    [7.0, 16.0, 25.0]
        .iter()
        .any(|&middle| oval(x, y, (middle, 16.0), 3.0, 3.0))
}
