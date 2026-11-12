//! The debug menu: backtick or the warning button opens a full-screen list of
//! actions to filter and choose from. See `docs/context/client/debug_menu.md`.

mod actions;
mod bodies;
mod menu;
mod panels;
mod run;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use shared::movement::Step;

use crate::common::{
    ClickMode, CurrentDungeon, DebugInfo, DebugTools, DetectMonsters, FreezeMobs, GameView,
    GodMode, Truesight,
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
            .init_resource::<panels::TeleportForm>()
            .add_message::<DebugAction>()
            .add_message::<panels::GenerateKind>()
            .add_plugins((
                panels::add_dev_area_panel,
                panels::add_create_mob_panel,
                panels::add_examine_mob_panel,
                panels::add_learn_skills_panel,
                panels::add_kept_choices,
            ))
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
                    panels::forget_closed,
                    run::remind_of_god_mode,
                    run::run_actions,
                    (bodies::fill_panels, bodies::bind_bodies),
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
                        panels::open_teleport_asked,
                        panels::press_teleport_buttons,
                        panels::update_teleport_buttons,
                        panels::press_vitals_buttons,
                        panels::update_vitals_buttons,
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
                    (panels::fill_html_sample, panels::show_html_events),
                    (menu::bind_menu, menu::show_menu, panels::bind_panels),
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

/// While on, the player walks east one step per frame (dev tools:
/// `HOW_DEBUG_WALK` and the debug API; no longer in the menu).
#[derive(Resource, Default)]
pub struct AutoWalk(pub bool);

/// What the menu's entries depend on.
#[derive(SystemParam)]
struct FactSources<'w> {
    view: Res<'w, State<GameView>>,
    dungeon: Res<'w, CurrentDungeon>,
    detect: Res<'w, DetectMonsters>,
    god: Res<'w, GodMode>,
    truesight: Res<'w, Truesight>,
    freeze: Res<'w, FreezeMobs>,
    mode: Res<'w, ClickMode>,
    info: Res<'w, DebugInfo>,
    tools: Res<'w, DebugTools>,
    role: Res<'w, crate::common::AccountRole>,
    panels: Res<'w, OpenPanels>,
}

impl FactSources<'_> {
    fn facts(&self) -> Facts {
        Facts {
            view: *self.view.get(),
            detecting: self.detect.0,
            god_mode: self.god.0,
            truesight: self.truesight.0,
            freezing: self.freeze.0,
            teleporting: *self.mode == ClickMode::Teleport,
            changing_cells: *self.mode == ClickMode::ChangeCell,
            showing_info: self.info.0,
            in_town: *self.view.get() == GameView::Dungeon && self.dungeon.is_town(),
            world_time_open: self.panels.is_open(DebugAction::WorldTime),
            suns_and_moons_open: self.panels.is_open(DebugAction::SunsAndMoons),
            tools: *self.tools,
            accounts: self.role.has_accounts_panel(),
        }
    }

    /// Whether there is a menu at all: only with a debug tool, or the Admin
    /// Tools' Accounts (owner, 2026-11-09 and 2026-11-10).
    fn any_entry(&self) -> bool {
        self.tools.any() || self.role.has_accounts_panel()
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
