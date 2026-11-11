//! Making the gear button and the menu's layers: main panel, quit requester, settings.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::{Action, CORNER_GAP, GEAR_BUTTON, GearButton};
use crate::dom::{Dom, Pictures};
use crate::widgets::{ButtonState, ButtonTooltip, Pressable, spawn_icon_button};

/// A gear, drawn here so there is no file to ship for it.
pub(super) fn gear_icon() -> Image {
    const SIZE: usize = 64;
    const TEETH: f32 = 8.0;
    const OUTSIDE: f32 = 29.0;
    const BETWEEN_TEETH: f32 = 22.0;
    const HOLE: f32 = 9.0;
    let mut bytes = vec![0; SIZE * SIZE * 4];
    let middle = SIZE as f32 / 2.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (across, down) = (x as f32 + 0.5 - middle, y as f32 + 0.5 - middle);
            let distance = across.hypot(down);
            let turn = down.atan2(across) / std::f32::consts::TAU * TEETH;
            let reach = if turn.rem_euclid(1.0) < 0.5 {
                OUTSIDE
            } else {
                BETWEEN_TEETH
            };
            if (HOLE..=reach).contains(&distance) {
                bytes[(y * SIZE + x) * 4..][..4].copy_from_slice(&[60, 40, 20, 255]);
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

pub(super) fn spawn_menu(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut dom: Dom,
    mut pictures: ResMut<Pictures>,
) {
    let icon = images.add(gear_icon());
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(CORNER_GAP),
                right: px(CORNER_GAP),
                ..default()
            },
            // Above the maps, as the warning button is.
            GlobalZIndex(15),
        ))
        .with_children(|holder| {
            let node = Node {
                width: px(GEAR_BUTTON),
                height: px(GEAR_BUTTON),
                ..default()
            };
            spawn_icon_button(holder, ButtonState::Normal, icon.clone(), node).insert((
                GearButton,
                Action::Open,
                Action::Open.ui_id("hud"),
                Pressable,
                ButtonTooltip("Menu".into()),
            ));
        });
    // The gear again, on the menu's Server Options button.
    pictures.name("gear", icon.clone());
    super::panel::spawn_main_panel(&mut commands);
    super::requesters::spawn_requesters(&mut commands, &mut dom, super::slow_drawing::WORDS);
    super::settings::spawn_settings(&mut commands, &mut dom);
    super::credits::spawn_credits(&mut commands, &mut dom);
    let icons = super::icons::ConnectIcons::new(&mut images);
    super::connect::spawn_connect(&mut commands, &mut dom, &mut pictures, &icons);
    super::connect::spawn_popups(&mut commands);
    #[cfg(not(target_arch = "wasm32"))]
    super::server_options::spawn_server_options(&mut commands);
    commands.insert_resource(icons);
}
