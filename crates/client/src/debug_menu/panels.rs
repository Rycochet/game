//! The moveable panels the menu opens: Previous and Next for what a place draws,
//! and one showing every widget.

use std::collections::HashMap;

use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::site::StairsStyle;

use super::actions::DebugAction;
use crate::common::{CurrentDungeon, GameView, HerbGrowthShift};
use crate::dungeon_map::{CROP_STAGES, HERB_STAGES};
use crate::widgets::{
    BUTTON_PADDING, ButtonPressed, ButtonState, Pressable, RaisePanel, raised_box, spawn_button,
    spawn_check_button, spawn_close_button, spawn_moveable_panel, spawn_tooltip,
};

const FONT: f32 = 16.0;
const CHECK_ROW: f32 = 34.0;
/// Where the first panel goes, and how far along each one after it.
const FIRST_AT: Vec2 = Vec2::new(60.0, 110.0);
const NEXT_AT: Vec2 = Vec2::new(30.0, 30.0);

/// The panels open, by what opened them. Closed ones are noticed when opening
/// them again.
#[derive(Resource, Default)]
pub(super) struct OpenPanels {
    panels: HashMap<DebugAction, Entity>,
    opened: u32,
}

/// A button that moves what `action`'s panel is for on or back a step.
#[derive(Component, Clone, Copy)]
pub(super) struct StepButton {
    action: DebugAction,
    forward: bool,
}

impl OpenPanels {
    /// Opens this action's panel, or if it is open already, brings it to the
    /// front. `nodes` says which panels are still there: closing despawns them.
    pub(super) fn open(
        &mut self,
        action: DebugAction,
        commands: &mut Commands,
        raise: &mut MessageWriter<RaisePanel>,
        nodes: &Query<(), With<Node>>,
    ) {
        if let Some(&panel) = self.panels.get(&action)
            && nodes.contains(panel)
        {
            raise.write(RaisePanel(panel));
            return;
        }
        let at = FIRST_AT + NEXT_AT * (self.opened % 8) as f32;
        self.opened += 1;
        let (panel, content) = spawn_moveable_panel(commands, action.name(), at);
        commands.entity(content).with_children(|content| {
            if action == DebugAction::ShowWidgets {
                spawn_every_widget(content);
            } else {
                spawn_step_buttons(content, action);
            }
        });
        self.panels.insert(action, panel);
    }
}

fn spawn_step_buttons(content: &mut ChildSpawnerCommands, action: DebugAction) {
    content
        .spawn(Node {
            column_gap: px(4),
            ..default()
        })
        .with_children(|row| {
            for (label, forward) in [("Previous", false), ("Next", true)] {
                let node = Node {
                    padding: BUTTON_PADDING,
                    ..default()
                };
                spawn_button(row, ButtonState::Normal, label, FONT, node)
                    .insert((StepButton { action, forward }, Pressable));
            }
        });
}

/// One of each thing `widgets` makes, to see how they look together.
fn spawn_every_widget(content: &mut ChildSpawnerCommands) {
    for (state, label) in [
        (ButtonState::Normal, "Normal"),
        (ButtonState::Selected, "Selected"),
        (ButtonState::Active, "Active"),
        (ButtonState::Disabled, "Disabled"),
        (ButtonState::DisabledSelected, "Disabled, selected"),
    ] {
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, state, label, FONT, node).insert(Pressable);
    }
    for (on, label) in [
        (Some(true), "Check on"),
        (Some(false), "Check off"),
        (None, "No check"),
    ] {
        let node = Node {
            height: px(CHECK_ROW),
            ..default()
        };
        spawn_check_button(content, ButtonState::Normal, on, label, FONT, node).insert(Pressable);
    }
    spawn_close_button(content, default());
    content.spawn(raised_box()).with_child((
        Text::new("Raised box"),
        TextFont::from_font_size(FONT),
        TextColor(crate::widgets::INK),
    ));
    content
        .spawn(Node::default())
        .with_children(|holder| spawn_tooltip(holder, SampleTooltip, "Tooltip"));
}

/// The sample tooltip's text.
#[derive(Component)]
pub(super) struct SampleTooltip;

/// Whether a panel's buttons can do anything here.
fn can_step(action: DebugAction, view: GameView, dungeon: &CurrentDungeon) -> bool {
    view == GameView::Dungeon
        && match action {
            DebugAction::StairsStyle => !dungeon.stairs.is_empty(),
            DebugAction::FieldsState => !dungeon.fields.is_empty(),
            DebugAction::HerbsState => dungeon.level_has(Cell::HerbBed),
            _ => true,
        }
}

/// The style before this one, going round.
fn previous_style(style: StairsStyle) -> StairsStyle {
    let all = StairsStyle::ALL;
    let at = all.iter().position(|&other| other == style).unwrap_or(0);
    all[(at + all.len() - 1) % all.len()]
}

/// Moves what the pressed button's panel is for on or back a step: only what is
/// drawn, and the next level goes back to its own.
pub(super) fn press_panel_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&StepButton>,
    mut dungeon: ResMut<CurrentDungeon>,
    mut shift: ResMut<HerbGrowthShift>,
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
            _ => {}
        }
    }
}

/// Greys the buttons where they can do nothing; the panel stays open.
pub(super) fn update_panel_buttons(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previous_undoes_next_for_every_style() {
        for style in StairsStyle::ALL {
            assert_eq!(previous_style(style.next()), style);
        }
    }

    #[test]
    fn panels_need_a_site_with_what_they_change() {
        let dungeon = CurrentDungeon::default();
        let in_site = |action| can_step(action, GameView::Dungeon, &dungeon);
        assert!(!in_site(DebugAction::StairsStyle));
        assert!(!in_site(DebugAction::FieldsState));
        assert!(!in_site(DebugAction::HerbsState));
        assert!(!can_step(
            DebugAction::StairsStyle,
            GameView::WorldMap,
            &dungeon
        ));
    }
}
