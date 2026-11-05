//! Making the maps' parts and pictures.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::gate_runes::{self, RUNES_HEIGHT};
use super::{
    CORNER_GAP, FULL_MAP_LAYER, FULL_MAP_MARGIN, FullMapDarkening, FullMapPanel, FullMapPicture,
    FullMapTitle, HOVER_LIFT, HoverBox, HoverRunes, HoverRunesPicture, HoverText, MapPictures,
    Minimap, TITLE_HEIGHT, WorldHoverBox, WorldHoverText,
};
use crate::widgets::{
    darkening, layer_z, leaves_presses, panel, panel_frame, panel_title, spawn_close_button,
    spawn_tooltip, tooltip_box,
};

pub(super) fn new_picture() -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// A see-through picture over the whole of the map it is put in.
fn marks_layer(picture: &Handle<Image>) -> impl Bundle {
    (
        ImageNode::new(picture.clone()),
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: percent(100),
            ..default()
        },
    )
}

pub(super) fn spawn_maps(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let minimap = images.add(new_picture());
    let full = images.add(new_picture());
    let minimap_marks = images.add(new_picture());
    let full_marks = images.add(new_picture());
    commands
        .spawn((
            Minimap,
            ImageNode::new(minimap.clone()),
            Node {
                position_type: PositionType::Absolute,
                top: px(CORNER_GAP),
                right: px(CORNER_GAP),
                ..default()
            },
            GlobalZIndex(10),
            Visibility::Hidden,
        ))
        .with_child(marks_layer(&minimap_marks));
    let behind = commands
        .spawn((
            darkening(FULL_MAP_LAYER),
            FullMapDarkening,
            Visibility::Hidden,
        ))
        .id();
    commands.entity(behind).insert(leaves_presses(behind));
    commands
        .spawn((
            panel(),
            FullMapPanel,
            GlobalZIndex(layer_z(FULL_MAP_LAYER) + 1),
            Visibility::Hidden,
        ))
        .with_children(|panel| {
            panel.spawn(panel_frame());
            panel.spawn((
                FullMapTitle,
                panel_title(""),
                TextLayout::justify(Justify::Center),
                Node {
                    position_type: PositionType::Absolute,
                    top: px(FULL_MAP_MARGIN),
                    left: px(0),
                    right: px(0),
                    ..default()
                },
            ));
            panel
                .spawn((
                    FullMapPicture,
                    ImageNode::new(full.clone()),
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(FULL_MAP_MARGIN + TITLE_HEIGHT),
                        left: px(FULL_MAP_MARGIN),
                        ..default()
                    },
                ))
                .with_child(marks_layer(&full_marks));
            spawn_close_button(
                panel,
                Node {
                    position_type: PositionType::Absolute,
                    top: px(FULL_MAP_MARGIN),
                    right: px(FULL_MAP_MARGIN),
                    ..default()
                },
            );
            panel
                .spawn((
                    HoverBox,
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: px(FULL_MAP_MARGIN + HOVER_LIFT),
                        left: px(0),
                        right: px(0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Visibility::Hidden,
                ))
                .with_children(|holder| {
                    spawn_tooltip(holder, HoverText, "");
                    let (height, width) = (RUNES_HEIGHT, RUNES_HEIGHT * gate_runes::aspect());
                    holder
                        .spawn((HoverRunes, tooltip_box(Node::default()), Visibility::Hidden))
                        .with_child((
                            HoverRunesPicture,
                            ImageNode::default(),
                            Node {
                                width: px(width),
                                height: px(height),
                                ..default()
                            },
                        ));
                });
        });
    commands
        .spawn((
            WorldHoverBox,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(FULL_MAP_MARGIN + HOVER_LIFT),
                left: px(0),
                right: px(0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(10),
            Visibility::Hidden,
        ))
        .with_children(|holder| spawn_tooltip(holder, WorldHoverText, ""));
    commands.insert_resource(MapPictures {
        minimap,
        full,
        minimap_marks,
        full_marks,
    });
}
