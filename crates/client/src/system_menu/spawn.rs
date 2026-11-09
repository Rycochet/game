//! Making the gear button and the menu's layers: main panel, quit requester, settings.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::{
    ABOVE_MENU_LAYER, Action, BUTTON_ROW, CLOSE_MARGIN, CORNER_GAP, FONT, GAP, GEAR_BUTTON,
    GearButton, Layer, MARGIN, MENU_WIDTH, SETTINGS_HEIGHT, SETTINGS_TITLE_ROW, SETTINGS_WIDTH,
    SystemMenu,
};
use crate::widgets::{
    ButtonState, ButtonStyle, ButtonTooltip, CLOSE_PIXELS, INK, Pressable, UiId, panel_title,
    spawn_button, spawn_close_button, spawn_dialog, spawn_icon_button, spawn_sized_layer,
};

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

pub(super) fn spawn_menu(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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
    super::panel::spawn_main_panel(&mut commands, icon.clone());
    spawn_requester(&mut commands);
    spawn_settings(&mut commands);
    super::settings::spawn_keep_requester(&mut commands);
    super::settings::spawn_ironman_requester(&mut commands);
    super::credits::spawn_credits(&mut commands);
    let icons = super::icons::ConnectIcons::new(&mut images);
    super::connect::spawn_connect(&mut commands, &icons);
    super::connect::spawn_popups(&mut commands);
    super::refusal::spawn_refusal(&mut commands);
    super::slow_drawing::spawn_slow_drawing(&mut commands);
    #[cfg(not(target_arch = "wasm32"))]
    {
        super::online::spawn_online_requesters(&mut commands);
        super::server_options::spawn_server_options(&mut commands);
    }
    commands.insert_resource(icons);
}

pub(super) fn wide_button() -> Node {
    Node {
        width: percent(100),
        height: px(BUTTON_ROW),
        ..default()
    }
}

/// A layer that presses outside its panel close.
pub(super) fn spawn_layer(
    commands: &mut Commands,
    depth: i32,
    shown: &'static [SystemMenu],
    width: Val,
    height: Val,
    content: Node,
) -> Entity {
    let (layer, panel) = spawn_sized_layer(commands, depth, width, height, content);
    commands.entity(layer).insert((Layer(shown), Action::Back));
    panel
}

pub(super) fn spawn_requester(commands: &mut Commands) {
    let dialog = spawn_dialog(commands, ABOVE_MENU_LAYER, "Quit");
    commands
        .entity(dialog.layer)
        .insert((Layer(&[SystemMenu::Confirming]), Action::Back));
    commands.entity(dialog.content).with_child((
        Text::new("Are you sure?"),
        FONT.font(),
        TextColor(INK),
    ));
    commands.entity(dialog.buttons).with_children(|row| {
        for (label, action, style) in [
            ("Yes, quit", Action::ReallyQuit, ButtonStyle::Main),
            ("No", Action::Back, ButtonStyle::Plain),
        ] {
            let node = Node {
                width: px(MENU_WIDTH / 2.0 - GAP),
                height: px(BUTTON_ROW),
                ..default()
            };
            spawn_button(row, ButtonState::Normal, label, FONT, node).insert((
                action,
                style,
                Pressable,
                action.ui_id("quit"),
            ));
        }
    });
}

/// A big layer's title, with a close button on the right.
/// Its close button is `close` in `form`.
pub(super) fn spawn_title_row(panel: &mut ChildSpawnerCommands, title: &str, form: &'static str) {
    panel
        .spawn(Node {
            height: px(SETTINGS_TITLE_ROW),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn(panel_title(title));
            row.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(-CLOSE_MARGIN),
                    top: px((SETTINGS_TITLE_ROW - CLOSE_PIXELS) / 2.0 - CLOSE_MARGIN),
                    padding: UiRect::all(px(CLOSE_MARGIN)),
                    ..default()
                },
                Action::Back,
                Pressable,
                UiId::new(form, "close"),
            ))
            .with_children(|holder| {
                spawn_close_button(holder, default());
            });
        });
}

pub(super) fn spawn_settings(commands: &mut Commands) {
    let panel = spawn_layer(
        commands,
        ABOVE_MENU_LAYER,
        &[
            SystemMenu::Settings,
            SystemMenu::KeepVideo,
            SystemMenu::Ironman,
        ],
        percent(SETTINGS_WIDTH),
        percent(SETTINGS_HEIGHT),
        Node {
            padding: UiRect::all(px(MARGIN)),
            flex_direction: FlexDirection::Column,
            row_gap: px(MARGIN),
            ..default()
        },
    );
    commands.entity(panel).with_children(|panel| {
        spawn_title_row(panel, "Settings", "settings");
        super::settings::spawn_pages(panel);
    });
}
