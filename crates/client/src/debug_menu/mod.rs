//! The debug menu: backtick or the warning button opens a full-screen list of
//! actions to filter and choose from. See `docs/context/client/debug_menu.md`.

mod actions;
mod bodies;
mod delete_character;
mod menu;
mod panels;
mod run;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::movement::Step;

use crate::common::{
    ClickMode, CurrentDungeon, DebugInfo, DeleteMob, DetectMonsters, FreezeMobs, GameView, GodMode,
    InspectMob, KillMob, ServerDevTools, Truesight,
};
use crate::player::{PlayerSystems, TakeStep};
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
            .add_plugins((
                panels::add_dev_area_panel,
                panels::add_create_mob_panel,
                panels::add_inspect_mob_panel,
            ))
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
                    run::remind_of_god_mode,
                    run::run_actions,
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
    inspect: Res<'w, InspectMob>,
    delete: Res<'w, DeleteMob>,
    freeze: Res<'w, FreezeMobs>,
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
            inspecting: self.inspect.0,
            deleting: self.delete.0,
            freezing: self.freeze.0,
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
