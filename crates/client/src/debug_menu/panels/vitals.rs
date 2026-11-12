//! The Vitals panel: Hurt, Spend Mana, Heal Fully and Bonus, each asking the
//! server to change the character's Health and Mana (`DevVitals`); the vitals
//! circle shows the answer.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, DevVitals};

use crate::common::DebugTools;
use crate::connection::ToServer;
use crate::dom::Template;
use crate::html;
use crate::widgets::{ButtonPressed, ButtonState, UiId};
use shared::role::DebugTool;

/// How much Hurt takes and Spend Mana spends (the buttons' words say so too).
const STEP: u32 = 5;

/// A button sending this to the server.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct VitalsButton(DevVitals);

/// The panel's contents, as markup.
pub(super) static VITALS: Template = html!(
    "debug_menu.vitals",
    r#"
<form id="debug.vitals">
  <panel id="panel" moveable>
    <span nowrap>
      <button id="hurt">Hurt 5</button>
      <button id="spend_mana">Spend 5 Mana</button>
      <button id="heal_fully">Heal Fully</button>
      <button id="bonus">Bonus</button>
    </span>
  </panel>
</form>
"#
);

/// What a built button sends, from its id.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId) {
    let vitals = match id.parts().1 {
        "hurt" => DevVitals::Hurt(STEP),
        "spend_mana" => DevVitals::SpendMana(STEP),
        "heal_fully" => DevVitals::Fill,
        "bonus" => DevVitals::Bonus,
        _ => return,
    };
    bound.insert(VitalsButton(vitals));
}

/// Sends what the pressed button says.
pub(in crate::debug_menu) fn press_vitals_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&VitalsButton>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok(button) = buttons.get(press.0) {
            to_server.write(ToServer(ClientMessage::DevVitals(button.0)));
        }
    }
}

/// Greys the buttons without permission (the panel can be open from before).
pub(in crate::debug_menu) fn update_vitals_buttons(
    tools: Res<DebugTools>,
    mut buttons: Query<&mut ButtonState, With<VitalsButton>>,
) {
    let state = if tools.may(DebugTool::Vitals) {
        ButtonState::Normal
    } else {
        ButtonState::Disabled
    };
    for mut button in &mut buttons {
        button.set_if_neq(state);
    }
}
