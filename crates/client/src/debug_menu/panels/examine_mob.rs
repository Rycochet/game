//! The Examine Mob panel (owner's design, 2026-11-09): what the server knows of
//! the targeted mob, asked again twice a second while open so it follows it,
//! with buttons to kill it, destroy it (gone for good, leaving nothing) or open
//! Give Item, which gives to the target. It takes the place of the Kill, Inspect
//! and Delete Mob click modes.

use bevy::prelude::*;
use shared::id::Id;
use shared::protocol::{ClientMessage, DevMobTool, ServerMessage};
use shared::sheet::Feeling;

use super::super::actions::DebugAction;
use super::OpenPanels;
use crate::common::{DebugTools, InspectedMob, Other, Target};
use crate::connection::{FromServer, ToServer};
use crate::dom::{Dom, Template};
use crate::html;
use crate::widgets::{ButtonPressed, ButtonState, RaisePanel, UiId};
use shared::role::DebugTool;

/// How often an open panel asks again, in seconds.
const ASK_EVERY: f32 = 0.5;
/// What the panel says with no mob targeted.
const NO_MOB: &str = "No mob targeted: right-click or long press one.";

/// Holds the lines.
#[derive(Component)]
pub(in crate::debug_menu) struct ExamineLines;

/// What the panel's buttons do to the mob.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(in crate::debug_menu) enum ExamineButton {
    Kill,
    Destroy,
    GiveItem,
}

/// Adds the panel's systems to the debug menu's plugin.
pub(in crate::debug_menu) fn add_examine_mob_panel(app: &mut App) {
    app.add_systems(
        Update,
        (
            take_mob_info,
            ask_again,
            show_lines,
            press_buttons,
            grey_buttons,
        )
            .chain(),
    );
}

/// The panel's contents, as markup. Each line the server sends is a copy of
/// the template.
pub(super) static EXAMINE_MOB: Template = html!(
    "debug_menu.examine_mob",
    r#"
<form id="debug.examine_mob">
  <div id="lines" class="tight"/>
  <span nowrap>
    <button id="kill">Kill</button>
    <button id="destroy">Destroy</button>
    <button id="give_item">Give Item...</button>
  </span>
  <template id="line"><p id="line"/></template>
</form>
"#
);

/// The buttons, by their names in the markup.
const BUTTONS: [(&str, ExamineButton, DebugTool); 3] = [
    ("kill", ExamineButton::Kill, DebugTool::KillMob),
    ("destroy", ExamineButton::Destroy, DebugTool::DestroyMob),
    ("give_item", ExamineButton::GiveItem, DebugTool::GiveItem),
];

/// What a built part of the panel is known by, from its id.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId) {
    let name = id.parts().1;
    if name == "lines" {
        bound.insert(ExamineLines);
    } else if let Some((_, button, _)) = BUTTONS.iter().find(|(known, ..)| *known == name) {
        bound.insert(*button);
    }
}

/// The target, if it is a mob in sight (other players feel nothing).
fn targeted_mob(target: &Target, others: &Query<&Other>) -> Option<Id> {
    let id = target.0?;
    let other = others.iter().find(|other| other.id == id)?;
    (other.feeling != Feeling::None).then_some(id)
}

/// The server's answer, kept for the panel (and the debug API).
fn take_mob_info(mut from_server: MessageReader<FromServer>, mut inspected: ResMut<InspectedMob>) {
    for FromServer(message) in from_server.read() {
        if let ServerMessage::DevMobInfo { mob, lines } = message {
            inspected.mob = Some(*mob);
            inspected.lines.clone_from(lines);
        }
    }
}

/// While the panel is open, asks after the targeted mob now and then; with
/// none, says so.
fn ask_again(
    open: Res<OpenPanels>,
    target: Res<Target>,
    others: Query<&Other>,
    mut inspected: ResMut<InspectedMob>,
    time: Res<Time<Real>>,
    mut since: Local<f32>,
    mut to_server: MessageWriter<ToServer>,
) {
    *since += time.delta_secs();
    if !open.is_open(DebugAction::ExamineMob) {
        return;
    }
    let Some(mob) = targeted_mob(&target, &others) else {
        if inspected.mob.is_some() || inspected.lines.is_empty() {
            inspected.mob = None;
            inspected.lines = vec![NO_MOB.to_owned()];
        }
        return;
    };
    if *since >= ASK_EVERY || inspected.mob != Some(mob) {
        *since = 0.0;
        to_server.write(ToServer(ClientMessage::DevMobs(DevMobTool::Inspect(mob))));
    }
}

/// The lines, written out again when they change or the panel opens.
fn show_lines(inspected: Res<InspectedMob>, holders: Query<Ref<ExamineLines>>, mut dom: Dom) {
    for added in &holders {
        if !inspected.is_changed() && !added.is_added() {
            continue;
        }
        dom.empty("debug.examine_mob/lines");
        for (at, line) in inspected.lines.iter().enumerate() {
            dom.copy_template("debug.examine_mob/line", "debug.examine_mob/lines", at);
            dom.set_text(&format!("debug.examine_mob/line.{at}"), line.clone());
        }
    }
}

/// Kills or destroys the targeted mob, or opens Give Item for it.
#[allow(clippy::too_many_arguments)]
fn press_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&ExamineButton>,
    target: Res<Target>,
    others: Query<&Other>,
    mut open: ResMut<OpenPanels>,
    mut commands: Commands,
    mut raise: MessageWriter<RaisePanel>,
    nodes: Query<(), With<Node>>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        let (Ok(button), Some(mob)) = (buttons.get(press.0), targeted_mob(&target, &others)) else {
            continue;
        };
        let message = match button {
            ExamineButton::Kill => ClientMessage::DevKillMob { mob },
            ExamineButton::Destroy => ClientMessage::DevMobs(DevMobTool::Delete(mob)),
            ExamineButton::GiveItem => {
                open.open(DebugAction::GiveItem, &mut commands, &mut raise, &nodes);
                continue;
            }
        };
        to_server.write(ToServer(message));
    }
}

/// Greys the buttons with no mob targeted, and hides each without its own
/// permission (owner: Give Item... too).
fn grey_buttons(
    target: Res<Target>,
    others: Query<&Other>,
    tools: Res<DebugTools>,
    mut buttons: Query<&mut ButtonState, With<ExamineButton>>,
    mut dom: Dom,
    mut shown: Local<Option<[bool; 3]>>,
) {
    let state = if targeted_mob(&target, &others).is_some() {
        ButtonState::Normal
    } else {
        ButtonState::Disabled
    };
    // Again for a panel just opened, which shows them all.
    if buttons.is_empty() {
        *shown = None;
        return;
    }
    for mut button_state in &mut buttons {
        button_state.set_if_neq(state);
    }
    let may = BUTTONS.map(|(_, _, tool)| tools.may(tool));
    if *shown != Some(may) {
        *shown = Some(may);
        for ((name, ..), may) in BUTTONS.into_iter().zip(may) {
            dom.set_flag(&format!("debug.examine_mob/{name}"), "hidden", !may);
        }
    }
}
