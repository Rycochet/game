//! The page's parts, spawned once and shown while it is open: a full-screen layer
//! with a title row (zoom out, zoom in, fit, close), the tree's canvas and a line
//! of details under it.

use bevy::prelude::*;

use crate::widgets::{
    BUTTON_PADDING, ButtonState, ButtonTooltip, INK, Pressable, TextSize, UiId, leaves_presses,
    panel_title, spawn_button, spawn_close_button, spawn_fullscreen_layer,
};

/// The form all its widgets' ids are in.
pub(super) const FORM: &str = "skills";

#[derive(Component)]
pub(super) struct SkillsLayer;
/// Where the tree is drawn: presses on it are the page's to read.
#[derive(Component)]
pub(super) struct Canvas;
/// The middle of the canvas, which the tree's nodes are placed from.
#[derive(Component)]
pub(super) struct Origin;
/// The line under the canvas saying about the chosen skill.
#[derive(Component)]
pub(super) struct Details;

/// What a button on the title row does.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum PageButton {
    ZoomOut,
    ZoomIn,
    Fit,
    Close,
}

pub(super) fn spawn_page(mut commands: Commands) {
    let content = Node {
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(12)),
        row_gap: px(8),
        ..default()
    };
    let (layer, panel) = spawn_fullscreen_layer(&mut commands, 0, content);
    commands.entity(layer).insert(SkillsLayer);
    commands.entity(panel).with_children(|panel| {
        spawn_title_row(panel);
        let canvas = panel
            .spawn((
                Canvas,
                Node {
                    flex_grow: 1.0,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.25, 0.17, 0.1, 0.06)),
            ))
            .id();
        panel
            .commands()
            .entity(canvas)
            .insert(leaves_presses(canvas));
        panel.commands().entity(canvas).with_child((
            Origin,
            leaves_presses(canvas),
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(50),
                ..default()
            },
        ));
        panel.spawn((
            Details,
            Text::default(),
            TextSize::Medium.font(),
            TextColor(INK),
        ));
    });
}

fn spawn_title_row(panel: &mut ChildSpawnerCommands) {
    panel
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                panel_title("Skills"),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            for (button, label, name, tip) in [
                (PageButton::ZoomOut, "-", "zoom_out", "Zoom out"),
                (PageButton::ZoomIn, "+", "zoom_in", "Zoom in"),
                (PageButton::Fit, "Fit", "fit", "Show the whole tree"),
            ] {
                spawn_button(
                    row,
                    ButtonState::Normal,
                    label,
                    TextSize::Medium,
                    Node {
                        padding: BUTTON_PADDING,
                        min_width: px(36),
                        ..default()
                    },
                )
                .insert((
                    button,
                    Pressable,
                    UiId::new(FORM, name),
                    ButtonTooltip(tip.into()),
                ));
            }
            spawn_close_button(row, Node::default()).insert((
                PageButton::Close,
                Pressable,
                UiId::new(FORM, "close"),
            ));
        });
}
