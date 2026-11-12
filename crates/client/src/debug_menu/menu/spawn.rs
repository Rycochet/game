//! The menu's layers, as markup, and the warning icon.

use crate::widgets::UiId;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::{FORMS, FilterBox, MenuClose, MenuGrid, MenuRoot, PageButton, WarningButton};
use crate::dom::{Pictures, Template, document, top_strip};
use crate::html;
use crate::widgets::{ButtonTooltip, TextBox};

/// The warning button, in the window's top left corner (8 from it, 34
/// across). It has no form round it: a form on a strip has no height, and
/// the pointer is never on what is outside its form.
pub(super) static WARNING: Template = html!(
    "debug_menu.warning",
    r#"
<button id="debug_menu" form="hud" top="8" left="8" width="34" height="34">
  <img src="picture:warning"/>
</button>
"#
);

/// A warning triangle with an exclamation mark, drawn here so there is no file
/// to ship for it.
pub(super) fn warning_icon() -> Image {
    const SIZE: usize = 64;
    let mut bytes = vec![0; SIZE * SIZE * 4];
    let middle = SIZE as f32 / 2.0;
    for y in 4..60 {
        let half = (y - 4) as f32 / 56.0 * 30.0;
        for x in 0..SIZE {
            let across = (x as f32 + 0.5 - middle).abs();
            if across > half {
                continue;
            }
            let edge = across > half - 5.0 || y >= 54;
            let mark = across < 3.5 && ((22..41).contains(&y) || (44..50).contains(&y));
            let colour = if edge || mark {
                [60, 40, 20, 255]
            } else {
                [245, 200, 40, 255]
            };
            bytes[(y * SIZE + x) * 4..][..4].copy_from_slice(&colour);
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

pub(in crate::debug_menu) fn spawn_menu(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<Pictures>,
) {
    pictures.name("warning", images.add(warning_icon()));
    // Above the maps, as the other corner buttons are.
    let warning = (document(&WARNING), GlobalZIndex(15));
    commands.spawn(warning).insert(top_strip());
    commands.spawn(document(&MENU));
}

/// The menu's two layers, hidden: each its title, its filter box with the
/// buttons that move its columns on and back beside it (only there when
/// they don't all fit), and an empty grid. The second is the
/// submenu, above the first and smaller.
pub(super) static MENU: Template = html!(
    "debug_menu.menu",
    r#"
<form id="debug.menu">
  <dialog id="layer" title="Debug Menu" modal closable depth="0" hidden>
    <span nowrap>
      <input id="filter" type="text" prefix="Filter: "/>
      <span id="pages" nowrap hidden>
        <button id="previous" class="small">&lt;</button>
        <button id="next" class="small">&gt;</button>
      </span>
    </span>
    <span id="grid" class="columns" nowrap valign="top"/>
  </dialog>
</form>
<form id="debug.submenu">
  <dialog id="layer" title="Generate POI" modal closable depth="1" hidden>
    <span nowrap>
      <input id="filter" type="text" prefix="Filter: "/>
      <span id="pages" nowrap hidden>
        <button id="previous" class="small">&lt;</button>
        <button id="next" class="small">&gt;</button>
      </span>
    </span>
    <span id="grid" class="columns" nowrap valign="top"/>
  </dialog>
</form>
"#
);

/// Gives the built layers' parts what the menu's systems know them by, found
/// by their ids.
pub(in crate::debug_menu) fn bind_menu(
    mut commands: Commands,
    new: Query<(Entity, &UiId), Added<UiId>>,
) {
    for (entity, id) in &new {
        let (form, name) = id.parts();
        if (form, name) == ("hud", "debug_menu") {
            let tip = ButtonTooltip("Debug Menu".into());
            commands.entity(entity).insert((WarningButton, tip));
        }
        let Some(layer) = FORMS.iter().position(|known| *known == form) else {
            continue;
        };
        let mut bound = commands.entity(entity);
        let page = |forward| PageButton { layer, forward };
        match name {
            "layer" => bound.insert(MenuRoot(layer)),
            "close" => bound.insert(MenuClose),
            "filter" => {
                // Return, Esc and the arrows are the menu's, not the box's.
                let mut text_box = bound.entry::<TextBox>();
                text_box.and_modify(|mut text_box| text_box.takes_keys = false);
                bound.insert(FilterBox(layer))
            }
            "grid" => bound.insert(MenuGrid(layer)),
            "previous" => bound.insert(page(false)),
            "next" => bound.insert(page(true)),
            _ => continue,
        };
    }
}
