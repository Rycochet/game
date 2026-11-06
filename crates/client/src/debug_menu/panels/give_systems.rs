//! What Give Item's dropdowns, boxes and buttons do, and Change Creature's buttons.

use bevy::prelude::*;
use shared::creature::Size;
use shared::item::{Favour, Quality};
use shared::protocol::{ClientMessage, LevelKind};

use super::{
    CreatureButton, GiveButton, GiveChoice, GiveCount, GiveCountStep, GiveForm, GiveRandomSeed,
    GiveSeed, SetChoice, give_kinds, new_seed,
};
use crate::common::{CurrentDungeon, GameView};
use crate::connection::ToServer;
use crate::widgets::{ButtonPressed, ButtonState, DropdownChosen, TextBox, TextEdited};

/// A newly opened panel starts as `GiveForm::default`, with a random seed.
pub(in crate::debug_menu) fn start_give_panel(
    mut seed_box: Query<&mut TextBox, Added<GiveSeed>>,
    mut form: ResMut<GiveForm>,
    time: Res<Time<Real>>,
) {
    for mut text_box in &mut seed_box {
        let seeds_made = form.seeds_made + 1;
        let seed = new_seed(time.elapsed().as_nanos() as u64, seeds_made);
        *form = GiveForm {
            seed: Some(seed),
            seeds_made,
            ..default()
        };
        text_box.text = seed.to_string();
    }
}

/// Follows Give Item's dropdowns.
pub(in crate::debug_menu) fn choose_give_options(
    mut chosen: MessageReader<DropdownChosen>,
    choices: Query<&GiveChoice>,
    mut form: ResMut<GiveForm>,
) {
    for &DropdownChosen(dropdown, index) in chosen.read() {
        let Ok(choice) = choices.get(dropdown) else {
            continue;
        };
        match choice {
            GiveChoice::Size => form.size = index.checked_sub(1).map(|index| Size::ALL[index]),
            GiveChoice::Identical => form.identical = index == 1,
            GiveChoice::Quality => form.quality = Quality::ALL[index],
            GiveChoice::Set => form.set = SetChoice::ALL[index],
            GiveChoice::Favour => {
                form.favour = [None, Some(Favour::Blessed), Some(Favour::Cursed)][index];
            }
            GiveChoice::Magic => form.magical = index == 1,
            GiveChoice::Kind => form.kind = give_kinds()[index],
        }
    }
}

/// Follows the count and seed boxes, the buttons beside them, and Give.
#[allow(clippy::too_many_arguments)]
pub(in crate::debug_menu) fn edit_give_boxes(
    mut edited: MessageReader<TextEdited>,
    mut pressed: MessageReader<ButtonPressed>,
    steps: Query<&GiveCountStep>,
    random: Query<(), With<GiveRandomSeed>>,
    mut counts: Query<&mut TextBox, (With<GiveCount>, Without<GiveSeed>)>,
    mut seeds: Query<&mut TextBox, (With<GiveSeed>, Without<GiveCount>)>,
    mut form: ResMut<GiveForm>,
    time: Res<Time<Real>>,
) {
    for &TextEdited(entity) in edited.read() {
        if let Ok(mut count) = counts.get_mut(entity) {
            let read = count.text.parse::<u32>().ok().filter(|&n| n >= 1);
            count.error = read.is_none();
            if let Some(read) = read {
                // Kept to the most allowed by `show_give_preview`.
                form.count = read;
            }
        }
        if let Ok(mut seed) = seeds.get_mut(entity) {
            form.seed = seed.text.parse().ok();
            seed.error = form.seed.is_none();
        }
    }
    for press in pressed.read() {
        if let Ok(&GiveCountStep(step)) = steps.get(press.0) {
            form.count = form.count.saturating_add_signed(step).max(1);
            for mut count in &mut counts {
                count.text = form.count.to_string();
                count.error = false;
            }
        }
        if random.contains(press.0) {
            form.seeds_made += 1;
            let seed = new_seed(time.elapsed().as_nanos() as u64, form.seeds_made);
            form.seed = Some(seed);
            for mut text_box in &mut seeds {
                text_box.text = seed.to_string();
                text_box.error = false;
            }
        }
    }
}

/// Give asks the server for what the panel shows.
pub(in crate::debug_menu) fn press_give_button(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&ButtonState, With<GiveButton>>,
    form: Res<GiveForm>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        let Ok(state) = buttons.get(press.0) else {
            continue;
        };
        let (Some(seed), ButtonState::Normal) = (form.seed, state) else {
            continue;
        };
        let order = form.order();
        to_server.write(ToServer(ClientMessage::DevGiveItem {
            kind: order.kind,
            size: order.size,
            magical: order.magical,
            quality: order.quality,
            set: order.set,
            favour: order.favour,
            seed: Some(seed),
            count: form.count,
            identical: form.identical || form.count == 1,
            request: order.request,
        }));
    }
}

/// What Change Creature's buttons do.
pub(in crate::debug_menu) fn press_creature_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<(&CreatureButton, &ButtonState)>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok((&CreatureButton(kind), ButtonState::Normal)) = buttons.get(press.0) {
            to_server.write(ToServer(ClientMessage::DevSetCreature(kind)));
        }
    }
}

/// Change Creature works only where there is ground for what the new body can't
/// wear: in a site or the wilderness, not on the world map or in the Hall, as the
/// server refuses it there.
pub(in crate::debug_menu) fn update_creature_buttons(
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    mut buttons: Query<&mut ButtonState, With<CreatureButton>>,
) {
    let in_hall = dungeon
        .info
        .as_ref()
        .is_some_and(|info| matches!(info.kind, LevelKind::Hall { .. }));
    let has_ground = *view.get() == GameView::Dungeon && !in_hall;
    for mut state in &mut buttons {
        state.set_if_neq(if has_ground {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
}
