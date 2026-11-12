//! Spawning the pad, hidden until there is a map to walk on, and its toggle
//! button in the bottom-right corner.

use bevy::prelude::*;

use super::FORM;
use super::icons::{Pointing, arrow, dpad, ladder, turn};
use crate::common::{PadButton, Preferences};
use crate::dom::{Dom, Pictures, Template, bottom_strip, document, top_strip};
use crate::html;
use crate::widgets::{ButtonTooltip, HoldButton, HoverTip, UiId};

/// The pad's panel.
#[derive(Component)]
pub(super) struct StepPad;

/// The button that opens and closes it.
#[derive(Component)]
pub(super) struct PadToggle;

/// The pad: a panel moved by its grip, just above the toggle button until it
/// is dragged (`right`, `bottom`: the corner's gap, and the toggle and the
/// gap over it; a test checks). Its buttons are in three rows round the
/// middle like a d-pad: Turn Left and Turn Right either side of Step Up, and
/// Down and Up either side of Step Down (owner, 2026-11-12). Gaps are a
/// button in size (`BUTTON`: a test checks).
pub(super) static PAD: Template = html!(
    "step_pad",
    r#"
<form id="step_pad">
  <panel id="panel" grip right="8" bottom="42" hidden>
    <div class="tight">
      <span>
        <button id="turn_left" width="44" height="44"><img src="picture:pad_turn_left"/></button>
        <button id="step_up" width="44" height="44"><img src="picture:pad_step_up"/></button>
        <button id="turn_right" width="44" height="44"><img src="picture:pad_turn_right"/></button>
      </span>
      <span>
        <button id="step_left" width="44" height="44"><img src="picture:pad_step_left"/></button>
        <div width="44" height="44"/>
        <button id="step_right" width="44" height="44"><img src="picture:pad_step_right"/></button>
      </span>
      <span>
        <button id="down" width="44" height="44"><img src="picture:pad_down"/></button>
        <button id="step_down" width="44" height="44"><img src="picture:pad_step_down"/></button>
        <button id="up" width="44" height="44"><img src="picture:pad_up"/></button>
      </span>
    </div>
  </panel>
</form>
"#
);

/// The toggle button, in the window's bottom right corner, hidden until
/// there is a map. It has no form round it: a form on a strip has no height,
/// and the pointer is never on what is outside its form.
pub(super) static TOGGLE_BUTTON: Template = html!(
    "step_pad.toggle",
    r#"
<button id="step_pad" form="hud" right="8" bottom="8" width="30" height="30" hidden>
  <img src="picture:dpad"/>
</button>
"#
);

/// Each button: what it is, its name in the markup (and in its picture's name,
/// `pad_<name>`), and its first tip (`site` keeps Up's and Down's).
pub(super) const BUTTONS: [(PadButton, &str, &str); 8] = [
    (PadButton::TurnLeft, "turn_left", "Turn Left"),
    (PadButton::StepUp, "step_up", "Step Up"),
    (PadButton::TurnRight, "turn_right", "Turn Right"),
    (PadButton::Down, "down", "Down"),
    (PadButton::Up, "up", "Up"),
    (PadButton::StepLeft, "step_left", "Step Left"),
    (PadButton::StepRight, "step_right", "Step Right"),
    (PadButton::StepDown, "step_down", "Step Down"),
];

/// The picture on one of the pad's buttons.
fn picture_of(button: PadButton) -> Image {
    match button {
        PadButton::Up => ladder(true),
        PadButton::Down => ladder(false),
        PadButton::StepUp => arrow(Pointing::Up),
        PadButton::StepDown => arrow(Pointing::Down),
        PadButton::StepLeft => arrow(Pointing::Left),
        PadButton::StepRight => arrow(Pointing::Right),
        PadButton::TurnLeft => turn(true),
        PadButton::TurnRight => turn(false),
    }
}

pub(super) fn spawn_pad(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<Pictures>,
    preferences: Res<Preferences>,
    mut dom: Dom,
) {
    // The pictures, by the names the markup shows them by.
    pictures.name("dpad", images.add(dpad()));
    for (button, name, _) in BUTTONS {
        let picture = images.add(picture_of(button));
        pictures.name(format!("pad_{name}"), picture);
    }
    // Above the maps, as the other corner buttons are.
    let toggle = (document(&TOGGLE_BUTTON), GlobalZIndex(15));
    commands.spawn(toggle).insert(bottom_strip());
    commands.spawn(document(&PAD)).insert(top_strip());
    // Where it was left: its share of the window, across and down.
    if let Some((x, y)) = preferences.step_pad.at {
        dom.set("step_pad/panel", "left", &format!("{}%", x * 100.0));
        dom.set("step_pad/panel", "top", &format!("{}%", y * 100.0));
    }
}

/// Makes each of the pad's buttons, once built, what the rest of the game
/// knows it by: held to act, with its tip. The panel and the toggle button
/// are marked too.
pub(super) fn bind_pad(new: Query<(Entity, &UiId), Added<UiId>>, mut commands: Commands) {
    for (entity, id) in &new {
        let (form, name) = id.parts();
        match (form, name) {
            (FORM, "panel") => {
                commands.entity(entity).insert(StepPad);
            }
            ("hud", "step_pad") => {
                let tip = ButtonTooltip("Movement Pad".into());
                commands.entity(entity).insert((PadToggle, tip));
            }
            _ => {}
        }
        let found = BUTTONS.iter().find(|(_, known, _)| *known == name);
        if let Some(&(button, _, tip)) = found.filter(|_| form == FORM) {
            let marks = (button, HoldButton::default(), HoverTip(tip.to_string()));
            commands.entity(entity).insert(marks);
        }
    }
}

#[cfg(test)]
mod tests;
