//! The Alignment panel: the character's hidden alignment, which only this
//! panel and the debug API show, asked after twice a second while open, with
//! buttons moving it or setting it back to neutral (`DevAlignment`).

use bevy::prelude::*;
use shared::alignment::Alignment;
use shared::protocol::{ClientMessage, DevAlignment, ServerMessage};
use shared::role::DebugTool;

use super::super::actions::DebugAction;
use super::OpenPanels;
use crate::common::DebugTools;
use crate::connection::{FromServer, ToServer};
use crate::dom::{Dom, Template};
use crate::html;
use crate::widgets::{ButtonPressed, ButtonState, UiId};

/// How often an open panel asks again, in seconds: kills change it.
const ASK_EVERY: f32 = 0.5;

/// A button sending this to the server.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct AlignmentButton(DevAlignment);

/// The panel's contents, as markup.
pub(super) static ALIGNMENT: Template = html!(
    "debug_menu.alignment",
    r#"
<form id="debug.alignment">
  <panel id="panel" moveable>
    <p id="value">Asking the server...</p>
    <span nowrap>
      <button id="less_100">-100</button>
      <button id="less_10">-10</button>
      <button id="neutral">Neutral</button>
      <button id="more_10">+10</button>
      <button id="more_100">+100</button>
    </span>
  </panel>
</form>
"#
);

/// Adds the panel's systems to the debug menu's plugin.
pub(in crate::debug_menu) fn add_alignment_panel(app: &mut App) {
    app.add_systems(
        Update,
        (ask_again, show_alignment, press_buttons, grey_buttons).chain(),
    );
}

/// What a built button sends, from its id.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId) {
    let change = match id.parts().1 {
        "less_100" => DevAlignment::Shift(-100),
        "less_10" => DevAlignment::Shift(-10),
        "neutral" => DevAlignment::Set(0),
        "more_10" => DevAlignment::Shift(10),
        "more_100" => DevAlignment::Shift(100),
        _ => return,
    };
    bound.insert(AlignmentButton(change));
}

/// While the panel is open, asks the server now and then.
fn ask_again(
    open: Res<OpenPanels>,
    time: Res<Time<Real>>,
    mut since: Local<Option<f32>>,
    mut to_server: MessageWriter<ToServer>,
) {
    if !open.is_open(DebugAction::Alignment) {
        *since = None;
        return;
    }
    let waited = since.get_or_insert(ASK_EVERY);
    *waited += time.delta_secs();
    if *waited >= ASK_EVERY {
        *waited = 0.0;
        to_server.write(ToServer(ClientMessage::DevAlignment(DevAlignment::Ask)));
    }
}

/// Shows the server's answer.
fn show_alignment(mut from_server: MessageReader<FromServer>, mut dom: Dom) {
    for FromServer(message) in from_server.read() {
        if let ServerMessage::DevAlignment(alignment) = message {
            dom.set_text("debug.alignment/value", words(*alignment));
        }
    }
}

/// How the panel says it.
fn words(alignment: Alignment) -> String {
    format!(
        "Alignment: {} ({})",
        alignment.value(),
        alignment.band().name()
    )
}

/// Sends what the pressed button says.
fn press_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&AlignmentButton>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok(button) = buttons.get(press.0) {
            to_server.write(ToServer(ClientMessage::DevAlignment(button.0)));
        }
    }
}

/// Greys the buttons without permission (the panel can be open from before).
fn grey_buttons(
    tools: Res<DebugTools>,
    mut buttons: Query<&mut ButtonState, With<AlignmentButton>>,
) {
    let state = if tools.may(DebugTool::Alignment) {
        ButtonState::Normal
    } else {
        ButtonState::Disabled
    };
    for mut button in &mut buttons {
        button.set_if_neq(state);
    }
}
