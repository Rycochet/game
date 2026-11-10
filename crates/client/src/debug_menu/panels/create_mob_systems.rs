//! What the Create Mob panel's widgets do, and keeping Mob Mode to where it can
//! be used.

use bevy::prelude::*;
use shared::creature::CreatureKind;

use super::create_mob::ATTITUDES;
use super::{
    MobChoice, MobClickToPlace, MobCreate, MobNewSeed, MobSeed, MobStill, OpenPanels, new_seed,
};
use crate::common::{ClickMode, CreateMobOrder, CurrentDungeon, DebugTools, GameView};
use crate::debug_menu::DebugAction;
use crate::widgets::{
    ButtonPressed, ButtonState, Checked, Dropdown, DropdownChosen, TextBox, TextEdited,
};
use shared::role::DebugTool;

/// How many seeds the new seed button has made, to keep them apart.
#[derive(Resource, Default)]
struct SeedsMade(i64);

/// Adds the panel's systems to the debug menu's plugin.
pub(in crate::debug_menu) fn add_create_mob_panel(app: &mut App) {
    app.init_resource::<SeedsMade>().add_systems(
        Update,
        (
            start_create_mob_panel,
            choose_mob_options,
            press_mob_buttons,
            keep_mob_mode_where_it_works,
            show_mob_mode,
        )
            .chain(),
    );
}

/// Whether Mob Mode can be used: in a site that isn't the wilderness or the
/// Hall, with permission to make mobs.
fn can_place(view: &GameView, dungeon: &CurrentDungeon, dev_tools: bool) -> bool {
    use shared::protocol::LevelKind;
    let in_site = dungeon
        .info
        .as_ref()
        .is_some_and(|info| matches!(info.kind, LevelKind::Poi(_)));
    *view == GameView::Dungeon && in_site && dev_tools
}

/// A panel opening shows the order kept, with a new seed the first time if
/// none was kept.
fn start_create_mob_panel(
    mut seeds: Query<&mut TextBox, Added<MobSeed>>,
    mut dropdowns: Query<(&MobChoice, &mut Dropdown), Added<MobChoice>>,
    mut still: Query<&mut Checked, Added<MobStill>>,
    mut order: ResMut<CreateMobOrder>,
    mut made: ResMut<SeedsMade>,
    preferences: Res<crate::common::Preferences>,
    time: Res<Time<Real>>,
) {
    for mut text_box in &mut seeds {
        // A new seed the first time, unless one was kept from last time.
        if made.0 == 0 && preferences.create_mob.is_none() {
            made.0 += 1;
            order.0.seed = new_seed(time.elapsed().as_nanos() as u64, made.0);
        }
        text_box.text = order.0.seed.to_string();
    }
    for (choice, mut dropdown) in &mut dropdowns {
        dropdown.selected = match choice {
            MobChoice::Kind => CreatureKind::ALL
                .iter()
                .position(|&kind| kind == order.0.kind)
                .unwrap_or(0),
            MobChoice::Attitude => ATTITUDES
                .iter()
                .position(|&(_, attitude)| attitude == order.0.attitude)
                .unwrap_or(0),
            MobChoice::Gear => usize::from(!order.0.gear),
        };
    }
    for mut checked in &mut still {
        checked.0 = order.0.still;
    }
}

fn choose_mob_options(
    mut chosen: MessageReader<DropdownChosen>,
    choices: Query<&MobChoice>,
    mut edited: MessageReader<TextEdited>,
    mut seeds: Query<&mut TextBox, With<MobSeed>>,
    mut order: ResMut<CreateMobOrder>,
) {
    for &DropdownChosen(dropdown, index) in chosen.read() {
        match choices.get(dropdown) {
            Ok(MobChoice::Kind) => order.0.kind = CreatureKind::ALL[index],
            Ok(MobChoice::Attitude) => order.0.attitude = ATTITUDES[index].1,
            Ok(MobChoice::Gear) => order.0.gear = index == 0,
            Err(_) => {}
        }
    }
    for &TextEdited(entity) in edited.read() {
        if let Ok(mut seed) = seeds.get_mut(entity) {
            let read = seed.text.parse().ok();
            seed.error = read.is_none();
            if let Some(read) = read {
                order.0.seed = read;
            }
        }
    }
}

/// Still, the new seed button, and Create, which turns Mob Mode on and off.
#[allow(clippy::too_many_arguments)]
fn press_mob_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    mut still: Query<&mut Checked, With<MobStill>>,
    new_seeds: Query<(), With<MobNewSeed>>,
    create: Query<&ButtonState, With<MobCreate>>,
    mut seeds: Query<&mut TextBox, With<MobSeed>>,
    mut order: ResMut<CreateMobOrder>,
    mut made: ResMut<SeedsMade>,
    mut mode: ResMut<ClickMode>,
    time: Res<Time<Real>>,
) {
    for press in pressed.read() {
        if let Ok(mut checked) = still.get_mut(press.0) {
            order.0.still = !order.0.still;
            checked.0 = order.0.still;
        }
        if new_seeds.contains(press.0) {
            made.0 += 1;
            order.0.seed = new_seed(time.elapsed().as_nanos() as u64, made.0);
            for mut seed in &mut seeds {
                seed.text = order.0.seed.to_string();
                seed.error = false;
            }
        }
        if let Ok(state) = create.get(press.0)
            && *state != ButtonState::Disabled
        {
            *mode = if *mode == ClickMode::PlaceMob {
                ClickMode::Normal
            } else {
                ClickMode::PlaceMob
            };
        }
    }
}

/// Mob Mode goes off with the panel, or anywhere it can't be used.
fn keep_mob_mode_where_it_works(
    open: Res<OpenPanels>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    tools: Res<DebugTools>,
    mut mode: ResMut<ClickMode>,
) {
    let usable = can_place(view.get(), &dungeon, tools.may(DebugTool::CreateMob));
    if *mode == ClickMode::PlaceMob && (!open.is_open(DebugAction::CreateMob) || !usable) {
        *mode = ClickMode::Normal;
    }
}

/// Create shows Mob Mode as on, greyed where it can't be used, with "Click to
/// place" beside it while on.
fn show_mob_mode(
    mode: Res<ClickMode>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    tools: Res<DebugTools>,
    mut create: Query<&mut ButtonState, With<MobCreate>>,
    mut words: Query<&mut Visibility, With<MobClickToPlace>>,
) {
    let on = *mode == ClickMode::PlaceMob;
    let state = match (
        can_place(view.get(), &dungeon, tools.may(DebugTool::CreateMob)),
        on,
    ) {
        (false, _) => ButtonState::Disabled,
        (true, true) => ButtonState::Selected,
        (true, false) => ButtonState::Normal,
    };
    for mut button in &mut create {
        button.set_if_neq(state);
    }
    let shown = if on {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut words {
        visibility.set_if_neq(shown);
    }
}
