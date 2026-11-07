//! The panels open, by what opened them; opening, closing and forgetting them.

use bevy::prelude::*;
use std::collections::HashMap;

use super::super::actions::DebugAction;
use super::super::bodies::BodiesToShow;
use super::{
    FIRST_AT, NEXT_AT, spawn_create_mob_panel, spawn_creature_buttons, spawn_dev_area_panel,
    spawn_every_widget, spawn_give_panel, spawn_go_to_world, spawn_inspect_panel,
    spawn_item_pictures, spawn_step_buttons, spawn_world_time,
};
use crate::widgets::{RaisePanel, UiId, spawn_moveable_panel};

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
                DebugAction::GiveItem => spawn_give_panel(content),
                DebugAction::ChangeCreature => spawn_creature_buttons(content),
                DebugAction::ItemPictures => spawn_item_pictures(content),
                DebugAction::WorldTime => spawn_world_time(content),
                DebugAction::GoToWorld => spawn_go_to_world(content),
                DebugAction::DevArea => spawn_dev_area_panel(content),
                DebugAction::CreateMob => spawn_create_mob_panel(content),
                DebugAction::InspectMob => spawn_inspect_panel(content),
                _ => spawn_step_buttons(content, action),
            });
        commands
            .entity(panel)
            .insert(UiId::new(form_of(action), "panel"));
        if action == DebugAction::SunsAndMoons {
            commands.entity(content).insert(BodiesToShow);
        }
        self.panels.insert(action, panel);
    }
}

/// The form a panel's widgets are in, for the debug API.
pub(in crate::debug_menu) fn form_of(action: DebugAction) -> &'static str {
    match action {
        DebugAction::ShowWidgets => "debug.every_widget",
        DebugAction::SunsAndMoons => "debug.suns_and_moons",
        DebugAction::GiveItem => "debug.give_item",
        DebugAction::ChangeCreature => "debug.change_creature",
        DebugAction::ItemPictures => "debug.item_pictures",
        DebugAction::WorldTime => "debug.world_time",
        DebugAction::GoToWorld => "debug.go_to_world",
        DebugAction::DevArea => "debug.dev_area",
        DebugAction::CreateMob => "debug.create_mob",
        DebugAction::InspectMob => "debug.inspect_mob",
        DebugAction::StairsStyle => "debug.stairs",
        DebugAction::FieldsState => "debug.fields",
        DebugAction::HerbsState => "debug.herbs",
        _ => "debug.steps",
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
