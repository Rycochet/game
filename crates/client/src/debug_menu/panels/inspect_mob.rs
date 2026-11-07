//! The Inspect Mob panel: what the server knows of the mob last clicked in
//! Inspect Mob, opened when its answer comes and asked again twice a second
//! while open, so it follows the mob.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, DevMobTool, ServerMessage};

use super::super::actions::DebugAction;
use super::{FONT, OpenPanels};
use crate::common::InspectedMob;
use crate::connection::{FromServer, ToServer};
use crate::widgets::{INK, RaisePanel};

/// How often an open panel asks again, in seconds.
const ASK_EVERY: f32 = 0.5;

/// Holds the lines.
#[derive(Component)]
pub(in crate::debug_menu) struct InspectLines;

/// Adds the panel's systems to the debug menu's plugin.
pub(in crate::debug_menu) fn add_inspect_mob_panel(app: &mut App) {
    app.add_systems(Update, (take_mob_info, ask_again, show_lines).chain());
}

pub(super) fn spawn_inspect_panel(content: &mut ChildSpawnerCommands) {
    let column = Node {
        flex_direction: FlexDirection::Column,
        min_width: px(320),
        ..default()
    };
    content.spawn((column, InspectLines));
}

/// The server's answer: kept, and the panel opened (or brought to the front).
fn take_mob_info(
    mut from_server: MessageReader<FromServer>,
    mut inspected: ResMut<InspectedMob>,
    mut open: ResMut<OpenPanels>,
    mut commands: Commands,
    mut raise: MessageWriter<RaisePanel>,
    nodes: Query<(), With<Node>>,
) {
    for FromServer(message) in from_server.read() {
        if let ServerMessage::DevMobInfo { mob, lines } = message {
            let first = inspected.mob != Some(*mob) || !open.is_open(DebugAction::InspectMob);
            inspected.mob = Some(*mob);
            inspected.lines.clone_from(lines);
            if first {
                open.open(DebugAction::InspectMob, &mut commands, &mut raise, &nodes);
            }
        }
    }
}

/// While the panel is open, asks after its mob again now and then.
fn ask_again(
    open: Res<OpenPanels>,
    inspected: Res<InspectedMob>,
    time: Res<Time<Real>>,
    mut since: Local<f32>,
    mut to_server: MessageWriter<ToServer>,
) {
    *since += time.delta_secs();
    let Some(mob) = inspected.mob else {
        return;
    };
    if *since >= ASK_EVERY && open.is_open(DebugAction::InspectMob) {
        *since = 0.0;
        to_server.write(ToServer(ClientMessage::DevMobs(DevMobTool::Inspect(mob))));
    }
}

/// The lines, written out again when they change or the panel opens.
fn show_lines(
    inspected: Res<InspectedMob>,
    holders: Query<(Entity, Ref<InspectLines>)>,
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
