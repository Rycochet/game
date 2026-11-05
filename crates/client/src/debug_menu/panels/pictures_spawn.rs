//! Item Pictures' panel: its page, headings and rows of pictures.

use bevy::prelude::*;
use shared::blade::Family;
use shared::body::{Frame, Parts};
use shared::creature::CreatureKind;
use shared::item::Quality;
use shared::item_form::Cut;
use shared::material::Material;

use super::page::PicturePage;
use super::{
    FONT, GemPicture, LARGE_PICTURE, LargePicture, PAGES_WIDTH, PICTURE, PICTURE_LABEL,
    PICTURE_ROW_LABEL, PICTURES_HEIGHT, PicturePages, PictureQualities, PicturesGrid, RING_COLUMNS,
    Shown, gem_to_show,
};
use crate::widgets::{
    BUTTON_PADDING, Dropdown, INK, Pressable, spawn_dropdown, spawn_scroll_area, spawn_toggle_row,
};

/// The page of pictures last chosen, filled in by `fill_pictures`, and whichever
/// picture was last pressed, large.
pub(super) fn spawn_item_pictures(content: &mut ChildSpawnerCommands) {
    let qualities: Vec<String> = Quality::ALL.iter().map(|q| format!("{q:?}")).collect();
    let qualities: Vec<&str> = qualities.iter().map(String::as_str).collect();
    let common = Quality::ALL
        .iter()
        .position(|&quality| quality == Quality::default())
        .unwrap_or(0);
    content
        .spawn(Node {
            column_gap: px(8),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|choices| {
            let pages = Dropdown {
                options: PicturePage::ALL.map(|page| page.name().to_owned()).to_vec(),
                selected: 0,
                searchable: false,
                ..default()
            };
            let node = Node {
                padding: BUTTON_PADDING,
                min_width: px(PAGES_WIDTH),
                ..default()
            };
            spawn_dropdown(choices, pages, FONT, node).insert(PicturePages);
            let row = spawn_toggle_row(choices, &qualities, common, FONT);
            choices.commands().entity(row).insert(PictureQualities);
        });
    content
        .spawn(Node {
            column_gap: px(8),
            ..default()
        })
        .with_children(|sides| {
            sides
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|column| {
                    let area = spawn_scroll_area(column, PICTURES_HEIGHT);
                    column.commands().entity(area).with_children(|area| {
                        area.spawn((
                            PicturesGrid::default(),
                            Node {
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                        ));
                    });
                });
            sides.spawn((
                LargePicture,
                Node {
                    width: px(LARGE_PICTURE),
                    height: px(LARGE_PICTURE),
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode::default(),
            ));
        });
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

/// Every kind that stands on two legs, bare, at its usual height beside a person.
pub(super) fn spawn_body_row(grid: &mut ChildSpawnerCommands) {
    grid.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|cells| {
        cells.spawn(picture_label("Bodies", PICTURE_ROW_LABEL));
        for kind in CreatureKind::ALL {
            if let Some(frame) = Frame::of(kind) {
                let height = Frame::usual_height(kind);
                spawn_picture(cells, Shown::Body(frame, Parts::of(kind), height));
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
