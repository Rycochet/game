//! Opening the Generate POI panel, and what its buttons do.

use crate::widgets::UiId;
use bevy::prelude::*;
use shared::protocol::ClientMessage;
use shared::site::{Generator, MAX_RUIN};

use super::super::actions::{DebugAction, kind_name};
use super::{
    CHECK_ROW, FIRST_AT, FONT, GenerateButton, GenerateForm, GenerateKind, NEXT_AT, NewSeedButton,
    OpenPanels, RuinButton, SEED_WIDTH, SeedBox, new_seed, row, words,
};
use crate::connection::ToServer;
use crate::widgets::spawn_new_seed_button;
use crate::widgets::{
    BUTTON_PADDING, ButtonPressed, ButtonState, ButtonStyle, Pressable, TextBox, spawn_button,
    spawn_moveable_panel, spawn_text_box,
};

/// Opens the Generate POI panel for the kind chosen, in place of any open.
#[allow(clippy::too_many_arguments)]
pub(in crate::debug_menu) fn open_generate_panel(
    mut kinds: MessageReader<GenerateKind>,
    mut commands: Commands,
    mut open: ResMut<OpenPanels>,
    mut form: ResMut<GenerateForm>,
    nodes: Query<(), With<Node>>,
    time: Res<Time<Real>>,
) {
    let Some(&GenerateKind(kind)) = kinds.read().last() else {
        return;
    };
    let action = DebugAction::GeneratePoi;
    if let Some(panel) = open.panels.remove(&action)
        && nodes.contains(panel)
    {
        commands.entity(panel).despawn();
    }
    let at = FIRST_AT + NEXT_AT * (open.opened % 8) as f32;
    open.opened += 1;
    let (panel, content) = spawn_moveable_panel(&mut commands, action.name(), at);
    commands
        .entity(panel)
        .insert(UiId::new("debug.generate_poi", "panel"));
    open.panels.insert(action, panel);
    form.kind = Some(kind);
    form.ruin = 0;
    form.seeds_made += 1;
    let seed = new_seed(time.elapsed().as_nanos() as u64, form.seeds_made);
    commands.entity(content).with_children(|content| {
        content.spawn(words(kind_name(kind)));
        content.spawn(row()).with_children(|row| {
            row.spawn(words("Ruin"));
            for ruin in 0..=MAX_RUIN {
                let node = Node {
                    padding: BUTTON_PADDING,
                    ..default()
                };
                spawn_button(row, ButtonState::Normal, ruin.to_string(), FONT, node).insert((
                    RuinButton(ruin),
                    Pressable,
                    UiId::new("debug.generate_poi", "ruin").at(usize::from(ruin)),
                ));
            }
        });
        content.spawn(row()).with_children(|row| {
            row.spawn(words("Seed"));
            let text_box = TextBox {
                text: seed.to_string(),
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
                .insert((SeedBox, UiId::new("debug.generate_poi", "seed")));
            spawn_new_seed_button(row, CHECK_ROW)
                .insert((NewSeedButton, UiId::new("debug.generate_poi", "new_seed")));
        });
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, ButtonState::Normal, "Generate", FONT, node)
            .insert(ButtonStyle::Main)
            .insert((
                GenerateButton,
                Pressable,
                UiId::new("debug.generate_poi", "generate"),
            ));
    });
}

/// What the Generate POI panel's buttons do.
#[allow(clippy::too_many_arguments)]
pub(in crate::debug_menu) fn press_generate_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    ruins: Query<&RuinButton>,
    new_seed_button: Query<(), With<NewSeedButton>>,
    generate: Query<(), With<GenerateButton>>,
    mut seed_box: Query<&mut TextBox, With<SeedBox>>,
    mut form: ResMut<GenerateForm>,
    time: Res<Time<Real>>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok(ruin) = ruins.get(press.0) {
            form.ruin = ruin.0;
        }
        let Ok(mut seed) = seed_box.single_mut() else {
            continue;
        };
        if new_seed_button.contains(press.0) {
            form.seeds_made += 1;
            let new = new_seed(time.elapsed().as_nanos() as u64, form.seeds_made);
            seed.text = new.to_string();
        }
        if generate.contains(press.0)
            && let (Some(kind), Ok(seed)) = (form.kind, seed.text.parse())
        {
            to_server.write(ToServer(ClientMessage::DevGenerateSite {
                kind,
                seed,
                ruin: form.ruin,
            }));
        }
    }
}

/// Shows the ruin chosen, and greys what can't be used: ruin for kinds that have
/// none, and Generate without a seed.
pub(in crate::debug_menu) fn update_generate_buttons(
    form: Res<GenerateForm>,
    mut ruins: Query<(&RuinButton, &mut ButtonState), Without<GenerateButton>>,
    mut generate: Query<&mut ButtonState, With<GenerateButton>>,
    seed_box: Query<&TextBox, With<SeedBox>>,
) {
    let can_ruin = form.kind.is_some_and(Generator::can_be_ruined);
    for (button, mut state) in &mut ruins {
        let chosen = button.0 == form.ruin;
        state.set_if_neq(match (can_ruin, chosen) {
            (true, true) => ButtonState::Selected,
            (true, false) => ButtonState::Normal,
            (false, true) => ButtonState::DisabledSelected,
            (false, false) => ButtonState::Disabled,
        });
    }
    let has_seed = seed_box
        .single()
        .is_ok_and(|seed| seed.text.parse::<u64>().is_ok());
    for mut state in &mut generate {
        state.set_if_neq(if has_seed {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
}
