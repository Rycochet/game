//! Doing what the menu's entries say, and telling a new connection what dev
//! tools were on.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use shared::protocol::{ClientMessage, DevMobTool, DevSkills, DevVitals, ServerMessage};

use super::{AutoWalk, DebugAction, DebugMenu, OpenPanels};
use crate::common::{
    ClickMode, DebugInfo, DeleteMob, DetectMonsters, FreezeMobs, GodMode, InspectMob, KillMob,
    LightningStrike, Truesight,
};
use crate::connection::{FromServer, ToServer};
use crate::planet_view::ViewPlanet;
use crate::view3d::OrbitCamera;

/// What clicks on mobs do (only one at a time), and whether they are frozen.
#[derive(SystemParam)]
pub(super) struct MobSwitches<'w> {
    kill: ResMut<'w, KillMob>,
    inspect: ResMut<'w, InspectMob>,
    delete: ResMut<'w, DeleteMob>,
    freeze: ResMut<'w, FreezeMobs>,
}

impl MobSwitches<'_> {
    /// Turns one mob click on or off, the others off.
    fn toggle(&mut self, action: DebugAction) {
        let (kill, inspect, delete) = (self.kill.0, self.inspect.0, self.delete.0);
        self.kill.0 = action == DebugAction::KillMob && !kill;
        self.inspect.0 = action == DebugAction::InspectMob && !inspect;
        self.delete.0 = action == DebugAction::DeleteMob && !delete;
    }
}

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
    mut walk: ResMut<AutoWalk>,
    mut detect: ResMut<DetectMonsters>,
    mut god: ResMut<GodMode>,
    mut truesight: ResMut<Truesight>,
    mut mobs: MobSwitches,
    mut mode: ResMut<ClickMode>,
    mut info: ResMut<DebugInfo>,
    mut open: ResMut<OpenPanels>,
    mut menu: ResMut<DebugMenu>,
    mut raise: MessageWriter<crate::widgets::RaisePanel>,
    nodes: Query<(), With<Node>>,
    mut cameras: Query<&mut OrbitCamera>,
    (mut to_server, mut view_planet, mut lightning): (
        MessageWriter<ToServer>,
        MessageWriter<ViewPlanet>,
        MessageWriter<LightningStrike>,
    ),
) {
    for &action in actions.read() {
        match action {
            DebugAction::AutoWalk => walk.0 = !walk.0,
            DebugAction::ChangeCellMode => *mode = toggled(*mode, ClickMode::ChangeCell),
            DebugAction::DetectMonsters => {
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
            DebugAction::KillMob | DebugAction::InspectMob | DebugAction::DeleteMob => {
                mobs.toggle(action);
            }
            DebugAction::FreezeMobs => {
                mobs.freeze.0 = !mobs.freeze.0;
                let freeze = DevMobTool::Freeze(mobs.freeze.0);
                to_server.write(ToServer(ClientMessage::DevMobs(freeze)));
            }
            DebugAction::ForgetMap => {
                to_server.write(ToServer(ClientMessage::DevForgetMap));
            }
            DebugAction::ForgetMapOnServer => {
                to_server.write(ToServer(ClientMessage::DevForgetMapOnServer));
            }
            DebugAction::GeneratePoi => menu.open_submenu(),
            DebugAction::TeleportToStart => {
                to_server.write(ToServer(ClientMessage::DevBackToStart));
            }
            DebugAction::RevealMap => {
                to_server.write(ToServer(ClientMessage::DevRevealMap));
            }
            DebugAction::TeleportToHall => {
                to_server.write(ToServer(ClientMessage::DevToHall));
            }
            DebugAction::TeleportToPoiShowcase => {
                to_server.write(ToServer(ClientMessage::DevToShowcase));
            }
            DebugAction::Hurt => {
                to_server.write(ToServer(ClientMessage::DevVitals(DevVitals::Hurt(5))));
            }
            DebugAction::SpendMana => {
                to_server.write(ToServer(ClientMessage::DevVitals(DevVitals::SpendMana(5))));
            }
            DebugAction::HealFully => {
                to_server.write(ToServer(ClientMessage::DevVitals(DevVitals::Fill)));
            }
            DebugAction::VitalsBonus => {
                to_server.write(ToServer(ClientMessage::DevVitals(DevVitals::Bonus)));
            }
            DebugAction::LearnSomeSkills => {
                to_server.write(ToServer(ClientMessage::DevSkills(DevSkills::LearnSome)));
            }
            DebugAction::ForgetSkills => {
                to_server.write(ToServer(ClientMessage::DevSkills(DevSkills::Forget)));
            }
            DebugAction::ResetView => {
                for mut camera in &mut cameras {
                    camera.reset();
                }
            }
            DebugAction::ShowDebugInfo => info.0 = !info.0,
            // Made again by `panels::kept_choices`.
            DebugAction::RegenerateDevArea => {}
            DebugAction::TeleportMode => *mode = toggled(*mode, ClickMode::Teleport),
            // Asks first (`delete_character`).
            DebugAction::DeleteCharacter => {}
            DebugAction::ViewPlanet => {
                view_planet.write(ViewPlanet);
            }
            DebugAction::LightningNow => {
                lightning.write(LightningStrike);
            }
            DebugAction::LightningStrikeHere => {
                to_server.write(ToServer(ClientMessage::DevLightning));
            }
            DebugAction::WorldTime | DebugAction::SunsAndMoons => {
                open.toggle(action, &mut commands, &mut raise, &nodes);
            }
            DebugAction::ChangeCreature
            | DebugAction::CreateMob
            | DebugAction::DevArea
            | DebugAction::FieldsState
            | DebugAction::GiveItem
            | DebugAction::GoToWorld
            | DebugAction::HerbsState
            | DebugAction::ItemPictures
            | DebugAction::ShowWidgets
            | DebugAction::StairsStyle => open.open(action, &mut commands, &mut raise, &nodes),
        }
    }
}
