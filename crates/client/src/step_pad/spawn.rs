//! Spawning the pad, hidden until there is a map to walk on, and its toggle
//! button in the bottom-right corner.

use bevy::prelude::*;

use super::icons::{Pointing, arrow, dpad, ladder};
use super::{CORNER_GAP, FORM, GAP, TOGGLE};
use crate::common::{PadButton, Preferences};
use crate::dom::{Pictures, Template, document_in};
use crate::html;
use crate::widgets::{
    ButtonState, ButtonTooltip, HoldButton, HoverTip, PanelPlace, Pressable, UiId,
    spawn_grip_panel, spawn_icon_button,
};

/// The pad's panel.
#[derive(Component)]
pub(super) struct StepPad;

/// The button that opens and closes it.
#[derive(Component)]
pub(super) struct PadToggle;

/// The pad's buttons, in three rows round the middle like a d-pad, with Down
/// and Up either side of Step Up (owner, 2026-11-08). Gaps are a button in
/// size (`BUTTON`: a test checks).
pub(super) static PAD: Template = html!(
    "step_pad",
    r#"
<form id="step_pad">
  <div class="tight">
    <span>
      <button id="down" width="44" height="44"><img src="picture:pad_down"/></button>
      <button id="step_up" width="44" height="44"><img src="picture:pad_step_up"/></button>
      <button id="up" width="44" height="44"><img src="picture:pad_up"/></button>
    </span>
    <span>
      <button id="step_left" width="44" height="44"><img src="picture:pad_step_left"/></button>
      <div width="44" height="44"/>
      <button id="step_right" width="44" height="44"><img src="picture:pad_step_right"/></button>
    </span>
    <span>
      <div width="44" height="44"/>
      <button id="step_down" width="44" height="44"><img src="picture:pad_step_down"/></button>
      <div width="44" height="44"/>
    </span>
  </div>
</form>
"#
);

/// Each button: what it is, its name in the markup (and in its picture's name,
/// `pad_<name>`), and its first tip (`site` keeps Up's and Down's).
pub(super) const BUTTONS: [(PadButton, &str, &str); 6] = [
    (PadButton::Down, "down", "Down"),
    (PadButton::StepUp, "step_up", "Step Up"),
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
    }
}

pub(super) fn spawn_pad(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<Pictures>,
    preferences: Res<Preferences>,
) {
    let icon = images.add(dpad());
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(CORNER_GAP),
                right: px(CORNER_GAP),
                ..default()
            },
            // Above the maps, as the other corner buttons are.
            GlobalZIndex(15),
        ))
        .with_children(|holder| {
            let node = Node {
                width: px(TOGGLE),
                height: px(TOGGLE),
                display: Display::None,
                ..default()
            };
            spawn_icon_button(holder, ButtonState::Normal, icon, node).insert((
                PadToggle,
                Pressable,
                UiId::new("hud", "step_pad"),
                ButtonTooltip("Movement Pad".into()),
            ));
        });

    // Where it was left, or just above the toggle button.
    let place = preferences.step_pad.at.map_or(
        PanelPlace::BottomRight(Vec2::new(CORNER_GAP, CORNER_GAP + TOGGLE + GAP)),
        |(x, y)| PanelPlace::Share(Vec2::new(x, y)),
    );
    let (panel, content) = spawn_grip_panel(&mut commands, place);
    commands
        .entity(panel)
        .insert((StepPad, UiId::new(FORM, "panel")));
    // The buttons' pictures, by the names the markup shows them by.
    for (button, name, _) in BUTTONS {
        let picture = images.add(picture_of(button));
        pictures.name(format!("pad_{name}"), picture);
    }
    commands.spawn(document_in(&PAD, content));
}

/// Makes each of the pad's buttons, once built, what the rest of the game
/// knows it by: held to act, with its tip.
pub(super) fn bind_pad(new: Query<(Entity, &UiId), Added<UiId>>, mut commands: Commands) {
    for (entity, id) in &new {
        let (form, name) = id.parts();
        let found = BUTTONS.iter().find(|(_, known, _)| *known == name);
        if let Some(&(button, _, tip)) = found.filter(|_| form == FORM) {
            let marks = (button, HoldButton::default(), HoverTip(tip.to_string()));
            commands.entity(entity).insert(marks);
        }
    }
}

#[cfg(test)]
mod tests;
