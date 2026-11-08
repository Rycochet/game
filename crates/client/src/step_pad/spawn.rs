//! Spawning the pad, hidden until there is a map to walk on, and its toggle
//! button in the bottom-right corner.

use bevy::prelude::*;

use super::icons::{Pointing, arrow, keypad, ladder};
use super::{BUTTON, CORNER_GAP, FORM, GAP, TOGGLE};
use crate::common::PadButton;
use crate::widgets::{
    ButtonState, ButtonTooltip, HoldButton, HoverTip, PanelPlace, Pressable, UiId,
    spawn_grip_panel, spawn_icon_button,
};

/// The pad's panel.
#[derive(Component)]
pub(super) struct StepPad;

/// The button that opens and closes it.
#[derive(Component)]
pub(super) struct PadToggle;

/// The pad's two rows, left to right: each button, its widget name and its
/// first tip (`site` keeps Up's and Down's).
const ROWS: [[(PadButton, &str, &str); 3]; 2] = [
    [
        (PadButton::Up, "up", "Up"),
        (PadButton::StepUp, "step_up", "Step Up"),
        (PadButton::Down, "down", "Down"),
    ],
    [
        (PadButton::StepLeft, "step_left", "Step Left"),
        (PadButton::StepDown, "step_down", "Step Down"),
        (PadButton::StepRight, "step_right", "Step Right"),
    ],
];

/// The picture on one of the pad's buttons.
fn picture_of(button: PadButton) -> Image {
    match button {
        PadButton::Up => ladder(true),
        PadButton::Down => ladder(false),
        PadButton::StepUp => arrow(Pointing::Up),
        PadButton::StepDown => arrow(Pointing::Down),
        PadButton::StepLeft => arrow(Pointing::Left),
        PadButton::StepRight => arrow(Pointing::Right),
    }
}

pub(super) fn spawn_pad(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let icon = images.add(keypad());
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(CORNER_GAP),
                right: px(CORNER_GAP),
                ..default()
            },
            // Above the maps, as the other corner buttons are.
            GlobalZIndex(15),
        ))
        .with_children(|holder| {
            let node = Node {
                width: px(TOGGLE),
                height: px(TOGGLE),
                display: Display::None,
                ..default()
            };
            spawn_icon_button(holder, ButtonState::Normal, icon, node).insert((
                PadToggle,
                Pressable,
                UiId::new("hud", "step_pad"),
                ButtonTooltip("Movement Pad".into()),
            ));
        });

    // Just above the toggle button, to start with.
    let place = PanelPlace::BottomRight(Vec2::new(CORNER_GAP, CORNER_GAP + TOGGLE + GAP));
    let (panel, content) = spawn_grip_panel(&mut commands, place);
    commands
        .entity(panel)
        .insert((StepPad, UiId::new(FORM, "panel")));
    for row in ROWS {
        let pictures = row.map(|(button, ..)| images.add(picture_of(button)));
        let row = commands
            .spawn(Node {
                column_gap: px(GAP),
                ..default()
            })
            .with_children(|row_of| {
                for ((button, name, tip), picture) in row.into_iter().zip(pictures) {
                    let node = Node {
                        width: px(BUTTON),
                        height: px(BUTTON),
                        ..default()
                    };
                    spawn_icon_button(row_of, ButtonState::Normal, picture, node).insert((
                        button,
                        HoldButton::default(),
                        HoverTip(tip.to_string()),
                        UiId::new(FORM, name),
                    ));
                }
            })
            .id();
        commands.entity(content).add_child(row);
    }
}
