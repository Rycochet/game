//! Item Pictures' panel: its page, headings and rows of pictures.

use bevy::prelude::*;
use shared::blade::Family;
use shared::body::{BEASTS, Layout};
use shared::creature::CreatureKind;
use shared::item_form::Cut;
use shared::material::Material;

use super::{
    GemPicture, LargePicture, PICTURE, PICTURE_LABEL, PICTURE_ROW_LABEL, PicturePages,
    PictureQualities, PicturesGrid, RING_COLUMNS, Shown, gem_to_show,
};
use crate::dom::Template;
use crate::html;
use crate::widgets::{INK, Pressable, UiId};

/// The panel's shell, as markup: the page's dropdown (its choices given by
/// `fill_pictures`), the qualities, the scrolling grid and the large picture.
/// The grid's rows of pictures and the large picture are drawn by code, in
/// the two canvases.
pub(super) static ITEM_PICTURES: Template = html!(
    "debug_menu.item_pictures",
    r#"
<form id="debug.item_pictures">
  <panel id="panel" moveable>
    <span nowrap valign="middle">
      <select id="page" width="160"><option>Gems</option></select>
      <toggle-row id="quality">
        <option>Poor</option>
        <option selected>Common</option>
        <option>Uncommon</option>
        <option>Rare</option>
        <option>Epic</option>
        <option>Legendary</option>
        <option>Artifact</option>
      </toggle-row>
    </span>
    <span nowrap valign="top">
      <scroll id="scroll" max-height="70%">
        <canvas id="grid"/>
      </scroll>
      <canvas id="large" width="256" height="256"/>
    </span>
  </panel>
</form>
"#
);

/// What a built part of the panel is known by, from its id.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId) {
    match id.parts().1 {
        "page" => bound.insert(PicturePages),
        "quality" => bound.insert(PictureQualities),
        "grid" => bound.insert(PicturesGrid::default()),
        // Whichever picture was last pressed, large, in the middle of its room.
        "large" => bound.with_child((LargePicture, Node::default(), ImageNode::default())),
        _ => bound,
    };
}

pub(super) fn picture_label(words: &str, width: f32) -> impl Bundle {
    (
        Text::new(words),
        PICTURE_LABEL.font(),
        TextColor(INK),
        Node {
            width: px(width),
            ..default()
        },
    )
}

pub(super) fn spawn_picture_heading(grid: &mut ChildSpawnerCommands) {
    grid.spawn(Node::default()).with_children(|row| {
        row.spawn(picture_label("", PICTURE_ROW_LABEL));
        for cut in Cut::ALL {
            row.spawn(picture_label(cut.name(), PICTURE));
        }
    });
}

pub(super) fn spawn_picture_row(grid: &mut ChildSpawnerCommands, material: Material) {
    grid.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|row| {
        row.spawn(picture_label(material.name(), PICTURE_ROW_LABEL));
        for cut in Cut::ALL {
            spawn_picture(row, Shown::Gem(gem_to_show(material, cut)));
        }
    });
}

pub(super) fn spawn_ring_row(grid: &mut ChildSpawnerCommands, row: u128) {
    grid.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|cells| {
        cells.spawn(picture_label(
            if row == 0 { "Rings" } else { "" },
            PICTURE_ROW_LABEL,
        ));
        for cell in 0..RING_COLUMNS {
            spawn_picture(cells, Shown::Ring(row * RING_COLUMNS + cell + 1));
        }
    });
}

/// Every kind whose body is drawn, bare: those on two legs at their usual height beside
/// a person, animals filling their pictures; then animals not yet kinds.
pub(super) fn spawn_body_row(grid: &mut ChildSpawnerCommands) {
    grid.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|cells| {
        cells.spawn(picture_label("Bodies", PICTURE_ROW_LABEL));
        for kind in CreatureKind::ALL {
            if Layout::of(kind).is_some() {
                spawn_picture(cells, Shown::Body(kind));
            }
        }
        for (index, beast) in BEASTS.iter().enumerate() {
            if !CreatureKind::ALL
                .iter()
                .any(|kind| kind.def().name == beast.name)
            {
                spawn_picture(cells, Shown::Beast(index));
            }
        }
    });
}

pub(super) fn spawn_blade_row(grid: &mut ChildSpawnerCommands, family: Family) {
    grid.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|cells| {
        cells.spawn(picture_label(&format!("{family:?}"), PICTURE_ROW_LABEL));
        for cell in 0..RING_COLUMNS {
            let seed = (family as u128 + 1) * 1000 + cell;
            spawn_picture(cells, Shown::Blade(family, seed));
        }
    });
}

pub(super) fn spawn_picture(row: &mut ChildSpawnerCommands, shown: Shown) {
    row.spawn((
        GemPicture(shown),
        Pressable,
        Node {
            width: px(PICTURE),
            height: px(PICTURE),
            flex_shrink: 0.0,
            ..default()
        },
        ImageNode::default(),
    ));
}
