//! Step buttons moving what a panel is for on or back, and greying them where they can't.

use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::protocol::ClientMessage;
use shared::site::StairsStyle;
use shared::world::MINUTES_PER_HOUR;

use super::super::actions::DebugAction;
use super::FONT;
use crate::common::{CurrentDungeon, GameView, HerbGrowthShift, ServerClock};
use crate::connection::ToServer;
use crate::dungeon_map::{CROP_STAGES, HERB_STAGES};
use crate::widgets::{BUTTON_PADDING, ButtonPressed, ButtonState, Pressable, spawn_button};

/// A button that moves what `action`'s panel is for on or back a step.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct StepButton {
    action: DebugAction,
    forward: bool,
}

pub(super) fn spawn_step_buttons(content: &mut ChildSpawnerCommands, action: DebugAction) {
    content
        .spawn(Node {
            column_gap: px(4),
            ..default()
        })
        .with_children(|row| {
            let [back, on] = if action == DebugAction::WorldTime {
                ["-1 hour", "+1 hour"]
            } else {
                ["Previous", "Next"]
            };
            for (label, forward) in [(back, false), (on, true)] {
                let node = Node {
                    padding: BUTTON_PADDING,
                    ..default()
                };
                spawn_button(row, ButtonState::Normal, label, FONT, node)
                    .insert((StepButton { action, forward }, Pressable));
            }
        });
}

/// Whether a panel's buttons can do anything here.
pub(super) fn can_step(action: DebugAction, view: GameView, dungeon: &CurrentDungeon) -> bool {
    action == DebugAction::WorldTime
        || view == GameView::Dungeon
            && match action {
                DebugAction::StairsStyle => !dungeon.stairs.is_empty(),
                DebugAction::FieldsState => !dungeon.fields.is_empty(),
                DebugAction::HerbsState => dungeon.level_has(Cell::HerbBed),
                _ => true,
            }
}

/// The style before this one, going round.
pub(super) fn previous_style(style: StairsStyle) -> StairsStyle {
    let all = StairsStyle::ALL;
    let at = all.iter().position(|&other| other == style).unwrap_or(0);
    all[(at + all.len() - 1) % all.len()]
}

/// Moves what the pressed button's panel is for on or back a step: only what is
/// drawn, and the next level goes back to its own.
pub(in crate::debug_menu) fn press_panel_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&StepButton>,
    mut dungeon: ResMut<CurrentDungeon>,
    mut shift: ResMut<HerbGrowthShift>,
    clock: Res<ServerClock>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        let Ok(&StepButton { action, forward }) = buttons.get(press.0) else {
            continue;
        };
        match action {
            DebugAction::StairsStyle => {
                for stairs in &mut dungeon.stairs {
                    stairs.style = if forward {
                        stairs.style.next()
                    } else {
                        previous_style(stairs.style)
                    };
                }
            }
            DebugAction::FieldsState => {
                for field in &mut dungeon.fields {
                    // As crops, which alone go through every stage.
                    field.livestock = false;
                    let by = if forward { 1 } else { CROP_STAGES as u32 - 1 };
                    field.variety = field.variety.wrapping_add(by);
                }
            }
            DebugAction::HerbsState => {
                // Back is nearly all the way round, as the shift can't go below 0.
                let by = if forward { 1 } else { HERB_STAGES - 1 };
                shift.0 = (shift.0 + by) % HERB_STAGES;
            }
            DebugAction::WorldTime => {
                let by = if forward {
                    MINUTES_PER_HOUR
                } else {
                    -MINUTES_PER_HOUR
                };
                to_server.write(ToServer(ClientMessage::DevSetTime {
                    minutes: clock.minutes + by,
                }));
            }
            _ => {}
        }
    }
}

/// Greys the buttons where they can do nothing; the panel stays open.
pub(in crate::debug_menu) fn update_panel_buttons(
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    mut buttons: Query<(&StepButton, &mut ButtonState)>,
) {
    for (button, mut state) in &mut buttons {
        state.set_if_neq(if can_step(button.action, *view.get(), &dungeon) {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
}
