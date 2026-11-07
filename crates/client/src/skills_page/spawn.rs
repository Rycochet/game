//! The page's parts, put in the Character panel's Skills tab whenever that panel
//! opens: a row of buttons (zoom out, zoom in, fit), the tree's canvas and a line
//! of details under it.

use bevy::prelude::*;

use crate::common::SkillsPage;
use crate::widgets::{
    BUTTON_PADDING, ButtonState, ButtonTooltip, INK, Pressable, TextSize, UiId, leaves_presses,
    spawn_button,
};

/// The form all its widgets' ids are in.
pub(super) const FORM: &str = "skills";
/// The canvas's size: the panel fits its pages, so this is the tab's.
const CANVAS_WIDTH: f32 = 640.0;
const CANVAS_HEIGHT: f32 = 480.0;

/// Where the tree is drawn: presses on it are the page's to read.
#[derive(Component)]
pub(super) struct Canvas;
/// The middle of the canvas, which the tree's nodes are placed from.
#[derive(Component)]
pub(super) struct Origin;
/// The line under the canvas saying about the chosen skill.
#[derive(Component)]
pub(super) struct Details;

/// What a button above the tree does.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum PageButton {
    ZoomOut,
    ZoomIn,
    Fit,
}

/// Fills each new Skills tab.
pub(super) fn fill_page(pages: Query<Entity, Added<SkillsPage>>, mut commands: Commands) {
    for page in &pages {
        commands.entity(page).with_children(|page| {
            page.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            })
            .with_children(spawn_parts);
        });
    }
}

fn spawn_parts(column: &mut ChildSpawnerCommands) {
    spawn_buttons(column);
    let canvas = column
        .spawn((
            Canvas,
            Node {
                width: px(CANVAS_WIDTH),
                height: px(CANVAS_HEIGHT),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.25, 0.17, 0.1, 0.06)),
        ))
        .id();
    column
        .commands()
        .entity(canvas)
        .insert(leaves_presses(canvas))
        .with_child((
            Origin,
            leaves_presses(canvas),
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(50),
                ..default()
            },
        ));
    column.spawn((
        Details,
        Text::default(),
        TextSize::Medium.font(),
        TextColor(INK),
        Node {
            max_width: px(CANVAS_WIDTH),
            ..default()
        },
    ));
}

fn spawn_buttons(column: &mut ChildSpawnerCommands) {
    column
        .spawn(Node {
            justify_content: JustifyContent::End,
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
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
        });
}
