//! Making the menu's layers, and the warning icon.

use crate::widgets::UiId;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::super::actions::DebugAction;
use super::{
    ACTIONS, CLOSE_MARGIN, COLUMN_GAP, CORNER_GAP, FILTER_HEIGHT, FONT, FilterBox, GAP, MARGIN,
    MenuClose, MenuGrid, MenuRoot, PAGE_BUTTON, PageButton, SUBMENU, TITLE_ROW, WARNING_BUTTON,
    WarningButton,
};
use crate::widgets::{
    ButtonState, ButtonTooltip, Pressable, TextBox, panel_title, spawn_button, spawn_close_button,
    spawn_fullscreen_layer, spawn_icon_button, spawn_text_box,
};

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

pub(in crate::debug_menu) fn spawn_menu(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let icon = images.add(warning_icon());
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(CORNER_GAP),
                left: px(CORNER_GAP),
                ..default()
            },
            GlobalZIndex(15),
        ))
        .with_children(|holder| {
            let node = Node {
                width: px(WARNING_BUTTON),
                height: px(WARNING_BUTTON),
                ..default()
            };
            spawn_icon_button(holder, ButtonState::Normal, icon, node).insert((
                WarningButton,
                Pressable,
                UiId::new("hud", "debug_menu"),
                ButtonTooltip("Debug Menu".into()),
            ));
        });
    spawn_layer(&mut commands, ACTIONS, "Debug Menu");
    spawn_layer(&mut commands, SUBMENU, DebugAction::GeneratePoi.name());
}

/// One layer of the menu, hidden: its title, filter box and an empty grid.
pub(super) fn spawn_layer(commands: &mut Commands, layer: usize, title: &str) {
    let (root, list) = spawn_fullscreen_layer(
        commands,
        layer as i32,
        Node {
            padding: UiRect::all(px(MARGIN)),
            flex_direction: FlexDirection::Column,
            row_gap: px(GAP * 3.0),
            ..default()
        },
    );
    commands.entity(root).insert(MenuRoot(layer));
    commands.entity(list).with_children(|root| {
        root.spawn(Node {
            height: px(TITLE_ROW),
            min_height: px(TITLE_ROW),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|bar| {
            bar.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(0),
                column_gap: px(GAP),
                ..default()
            })
            .with_children(|pages| {
                for (label, forward) in [("<", false), (">", true)] {
                    let node = Node {
                        width: px(PAGE_BUTTON),
                        height: px(PAGE_BUTTON),
                        ..default()
                    };
                    spawn_button(pages, ButtonState::Normal, label, FONT, node).insert((
                        PageButton { layer, forward },
                        Pressable,
                        UiId::new("debug.menu", if forward { "next" } else { "previous" })
                            .at(layer),
                        Visibility::Hidden,
                    ));
                }
            });
            bar.spawn(panel_title(title));
            bar.spawn((
                MenuClose,
                Pressable,
                UiId::new("debug.menu", "close").at(layer),
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0),
                    padding: UiRect::all(px(CLOSE_MARGIN)),
                    ..default()
                },
            ))
            .with_children(|holder| {
                spawn_close_button(holder, default());
            });
        });
        let text_box = TextBox {
            prefix: "Filter: ",
            ..default()
        };
        let node = Node {
            height: px(FILTER_HEIGHT),
            min_height: px(FILTER_HEIGHT),
            ..default()
        };
        spawn_text_box(root, text_box, FONT, node).insert((
            FilterBox(layer),
            UiId::new("debug.menu", "filter").at(layer),
        ));
        root.spawn((
            MenuGrid(layer),
            Node {
                column_gap: px(COLUMN_GAP),
                ..default()
            },
        ));
    });
}
