//! The debug menu: backtick or the warning button opens a full-screen list of
//! actions to filter and choose from. See `docs/context/client/debug_menu.md`.

mod actions;
mod bodies;
mod delete_character;
mod menu;
mod panels;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::movement::Step;
use shared::protocol::{ClientMessage, DevSkills, DevVitals, ServerMessage};

use crate::common::{
    ClickMode, CurrentDungeon, DebugInfo, DetectMonsters, GameView, GodMode, KillMob,
    ServerDevTools, Truesight,
};
use crate::connection::{FromServer, ToServer};
use crate::planet_view::ViewPlanet;
use crate::player::{PlayerSystems, TakeStep};
use crate::view3d::OrbitCamera;
use crate::widgets::WidgetSystems;
pub use actions::DebugAction;
use actions::Facts;
pub use menu::DebugMenu;
use panels::OpenPanels;

pub struct DebugMenuPlugin;

impl Plugin for DebugMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutoWalk>()
            .init_resource::<DebugMenu>()
            .init_resource::<OpenPanels>()
            .init_resource::<panels::GenerateForm>()
            .init_resource::<panels::GiveForm>()
            .init_resource::<panels::GiveTo>()
            .init_resource::<panels::ChosenChange>()
            .init_resource::<panels::PicturesForm>()
            .add_message::<DebugAction>()
            .add_message::<panels::GenerateKind>()
            .add_systems(
                Startup,
                (menu::spawn_menu, delete_character::spawn_delete_dialog),
            )
            .add_systems(
                PreUpdate,
                menu::take_input
                    .in_set(WidgetSystems::Modal)
                    .run_if(not(in_state(GameView::Connecting))),
            )
            .add_systems(
                Update,
                (
                    panels::forget_closed,
                    remind_of_god_mode,
                    run_actions,
                    delete_character::run_delete_dialog,
                    bodies::fill_panels,
                    bodies::apply_sliders,
                    panels::open_generate_panel,
                    panels::press_generate_buttons,
                    panels::update_generate_buttons,
                    panels::press_panel_buttons,
                    (
                        panels::press_time_steps,
                        panels::edit_time_box,
                        panels::press_rate_buttons,
                        panels::update_rate_buttons,
                        panels::press_go_to_world_buttons,
                        panels::update_go_to_world_button,
                    )
                        .chain(),
                    (
                        panels::start_give_panel,
                        panels::choose_give_options,
                        panels::edit_give_boxes,
                        panels::choose_give_option,
                        panels::fill_give_options,
                        panels::follow_target,
                        panels::show_give_preview,
                        panels::press_give_button,
                    )
                        .chain(),
                    (
                        panels::update_creature_buttons,
                        panels::press_creature_buttons,
                    )
                        .chain(),
                    panels::choose_pictures,
                    panels::fill_pictures,
                    panels::show_pictures,
                    panels::update_panel_buttons,
                    panels::show_typed_items,
                    panels::give_sample_icons,
                    menu::show_menu,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                // Only on the world map: in a site it would stop at a wall.
                auto_walk
                    .run_if(in_state(GameView::WorldMap))
                    .in_set(PlayerSystems::Input),
            );
    }
}

/// While on, the player walks east one step per frame.
#[derive(Resource, Default)]
pub struct AutoWalk(pub bool);

/// What the menu's entries depend on.
#[derive(SystemParam)]
struct FactSources<'w> {
    view: Res<'w, State<GameView>>,
    dungeon: Res<'w, CurrentDungeon>,
    walk: Res<'w, AutoWalk>,
    detect: Res<'w, DetectMonsters>,
    god: Res<'w, GodMode>,
    truesight: Res<'w, Truesight>,
    kill: Res<'w, KillMob>,
    mode: Res<'w, ClickMode>,
    info: Res<'w, DebugInfo>,
    server_dev_tools: Res<'w, ServerDevTools>,
    panels: Res<'w, OpenPanels>,
}

impl FactSources<'_> {
    fn facts(&self) -> Facts {
        Facts {
            view: *self.view.get(),
            has_stairs: !self.dungeon.stairs.is_empty(),
            has_fields: !self.dungeon.fields.is_empty(),
            has_herbs: self.dungeon.level_has(Cell::HerbBed),
            walking: self.walk.0,
            detecting: self.detect.0,
            god_mode: self.god.0,
            truesight: self.truesight.0,
            killing: self.kill.0,
            teleporting: *self.mode == ClickMode::Teleport,
            changing_cells: *self.mode == ClickMode::ChangeCell,
            showing_info: self.info.0,
            in_town: *self.view.get() == GameView::Dungeon && self.dungeon.is_town(),
            world_time_open: self.panels.is_open(DebugAction::WorldTime),
            suns_and_moons_open: self.panels.is_open(DebugAction::SunsAndMoons),
            dev_tools: self.server_dev_tools.0,
        }
    }
}

fn auto_walk(
    walk: Res<AutoWalk>,
    clock: Res<crate::common::ServerClock>,
    mut steps: MessageWriter<TakeStep>,
) {
    if walk.0 && !clock.paused {
        steps.write(TakeStep(Step::new(1, 0).expect("a step")));
    }
}

/// A new connection knows nothing of god mode or truesight.
fn remind_of_god_mode(
    mut from_server: MessageReader<FromServer>,
    god: Res<GodMode>,
    truesight: Res<Truesight>,
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
fn run_actions(
    mut actions: MessageReader<DebugAction>,
    mut commands: Commands,
    mut walk: ResMut<AutoWalk>,
    mut detect: ResMut<DetectMonsters>,
    mut god: ResMut<GodMode>,
    mut truesight: ResMut<Truesight>,
    mut kill: ResMut<KillMob>,
    mut mode: ResMut<ClickMode>,
    mut info: ResMut<DebugInfo>,
    mut open: ResMut<OpenPanels>,
    mut menu: ResMut<DebugMenu>,
    mut raise: MessageWriter<crate::widgets::RaisePanel>,
    nodes: Query<(), With<Node>>,
    mut cameras: Query<&mut OrbitCamera>,
    (mut to_server, mut view_planet): (MessageWriter<ToServer>, MessageWriter<ViewPlanet>),
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
            DebugAction::KillMob => kill.0 = !kill.0,
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
            DebugAction::TeleportMode => *mode = toggled(*mode, ClickMode::Teleport),
            // Asks first (`delete_character`).
            DebugAction::DeleteCharacter => {}
            DebugAction::ViewPlanet => {
                view_planet.write(ViewPlanet);
            }
            DebugAction::WorldTime | DebugAction::SunsAndMoons => {
                open.toggle(action, &mut commands, &mut raise, &nodes);
            }
            DebugAction::ChangeCreature
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
