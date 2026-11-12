//! The panels open, by what opened them; opening, closing and forgetting them.

use bevy::prelude::*;
use std::collections::HashMap;

use super::super::actions::DebugAction;
use super::{
    FIRST_AT, NEXT_AT, change_creature, create_mob, dev_area, examine_mob, generate, give,
    learn_skills, pictures_spawn, steps, teleport_to_world, vitals, world_time,
};
use crate::dom::{Dom, Template, UiEvent, UiValue, document, top_strip};
use crate::html;

/// Show Widgets' panel: its sample (`html_sample`) is built into its holder.
static EVERY_WIDGET: Template = html!(
    "debug_menu.every_widget",
    r#"
<form id="debug.every_widget">
  <panel id="panel" moveable>
    <scroll id="scroll" max-height="80%">
      <div id="holder"/>
    </scroll>
  </panel>
</form>
"#
);

/// Suns and Moons' panel: `bodies` fills its holder once there is a world.
static SUNS_AND_MOONS: Template = html!(
    "debug_menu.suns_and_moons_panel",
    r#"
<form id="debug.suns_and_moons">
  <panel id="panel" moveable>
    <div id="holder"/>
  </panel>
</form>
"#
);

/// The panels open, by what opened them: each its document, whose panel
/// (`<form>/panel`) goes with it.
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
        dom: &mut Dom,
        nodes: &Query<(), With<Node>>,
    ) {
        match self.panels.remove(&action) {
            Some(document) if nodes.contains(document) => commands.entity(document).despawn(),
            _ => self.open(action, commands, dom, nodes),
        }
    }

    /// Opens this action's panel, or if it is open already, brings it to the
    /// front. `nodes` says which documents are still there.
    pub(in crate::debug_menu) fn open(
        &mut self,
        action: DebugAction,
        commands: &mut Commands,
        dom: &mut Dom,
        nodes: &Query<(), With<Node>>,
    ) {
        let Some(template) = template_of(action) else {
            return;
        };
        let panel = format!("{}/panel", form_of(action));
        if self
            .panels
            .get(&action)
            .is_some_and(|&document| nodes.contains(document))
        {
            dom.raise(&panel);
            return;
        }
        let at = FIRST_AT + NEXT_AT * (self.opened % 8) as f32;
        self.opened += 1;
        // (On a strip: its own node takes no room over the view.)
        let mut document = commands.spawn(document(template));
        self.panels
            .insert(action, document.insert(top_strip()).id());
        // Made at the end of the frame, once the panel is built.
        dom.set(&panel, "title", action.name());
        dom.start_at(&panel, at);
    }
}

/// A panel as markup, for the actions that open one.
fn template_of(action: DebugAction) -> Option<&'static Template> {
    match action {
        DebugAction::ShowWidgets => Some(&EVERY_WIDGET),
        DebugAction::SunsAndMoons => Some(&SUNS_AND_MOONS),
        DebugAction::GeneratePoi => Some(&generate::GENERATE_POI),
        DebugAction::ChangeCreature => Some(&change_creature::CHANGE_CREATURE),
        DebugAction::Vitals => Some(&vitals::VITALS),
        DebugAction::DrawingStates => Some(&steps::DRAWING_STATES),
        DebugAction::WorldTime => Some(&world_time::WORLD_TIME),
        DebugAction::ExamineMob => Some(&examine_mob::EXAMINE_MOB),
        DebugAction::CreateMob => Some(&create_mob::CREATE_MOB),
        DebugAction::DevArea => Some(&dev_area::DEV_AREA),
        DebugAction::LearnSkills => Some(&learn_skills::LEARN_SKILLS),
        DebugAction::GiveItem => Some(&give::GIVE_ITEM),
        DebugAction::ItemPictures => Some(&pictures_spawn::ITEM_PICTURES),
        DebugAction::TeleportToWorld => Some(&teleport_to_world::TELEPORT_TO_WORLD),
        _ => None,
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
        DebugAction::TeleportToWorld => super::teleport_to_world::FORM,
        DebugAction::Vitals => "debug.vitals",
        DebugAction::DevArea => "debug.dev_area",
        DebugAction::CreateMob => "debug.create_mob",
        DebugAction::ExamineMob => "debug.examine_mob",
        DebugAction::LearnSkills => super::learn_skills::FORM,
        DebugAction::DrawingStates => "debug.drawing_states",
        DebugAction::GeneratePoi => "debug.generate_poi",
        _ => "debug.panel",
    }
}

/// Forgets panels closed by their own close buttons, which say so by their
/// ids: their documents go then.
pub(in crate::debug_menu) fn forget_closed(
    mut events: MessageReader<UiEvent>,
    mut open: ResMut<OpenPanels>,
    nodes: Query<(), With<Node>>,
    mut commands: Commands,
) {
    for event in events.read() {
        let (form, "panel") = event.id.parts() else {
            continue;
        };
        let known = |action: &DebugAction| form_of(*action) == form;
        let closed = open.panels.keys().copied().find(known);
        if let Some(action) = closed.filter(|_| event.value == UiValue::Closed)
            && let Some(document) = open.panels.remove(&action)
            && nodes.contains(document)
        {
            commands.entity(document).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_shells_are_good_markup() {
        super::EVERY_WIDGET.assert_good();
        super::SUNS_AND_MOONS.assert_good();
    }
}
