//! Sorting: a small button above each carried grid lays it out again in one go
//! (`Inventory::sort_grid`). The client tries first, and asks the server only if
//! that worked and changed something, so a sort that can't be done does nothing.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::item::Holder;
use shared::protocol::ClientMessage;

use super::server::Stacks;
use super::{Carried, Holding};
use crate::connection::ToServer;
use crate::dom::Pictures;
use crate::widgets::ButtonPressed;

/// A sort button (in its grid's markup), and the grid it sorts.
#[derive(Component, Clone, Copy)]
pub(super) struct SortButton(pub Holder);

/// Names the sort button's picture, for the markup (`picture:sort`).
pub(super) fn make_icon(mut images: ResMut<Assets<Image>>, mut pictures: ResMut<Pictures>) {
    pictures.name("sort", images.add(sort_icon()));
}

/// Sorts a grid when its button is pressed, unless something is held.
pub(super) fn press_sort(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&SortButton>,
    holding: Res<Holding>,
    stacks: Res<Stacks>,
    mut carried: ResMut<Carried>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        let Ok(&SortButton(holder)) = buttons.get(press.0) else {
            continue;
        };
        if holding.held.is_none() && carried.0.sort_grid(holder, stacks.0) {
            to_server.write(ToServer(ClientMessage::SortGrid { holder }));
        }
    }
}

/// Three bars, longest at the top, drawn here so there is no file to ship for it.
fn sort_icon() -> Image {
    const SIZE: usize = 32;
    const INK_BYTES: [u8; 4] = [60, 40, 20, 255];
    let mut bytes = vec![0; SIZE * SIZE * 4];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (x, y) = (x as i32, y as i32);
            let bar = |top: i32, right: i32| (top..top + 5).contains(&y) && (4..right).contains(&x);
            if bar(5, 28) || bar(14, 21) || bar(23, 14) {
                bytes[(y as usize * SIZE + x as usize) * 4..][..4].copy_from_slice(&INK_BYTES);
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
