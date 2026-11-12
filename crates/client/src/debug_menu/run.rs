//! Doing what the menu's entries say, and telling a new connection what dev
//! tools were on.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, DevMobTool, ServerMessage};

use super::{DebugAction, DebugMenu, OpenPanels};
use crate::common::{
    ClickMode, DebugInfo, DetectMonsters, FreezeMobs, GodMode, LightningStrike, Truesight,
};
use crate::connection::{FromServer, ToServer};
use crate::planet_view::ViewPlanet;

/// A new connection knows nothing of god mode, truesight or frozen mobs.
pub(super) fn remind_of_god_mode(
    mut from_server: MessageReader<FromServer>,
    god: Res<GodMode>,
    truesight: Res<Truesight>,
    freeze: Res<FreezeMobs>,
    mut to_server: MessageWriter<ToServer>,
) {
    for FromServer(message) in from_server.read() {
        if !matches!(message, ServerMessage::Welcome { .. }) {
            continue;
        }
        if god.0 {
            to_server.write(ToServer(ClientMessage::DevGodMode(true)));
        }
        if truesight.0 {
            to_server.write(ToServer(ClientMessage::DevTruesight(true)));
        }
        if freeze.0 {
            to_server.write(ToServer(ClientMessage::DevMobs(DevMobTool::Freeze(true))));
        }
    }
}

/// Turns one mode of clicking on or off; only one can be on.
fn toggled(mode: ClickMode, this: ClickMode) -> ClickMode {
    if mode == this {
        ClickMode::Normal
    } else {
        this
    }
}

/// Does what was chosen.
#[allow(clippy::too_many_arguments)]
pub(super) fn run_actions(
    mut actions: MessageReader<DebugAction>,
    mut commands: Commands,
    mut detect: ResMut<DetectMonsters>,
    mut god: ResMut<GodMode>,
    mut truesight: ResMut<Truesight>,
    mut freeze: ResMut<FreezeMobs>,
    mut mode: ResMut<ClickMode>,
    mut info: ResMut<DebugInfo>,
    mut open: ResMut<OpenPanels>,
    mut menu: ResMut<DebugMenu>,
    mut dom: crate::dom::Dom,
    nodes: Query<(), With<Node>>,
    (mut to_server, mut view_planet, mut lightning, mut accounts): (
        MessageWriter<ToServer>,
        MessageWriter<ViewPlanet>,
        MessageWriter<LightningStrike>,
        MessageWriter<crate::common::OpenAccounts>,
    ),
) {
    for &action in actions.read() {
        match action {
            DebugAction::Accounts => {
                accounts.write(crate::common::OpenAccounts);
            }
            DebugAction::ChangeCellMode => *mode = toggled(*mode, ClickMode::ChangeCell),
            DebugAction::DetectMobs => {
                detect.0 = !detect.0;
                to_server.write(ToServer(ClientMessage::DevDetectMonsters(detect.0)));
            }
            DebugAction::GodMode => {
                god.0 = !god.0;
                to_server.write(ToServer(ClientMessage::DevGodMode(god.0)));
            }
            DebugAction::Truesight => {
                truesight.0 = !truesight.0;
                to_server.write(ToServer(ClientMessage::DevTruesight(truesight.0)));
            }
            DebugAction::FreezeMobs => {
                freeze.0 = !freeze.0;
                let freeze = DevMobTool::Freeze(freeze.0);
                to_server.write(ToServer(ClientMessage::DevMobs(freeze)));
            }
            DebugAction::ForgetMap => {
                to_server.write(ToServer(ClientMessage::DevForgetMap));
            }
            DebugAction::RemakeSite => {
                to_server.write(ToServer(ClientMessage::DevForgetMapOnServer));
            }
            DebugAction::GeneratePoi => menu.open_submenu(),
            DebugAction::KeyForThisDoor => {
                let ask = shared::protocol::DoorAsk::DevKey;
                to_server.write(ToServer(ClientMessage::Door(ask)));
            }
            DebugAction::OpenAllDoors | DebugAction::CloseAllDoors => {
                let open = action == DebugAction::OpenAllDoors;
                let ask = shared::protocol::DoorAsk::DevAll { open };
                to_server.write(ToServer(ClientMessage::Door(ask)));
            }
            DebugAction::RevealMap => {
                to_server.write(ToServer(ClientMessage::DevRevealMap));
            }
            DebugAction::ShowDebugInfo => info.0 = !info.0,
            // Made again by `panels::kept_choices`.
            DebugAction::RegenerateDevArea => {}
            DebugAction::TeleportMode => *mode = toggled(*mode, ClickMode::Teleport),
            DebugAction::ViewPlanet => {
                view_planet.write(ViewPlanet { world: None });
            }
            DebugAction::LightningNow => {
                lightning.write(LightningStrike);
            }
            DebugAction::LightningStrikeHere => {
                to_server.write(ToServer(ClientMessage::DevLightning));
            }
            DebugAction::WorldTime | DebugAction::SunsAndMoons => {
                open.toggle(action, &mut commands, &mut dom, &nodes);
            }
            DebugAction::ChangeCreature
            | DebugAction::CreateMob
            | DebugAction::DevArea
            | DebugAction::DrawingStates
            | DebugAction::ExamineMob
            | DebugAction::GiveItem
            | DebugAction::LearnSkills
            | DebugAction::ItemPictures
            | DebugAction::ShowWidgets
            | DebugAction::TeleportToWorld
            | DebugAction::Vitals => open.open(action, &mut commands, &mut dom, &nodes),
        }
    }
}
