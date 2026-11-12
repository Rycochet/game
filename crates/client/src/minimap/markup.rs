//! The maps as markup: the round minimap, and the full map's parchment with
//! its title, picture, close button and the box naming what is pointed at.
//! The pictures are drawn by code into `canvas`es.

use bevy::prelude::*;

use super::gate_runes::{self, RUNES_HEIGHT};
use super::spawn::{marks_layer, new_picture};
use super::{FULL_MAP_LAYER, HoverRunesPicture, MapPictures};
use crate::dom::{Dom, Template, document, top_strip};
use crate::html;
use crate::widgets::{UiId, layer_z, spawn_close_button};

/// The minimap: a place its picture is painted into, in the window's top
/// right corner (`CORNER_GAP`), hidden until there is a map. The pointer is
/// never on it (`inert`): `take_input` and `hold_minimap` do their own search.
pub(super) static MINIMAP: Template = html!(
    "minimap",
    r#"
<canvas id="map" form="minimap" top="8" right="8" width="1" height="1" inert hidden/>
"#
);

/// The full map: a parchment with its title along the top, the picture
/// under it (`FULL_MAP_MARGIN` in, and `TITLE_HEIGHT` down), and near its
/// bottom the box naming what is pointed at, or a World Gate's code in
/// runes. Places inside the parchment are measured from inside its edge
/// (`PANEL_EDGE`, 3), so each is that much less than the constant it comes
/// from. `place_maps` keeps its place and sizes. The pointer is never on it
/// (`inert`), as `take_input` does its own search; the close button is
/// beside it, not in it, so it is still pointed at as a button. Over it is
/// the column of a site's known floors (`floors`): a button for each is a
/// copy of the template, `BUTTON_WIDTH` by `BUTTON` and `BUTTON_GAP` apart
/// (`div class="tight"`), found by `take_input`'s own search too.
pub(super) static FULL_MAP: Template = html!(
    "full_map",
    r#"
<panel id="parchment" form="full_map" top="0" left="0" width="1" height="1" inert hidden>
  <h1 id="title" form="full_map" top="15" left="0" right="0" align="center"/>
  <canvas id="picture" form="full_map" top="51" left="17" width="1" height="1"/>
  <div id="hover" form="full_map" class="joined" bottom="29" left="0" right="0" align="center" hidden>
    <span id="name" form="full_map" class="tip" hidden><p id="words" form="full_map"/></span>
    <span id="runes" form="full_map" class="tip" hidden>
      <canvas id="code" form="full_map" width="1" height="22"/>
    </span>
  </div>
</panel>
<div id="close" form="full_map" top="0" left="0" hidden/>
<div id="floors" form="full_map" class="tight" top="0" left="0" inert hidden/>
<template id="floor" form="full_map">
  <button id="go" form="full_map" width="36" height="26">0</button>
</template>
"#
);

/// The maps' documents, and the pictures they are drawn in.
pub(super) fn spawn_maps(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut dom: Dom) {
    let mut picture = || images.add(new_picture());
    commands.insert_resource(MapPictures {
        minimap: picture(),
        full: picture(),
        minimap_marks: picture(),
        full_marks: picture(),
    });
    // Above the maps of the world, as the other corner pieces are.
    let minimap = (document(&MINIMAP), GlobalZIndex(10));
    commands.spawn(minimap).insert(top_strip());
    // Over its darkening.
    let over = GlobalZIndex(layer_z(FULL_MAP_LAYER) + 1);
    commands
        .spawn((document(&FULL_MAP), over))
        .insert(top_strip());
    let width = RUNES_HEIGHT * gate_runes::aspect();
    dom.set("full_map/code", "width", &width.to_string());
}

/// Gives each place, once built, what is drawn in it: the maps' pictures
/// with their marks over them, the runes' picture, and the close button.
pub(super) fn bind_maps(
    new: Query<(Entity, &UiId), Added<UiId>>,
    pictures: Res<MapPictures>,
    mut commands: Commands,
) {
    for (entity, id) in &new {
        let mut place = commands.entity(entity);
        match id.parts() {
            ("minimap", "map") => {
                place.insert(ImageNode::new(pictures.minimap.clone()));
                place.with_child(marks_layer(&pictures.minimap_marks));
            }
            ("full_map", "picture") => {
                place.insert(ImageNode::new(pictures.full.clone()));
                place.with_child(marks_layer(&pictures.full_marks));
            }
            ("full_map", "code") => {
                place.insert((HoverRunesPicture, ImageNode::default()));
            }
            ("full_map", "close") => {
                place.with_children(|place| {
                    spawn_close_button(place, Node::default());
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
