//! Buttons that count as pressed only while they are held: by the mouse button
//! that went down on them while the pointer stays on them, or by the finger
//! that landed on them while it stays on them. Sliding off or letting go lets
//! them go.

use bevy::prelude::*;

use super::moveable::PanelPart;
use super::{ButtonPressed, ButtonState, Pressable};
use crate::common::{TouchGesture, UiPointer, in_interface};
use crate::dom::Finder;

/// Put this on a button (from `spawn_icon_button` or `spawn_button`) for it to
/// be held rather than pressed. Becoming held also sends `ButtonPressed`, once;
/// `held()` says whether it still is. A disabled one is never held.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct HoldButton {
    pub(in crate::widgets) held: bool,
    /// Held by `client.ui.set`, whatever the pointer does.
    pub(in crate::widgets) by_api: bool,
}

impl HoldButton {
    /// Whether it is being held now.
    pub fn held(&self) -> bool {
        self.held
    }
}

/// Which hold button the mouse button and the finger went down on.
#[derive(Resource, Default)]
pub(super) struct Holding {
    mouse: Option<Entity>,
    finger: Option<Entity>,
    finger_down: bool,
}

pub(super) fn add(app: &mut App) {
    app.init_resource::<Holding>().add_systems(
        PreUpdate,
        hold_buttons
            .in_set(crate::dom::DomInput::Presses)
            .after(super::feedback::track_pointer)
            .before(super::moveable::take_presses),
    );
}

/// What a press or touch may be on: every widget, so one in front hides a
/// hold button behind it.
type Widgets<'w, 's> = Query<'w, 's, (), Or<(With<Pressable>, With<PanelPart>, With<ButtonState>)>>;

/// Notes which hold buttons are held this frame. Before the press is taken
/// (a press on one stops at its panel, as any press on a panel does).
pub(super) fn hold_buttons(
    (mouse, pointer): (Res<ButtonInput<MouseButton>>, Res<UiPointer>),
    (gesture, scale): (Res<TouchGesture>, Res<UiScale>),
    (finder, widgets): (Finder, Widgets),
    mut buttons: Query<(Entity, &mut HoldButton, &ButtonState)>,
    mut holding: ResMut<Holding>,
    mut pressed: MessageWriter<ButtonPressed>,
) {
    // The nearest widget to what is in front there (`dom`'s one search).
    let front_at = |at: Vec2| {
        let chain = finder.chain_at(at);
        chain.into_iter().find(|&entity| widgets.contains(entity))
    };
    let usable = |entity: Entity| {
        buttons
            .get(entity)
            .is_ok_and(|(_, _, state)| !state.is_disabled())
    };
    // A finger's tap also arrives as a press when it lifts: the finger held it.
    if mouse.just_pressed(MouseButton::Left) && !gesture.tap {
        holding.mouse = pointer.0.and_then(front_at).filter(|&e| usable(e));
    }
    if !mouse.pressed(MouseButton::Left) {
        holding.mouse = None;
    }
    let finger = gesture.first_finger.map(|at| in_interface(at, &scale));
    match finger {
        Some(at) if !holding.finger_down => {
            holding.finger_down = true;
            holding.finger = front_at(at).filter(|&e| usable(e));
        }
        Some(_) => {}
        None => {
            holding.finger_down = false;
            holding.finger = None;
        }
    }
    let under_mouse = pointer.0.and_then(front_at);
    let under_finger = finger.and_then(front_at);
    for (entity, mut button, state) in &mut buttons {
        let by_mouse = holding.mouse == Some(entity) && under_mouse == Some(entity);
        let by_finger = holding.finger == Some(entity) && under_finger == Some(entity);
        let held = !state.is_disabled() && (button.by_api || by_mouse || by_finger);
        if held && !button.held {
            pressed.write(ButtonPressed(entity));
        }
        if button.held != held {
            button.held = held;
        }
    }
}

#[cfg(test)]
mod tests;
