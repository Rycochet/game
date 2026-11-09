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
use super::{FONT, OpenPanels, row};
use crate::common::{InspectedMob, Other, ServerDevTools, Target};
use crate::connection::{FromServer, ToServer};
use crate::widgets::{
    BUTTON_PADDING, ButtonPressed, ButtonState, INK, Pressable, RaisePanel, UiId, spawn_button,
};

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

pub(super) fn spawn_examine_panel(content: &mut ChildSpawnerCommands) {
    let column = Node {
        flex_direction: FlexDirection::Column,
        min_width: px(320),
        ..default()
    };
    content.spawn((column, ExamineLines));
    content.spawn(row()).with_children(|row| {
        for (label, button, name) in [
            ("Kill", ExamineButton::Kill, "kill"),
            ("Destroy", ExamineButton::Destroy, "destroy"),
            ("Give Item...", ExamineButton::GiveItem, "give_item"),
        ] {
            let node = Node {
                padding: BUTTON_PADDING,
                ..default()
            };
            spawn_button(row, ButtonState::Normal, label, FONT, node).insert((
                button,
                Pressable,
                UiId::new("debug.examine_mob", name),
            ));
        }
    });
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
fn show_lines(
    inspected: Res<InspectedMob>,
    holders: Query<(Entity, Ref<ExamineLines>)>,
    mut commands: Commands,
) {
    for (holder, added) in &holders {
        if !inspected.is_changed() && !added.is_added() {
            continue;
        }
        commands.entity(holder).despawn_related::<Children>();
        commands.entity(holder).with_children(|column| {
            for line in &inspected.lines {
                column.spawn((Text::new(line.clone()), FONT.font(), TextColor(INK)));
            }
        });
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

/// Greys the buttons with no mob targeted, or without the server's dev tools.
fn grey_buttons(
    target: Res<Target>,
    others: Query<&Other>,
    dev_tools: Res<ServerDevTools>,
    mut buttons: Query<&mut ButtonState, With<ExamineButton>>,
) {
    let can = dev_tools.0 && targeted_mob(&target, &others).is_some();
    let state = if can {
        ButtonState::Normal
    } else {
        ButtonState::Disabled
    };
    for mut button in &mut buttons {
        button.set_if_neq(state);
    }
}
