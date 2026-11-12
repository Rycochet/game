//! Opening the Generate POI panel, and what its buttons do.

use crate::widgets::UiId;
use bevy::prelude::*;
use shared::protocol::ClientMessage;
use shared::site::{Generator, MAX_RUIN};

use super::super::actions::{DebugAction, kind_name};
use super::{
    GenerateButton, GenerateForm, GenerateKind, NewSeedButton, OpenPanels, RuinButton, SeedBox,
    new_seed,
};
use crate::connection::ToServer;
use crate::dom::{Dom, Template};
use crate::html;
use crate::widgets::{ButtonPressed, ButtonState, TextBox};

/// Opens the Generate POI panel for the kind chosen, in place of any open.
#[allow(clippy::too_many_arguments)]
pub(in crate::debug_menu) fn open_generate_panel(
    mut kinds: MessageReader<GenerateKind>,
    mut commands: Commands,
    mut open: ResMut<OpenPanels>,
    mut form: ResMut<GenerateForm>,
    nodes: Query<(), With<Node>>,
    time: Res<Time<Real>>,
    mut dom: Dom,
) {
    let Some(&GenerateKind(kind)) = kinds.read().last() else {
        return;
    };
    // Opened again from the start, at the next place.
    let action = DebugAction::GeneratePoi;
    if let Some(document) = open.panels.remove(&action)
        && nodes.contains(document)
    {
        commands.entity(document).despawn();
    }
    open.open(action, &mut commands, &mut dom, &nodes);
    form.kind = Some(kind);
    form.ruin = 0;
    form.seeds_made += 1;
    let seed = new_seed(time.elapsed().as_nanos() as u64, form.seeds_made);
    // Filled as it is built, at the end of this frame.
    dom.set_text("debug.generate_poi/kind", kind_name(kind));
    dom.set("debug.generate_poi/seed", "value", &seed.to_string());
}

/// The panel's contents, as markup. Each amount of ruin's button is a copy
/// of the template.
pub(super) static GENERATE_POI: Template = html!(
    "debug_menu.generate_poi",
    r#"
<form id="debug.generate_poi">
  <panel id="panel" moveable>
    <p id="kind"/>
    <span nowrap valign="middle">
      Ruin
      <span id="ruins" nowrap/>
    </span>
    <span nowrap valign="middle">
      <label for="seed">Seed</label>
      <input id="seed" type="text" inputmode="numeric" width="240"/>
      <button id="new_seed" title="Random seed"><img src="picture:new_seed"/></button>
    </span>
    <button id="generate" class="main">Generate</button>
    <template id="ruin"><button id="ruin"/></template>
  </panel>
</form>
"#
);

/// What a built part of the panel is known by, from its id; the place for
/// ruin is filled with a button for each amount.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId, dom: &mut Dom) {
    match (id.parts().1, id.index()) {
        ("ruin", Some(ruin)) => bound.insert(RuinButton(ruin as u8)),
        ("seed", _) => bound.insert(SeedBox),
        ("new_seed", _) => bound.insert(NewSeedButton),
        ("generate", _) => bound.insert(GenerateButton),
        ("ruins", _) => {
            for ruin in 0..=usize::from(MAX_RUIN) {
                dom.copy_template("debug.generate_poi/ruin", "debug.generate_poi/ruins", ruin);
                dom.set_text(&format!("debug.generate_poi/ruin.{ruin}"), ruin.to_string());
            }
            bound
        }
        _ => bound,
    };
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
