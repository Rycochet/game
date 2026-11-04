//! The panels open, by what opened them; opening, closing and forgetting them.

use bevy::prelude::*;
use std::collections::HashMap;

use super::super::actions::DebugAction;
use super::super::bodies::BodiesToShow;
use super::{
    FIRST_AT, NEXT_AT, spawn_creature_buttons, spawn_every_widget, spawn_give_buttons,
    spawn_item_pictures, spawn_step_buttons, spawn_world_time,
};
use crate::widgets::{RaisePanel, spawn_moveable_panel};

/// The panels open, by what opened them.
#[derive(Resource, Default)]
pub(in crate::debug_menu) struct OpenPanels {
    pub(super) panels: HashMap<DebugAction, Entity>,
    pub(super) opened: u32,
}

impl OpenPanels {
    pub(in crate::debug_menu) fn is_open(&self, action: DebugAction) -> bool {
        self.panels.contains_key(&action)
    }

    /// Closes this action's panel if it is open, otherwise opens it.
    pub(in crate::debug_menu) fn toggle(
        &mut self,
        action: DebugAction,
        commands: &mut Commands,
        raise: &mut MessageWriter<RaisePanel>,
        nodes: &Query<(), With<Node>>,
    ) {
        match self.panels.remove(&action) {
            Some(panel) if nodes.contains(panel) => commands.entity(panel).despawn(),
            _ => self.open(action, commands, raise, nodes),
        }
    }

    /// Opens this action's panel, or if it is open already, brings it to the
    /// front. `nodes` says which panels are still there: closing despawns them.
    pub(in crate::debug_menu) fn open(
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
        commands
            .entity(content)
            .with_children(|content| match action {
                DebugAction::ShowWidgets => spawn_every_widget(content),
                DebugAction::SunsAndMoons => {}
                DebugAction::GiveItem => spawn_give_buttons(content),
                DebugAction::ChangeCreature => spawn_creature_buttons(content),
                DebugAction::ItemPictures => spawn_item_pictures(content),
                DebugAction::WorldTime => spawn_world_time(content),
                _ => spawn_step_buttons(content, action),
            });
        if action == DebugAction::SunsAndMoons {
            commands.entity(content).insert(BodiesToShow);
        }
        self.panels.insert(action, panel);
    }
}

/// Forgets panels closed by their own close buttons.
pub(in crate::debug_menu) fn forget_closed(
    mut open: ResMut<OpenPanels>,
    nodes: Query<(), With<Node>>,
) {
    let closed = open.panels.values().any(|&panel| !nodes.contains(panel));
    if closed {
        open.panels.retain(|_, panel| nodes.contains(*panel));
    }
}
