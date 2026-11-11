//! What the Dev Area panel's dropdowns, boxes and buttons do.

use bevy::prelude::*;
use shared::protocol::ClientMessage;
use shared::site::{DEV_AREA_SIZES, DevLayout, MOST_DEV_MOBS};

use super::{
    DevAreaChoice, DevAreaCount, DevAreaCountStep, DevAreaForm, DevAreaGenerate, DevAreaNewSeed,
    DevAreaSeed, new_seed,
};
use crate::connection::ToServer;
use crate::dom::Dom;
use crate::widgets::{ButtonPressed, ButtonState, DropdownChosen, TextBox, TextEdited};

/// Adds the panel's form and systems to the debug menu's plugin.
pub(in crate::debug_menu) fn add_dev_area_panel(app: &mut App) {
    app.init_resource::<DevAreaForm>().add_systems(
        Update,
        (
            start_dev_area_panel,
            choose_dev_area_options,
            edit_dev_area_boxes,
            update_dev_area_buttons,
            press_dev_area_generate,
        )
            .chain(),
    );
}

/// A panel opening shows the form as it was left, with a random seed if none.
fn start_dev_area_panel(
    mut seed_box: Query<&mut TextBox, (Added<DevAreaSeed>, Without<DevAreaCount>)>,
    mut counts: Query<&mut TextBox, (Added<DevAreaCount>, Without<DevAreaSeed>)>,
    dropdowns: Query<&DevAreaChoice, Added<DevAreaChoice>>,
    mut form: ResMut<DevAreaForm>,
    time: Res<Time<Real>>,
    mut dom: Dom,
) {
    for mut text_box in &mut seed_box {
        if form.seed.trim().is_empty() {
            form.seeds_made += 1;
            form.seed = new_seed(time.elapsed().as_nanos() as u64, form.seeds_made).to_string();
        }
        text_box.text = form.seed.clone();
    }
    for mut count in &mut counts {
        count.text = form.count.to_string();
    }
    for &choice in &dropdowns {
        let selected = match choice {
            DevAreaChoice::Size => DEV_AREA_SIZES.iter().position(|&s| s == form.size),
            DevAreaChoice::Layout => DevLayout::all().iter().position(|&l| l == form.layout),
            DevAreaChoice::Mobs => Some(form.mobs_index()),
        };
        // Each choice's value is its place in the list.
        let id = format!("debug.dev_area/{}", choice.ui_name());
        let options = choice.options().into_iter().enumerate();
        dom.set_options(&id, options.map(|(at, name)| (at.to_string(), name)));
        dom.set(&id, "value", &selected.unwrap_or(0).to_string());
    }
}

fn choose_dev_area_options(
    mut chosen: MessageReader<DropdownChosen>,
    choices: Query<&DevAreaChoice>,
    mut form: ResMut<DevAreaForm>,
) {
    for &DropdownChosen(dropdown, index) in chosen.read() {
        match choices.get(dropdown) {
            Ok(DevAreaChoice::Size) => form.size = DEV_AREA_SIZES[index],
            Ok(DevAreaChoice::Layout) => form.layout = DevLayout::all()[index],
            Ok(DevAreaChoice::Mobs) => form.choose_mobs(index),
            Err(_) => {}
        }
    }
}

/// Follows the count and seed boxes and the buttons beside them.
#[allow(clippy::too_many_arguments)]
fn edit_dev_area_boxes(
    mut edited: MessageReader<TextEdited>,
    mut pressed: MessageReader<ButtonPressed>,
    steps: Query<&DevAreaCountStep>,
    new_seeds: Query<(), With<DevAreaNewSeed>>,
    mut counts: Query<&mut TextBox, (With<DevAreaCount>, Without<DevAreaSeed>)>,
    mut seeds: Query<&mut TextBox, (With<DevAreaSeed>, Without<DevAreaCount>)>,
    mut form: ResMut<DevAreaForm>,
    time: Res<Time<Real>>,
) {
    for &TextEdited(entity) in edited.read() {
        if let Ok(mut count) = counts.get_mut(entity) {
            let read = count.text.parse::<u8>().ok();
            let read = read.filter(|n| (1..=MOST_DEV_MOBS).contains(n));
            count.error = read.is_none();
            if let Some(read) = read {
                form.count = read;
            }
        }
        if let Ok(mut seed) = seeds.get_mut(entity) {
            seed.error = seed.text.trim().is_empty();
            form.seed = seed.text.clone();
        }
    }
    for press in pressed.read() {
        if let Ok(&DevAreaCountStep(step)) = steps.get(press.0) {
            form.step_count(step);
            for mut count in &mut counts {
                count.text = form.count.to_string();
                count.error = false;
            }
        }
        if new_seeds.contains(press.0) {
            form.seeds_made += 1;
            form.seed = new_seed(time.elapsed().as_nanos() as u64, form.seeds_made).to_string();
            for mut seed in &mut seeds {
                seed.text = form.seed.clone();
                seed.error = false;
            }
        }
    }
}

/// The count only goes with one kind of mob, and Generate needs a seed.
fn update_dev_area_buttons(
    form: Res<DevAreaForm>,
    mut steps: Query<&mut ButtonState, (With<DevAreaCountStep>, Without<DevAreaGenerate>)>,
    mut generate: Query<&mut ButtonState, With<DevAreaGenerate>>,
) {
    let with_count = if form.has_count() {
        ButtonState::Normal
    } else {
        ButtonState::Disabled
    };
    for mut state in &mut steps {
        state.set_if_neq(with_count);
    }
    let can_make = if form.seed.trim().is_empty() {
        ButtonState::Disabled
    } else {
        ButtonState::Normal
    };
    for mut state in &mut generate {
        state.set_if_neq(can_make);
    }
}

/// Generate asks the server for the area, which puts the player in it.
fn press_dev_area_generate(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&ButtonState, With<DevAreaGenerate>>,
    form: Res<DevAreaForm>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if buttons.get(press.0) == Ok(&ButtonState::Normal) {
            to_server.write(ToServer(ClientMessage::DevGenerateArea(form.area())));
        }
    }
}
