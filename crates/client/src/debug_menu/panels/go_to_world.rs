//! The Go to World panel: a seed (typed, or a random one) and a button that
//! asks the server to move the player to that seed's world.

use crate::widgets::UiId;
use bevy::prelude::*;
use shared::protocol::ClientMessage;

use super::{CHECK_ROW, FONT, GenerateForm, SEED_WIDTH, new_seed, row, words};
use crate::connection::ToServer;
use crate::widgets::{
    BUTTON_PADDING, ButtonPressed, ButtonState, ButtonStyle, Pressable, TextBox, spawn_button,
    spawn_new_seed_button, spawn_text_box,
};

/// The box with the seed of the world to go to.
#[derive(Component)]
pub(in crate::debug_menu) struct WorldSeedBox;

/// Fills the box with a random seed.
#[derive(Component)]
pub(in crate::debug_menu) struct RandomWorldButton;

/// Asks to go to the world in the box.
#[derive(Component)]
pub(in crate::debug_menu) struct GoToWorldButton;

pub(super) fn spawn_go_to_world(content: &mut ChildSpawnerCommands) {
    content.spawn(row()).with_children(|row| {
        row.spawn(words("Seed"));
        let text_box = TextBox {
            digits_only: true,
            takes_keys: true,
            ..default()
        };
        let node = Node {
            width: px(SEED_WIDTH),
            height: px(CHECK_ROW),
            ..default()
        };
        spawn_text_box(row, text_box, FONT, node)
            .insert((WorldSeedBox, UiId::new("debug.go_to_world", "seed")));
        spawn_new_seed_button(row, CHECK_ROW).insert((
            RandomWorldButton,
            UiId::new("debug.go_to_world", "new_seed"),
        ));
    });
    let node = Node {
        padding: BUTTON_PADDING,
        ..default()
    };
    spawn_button(content, ButtonState::Normal, "Go", FONT, node)
        .insert(ButtonStyle::Main)
        .insert((
            GoToWorldButton,
            Pressable,
            UiId::new("debug.go_to_world", "go"),
        ));
}

/// What the Go to World panel's buttons do.
pub(in crate::debug_menu) fn press_go_to_world_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    random: Query<(), With<RandomWorldButton>>,
    go: Query<(), With<GoToWorldButton>>,
    mut seed_box: Query<&mut TextBox, With<WorldSeedBox>>,
    // Its count of seeds made keeps random ones apart.
    mut form: ResMut<GenerateForm>,
    time: Res<Time<Real>>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        let Ok(mut seed) = seed_box.single_mut() else {
            continue;
        };
        if random.contains(press.0) {
            form.seeds_made += 1;
            let new = new_seed(time.elapsed().as_nanos() as u64, form.seeds_made);
            seed.text = new.to_string();
        }
        if go.contains(press.0)
            && let Ok(seed) = seed.text.parse()
        {
            to_server.write(ToServer(ClientMessage::DevGoToWorld { seed }));
        }
    }
}

/// Greys Go while there is no seed to go to.
pub(in crate::debug_menu) fn update_go_to_world_button(
    seed_box: Query<&TextBox, With<WorldSeedBox>>,
    mut go: Query<&mut ButtonState, With<GoToWorldButton>>,
) {
    let has_seed = seed_box
        .single()
        .is_ok_and(|seed| seed.text.parse::<u64>().is_ok());
    for mut state in &mut go {
        state.set_if_neq(if has_seed {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
}
