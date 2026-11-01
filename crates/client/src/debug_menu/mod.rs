//! The debug menu: backtick or the warning button opens a full-screen list of
//! actions to filter and choose from. See `docs/context/client/debug_menu.md`.

mod actions;
mod menu;
mod panels;
mod soft_keyboard;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::movement::Step;
use shared::protocol::ClientMessage;

use crate::common::{
    ClickMode, CurrentDungeon, DebugInfo, GameView, NORMAL_SNOW_LINE, ServerDevTools, Weather,
};
use crate::connection::ToServer;
use crate::player::{PlayerSystems, TakeStep};
use crate::view3d::OrbitCamera;
use crate::widgets::WidgetSystems;
pub use actions::DebugAction;
use actions::Facts;
pub use menu::DebugMenu;
use panels::OpenPanels;

/// How fast the snow line moves during the temperature cycle, in height units
/// per second (the prototype moved 1 unit every 166 ms).
const SNOW_LINE_SPEED: f64 = 6.0;

pub struct DebugMenuPlugin;

impl Plugin for DebugMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutoWalk>()
            .init_resource::<TemperatureCycle>()
            .init_resource::<DebugMenu>()
            .init_resource::<OpenPanels>()
            .add_message::<DebugAction>()
            .add_systems(Startup, menu::spawn_menu)
            .add_systems(
                PreUpdate,
                menu::take_input
                    .in_set(WidgetSystems::Modal)
                    .run_if(not(in_state(GameView::Connecting))),
            )
            .add_systems(
                Update,
                (
                    run_actions,
                    panels::press_panel_buttons,
                    panels::update_panel_buttons,
                    temperature_cycle,
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

/// Where the temperature cycle is in its cycle.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemperatureCycle {
    #[default]
    Off,
    Cooling,
    Warming,
}

/// What the menu's entries depend on.
#[derive(SystemParam)]
struct FactSources<'w> {
    view: Res<'w, State<GameView>>,
    dungeon: Res<'w, CurrentDungeon>,
    walk: Res<'w, AutoWalk>,
    cycle: Res<'w, TemperatureCycle>,
    mode: Res<'w, ClickMode>,
    info: Res<'w, DebugInfo>,
    server_dev_tools: Res<'w, ServerDevTools>,
}

impl FactSources<'_> {
    fn facts(&self) -> Facts {
        Facts {
            view: *self.view.get(),
            has_stairs: !self.dungeon.stairs.is_empty(),
            has_fields: !self.dungeon.fields.is_empty(),
            has_herbs: self.dungeon.level_has(Cell::HerbBed),
            walking: self.walk.0,
            cycling: *self.cycle != TemperatureCycle::Off,
            teleporting: *self.mode == ClickMode::Teleport,
            changing_cells: *self.mode == ClickMode::ChangeCell,
            showing_info: self.info.0,
            in_town: *self.view.get() == GameView::Dungeon && self.dungeon.is_town(),
            dev_tools: self.server_dev_tools.0,
        }
    }
}

fn auto_walk(walk: Res<AutoWalk>, mut steps: MessageWriter<TakeStep>) {
    if walk.0 {
        steps.write(TakeStep(Step::new(1, 0).expect("a step")));
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
    mut cycle: ResMut<TemperatureCycle>,
    mut mode: ResMut<ClickMode>,
    mut info: ResMut<DebugInfo>,
    mut open: ResMut<OpenPanels>,
    mut raise: MessageWriter<crate::widgets::RaisePanel>,
    nodes: Query<(), With<Node>>,
    mut cameras: Query<&mut OrbitCamera>,
    mut to_server: MessageWriter<ToServer>,
) {
    for &action in actions.read() {
        match action {
            DebugAction::AutoWalk => walk.0 = !walk.0,
            DebugAction::BackToStart => {
                to_server.write(ToServer(ClientMessage::DevBackToStart));
            }
            DebugAction::ChangeCellMode => *mode = toggled(*mode, ClickMode::ChangeCell),
            DebugAction::CycleTemperature => {
                *cycle = match *cycle {
                    TemperatureCycle::Off => TemperatureCycle::Cooling,
                    // Off at once: a slow warming would leave the box ticked.
                    _ => {
                        to_server.write(ToServer(ClientMessage::DevSnowLine(
                            NORMAL_SNOW_LINE as u32,
                        )));
                        TemperatureCycle::Off
                    }
                };
            }
            DebugAction::ForgetMap => {
                to_server.write(ToServer(ClientMessage::DevForgetMap));
            }
            DebugAction::RevealMap => {
                to_server.write(ToServer(ClientMessage::DevRevealMap));
            }
            DebugAction::PoiShowcase => {
                to_server.write(ToServer(ClientMessage::DevToShowcase));
            }
            DebugAction::ResetView => {
                for mut camera in &mut cameras {
                    camera.reset();
                }
            }
            DebugAction::ShowDebugInfo => info.0 = !info.0,
            DebugAction::TeleportMode => *mode = toggled(*mode, ClickMode::Teleport),
            DebugAction::FieldsState
            | DebugAction::HerbsState
            | DebugAction::ShowWidgets
            | DebugAction::StairsStyle => open.open(action, &mut commands, &mut raise, &nodes),
        }
    }
}

/// Moves the snow line by asking the server, whose word `Weather` holds, as it
/// decides where ice can be walked on (dev tools: only the embedded server
/// allows it). `line` keeps the fractions between whole heights.
fn temperature_cycle(
    time: Res<Time>,
    mut cycle: ResMut<TemperatureCycle>,
    weather: Res<Weather>,
    mut line: Local<f64>,
    mut to_server: MessageWriter<ToServer>,
) {
    let step = SNOW_LINE_SPEED * time.delta_secs_f64();
    match *cycle {
        TemperatureCycle::Off => {
            *line = weather.snow_line;
            return;
        }
        TemperatureCycle::Cooling => {
            *line = (*line - step).max(0.0);
            if *line == 0.0 {
                *cycle = TemperatureCycle::Warming;
            }
        }
        TemperatureCycle::Warming => {
            *line = (*line + step).min(NORMAL_SNOW_LINE);
            if *line == NORMAL_SNOW_LINE {
                *cycle = TemperatureCycle::Off;
            }
        }
    }
    let whole = line.round();
    if whole != weather.snow_line {
        to_server.write(ToServer(ClientMessage::DevSnowLine(whole as u32)));
    }
}
