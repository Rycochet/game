//! Targeting by pointing: a right click, or a long press of a finger, on a
//! creature targets it; on nothing at all, it clears the target.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use shared::id::Id;

use super::choose::{Pointing, pointing};
use crate::common::{
    HoveredBag, HoveredInSky, HoveredOnMap, Other, PointedAtMap, PointerPosition, Target,
    TouchGesture,
};
use crate::dom::Hit;
use crate::view3d::HoveredToken;

/// How far the pointer may move between pressing and letting go of the right
/// button for it to count as a click rather than turning the camera, in pixels.
/// (A right press is to open a menu of choices later, targeting among them.)
const CLICK_SLOP: f32 = 5.0;

/// How long a finger stays still to target, or clear the target, in seconds.
const LONG_PRESS: f32 = 0.5;

/// What is under the pointer, as far as targeting goes. Anything that takes
/// right presses for itself (the action bar, a held item, the maps) has
/// taken them by now, in `PreUpdate`.
#[derive(SystemParam)]
pub(super) struct UnderPointer<'w, 's> {
    hovered: Res<'w, HoveredToken>,
    others: Query<'w, 's, &'static Other>,
    interface: Res<'w, Hit>,
    bag: Res<'w, HoveredBag>,
    place: Res<'w, HoveredOnMap>,
    sky: Res<'w, HoveredInSky>,
}

impl UnderPointer<'_, '_> {
    /// The creature under the pointer, if any.
    fn creature(&self) -> Option<Id> {
        let token = self.hovered.0?;
        self.others.get(token).ok().map(|other| other.id)
    }

    /// Whether anything else on the map that answers the pointer is under it.
    fn something(&self) -> bool {
        self.hovered.0.is_some()
            || self.bag.0.is_some()
            || self.place.0.is_some()
            || self.sky.0.is_some()
    }

    /// What pointing here does.
    fn pointing(&self) -> Pointing {
        if self.interface.target.is_some() {
            return Pointing::Keep;
        }
        pointing(self.creature(), self.something())
    }
}

/// Points at what is under the pointer: `true` if the target changed or was
/// cleared (so the press is used up).
fn point(under: &UnderPointer, target: &mut ResMut<Target>) -> bool {
    match under.pointing() {
        Pointing::Target(id) => {
            target.set_if_neq(Target(Some(id)));
            true
        }
        Pointing::Clear => {
            target.set_if_neq(Target(None));
            true
        }
        Pointing::Keep => false,
    }
}

/// A right click on a creature targets it, and on nothing clears the target.
/// Anywhere on the map it turns the player there (`PointedAtMap`: where a
/// menu of choices will open). Pressing and dragging turns the camera instead.
pub(super) fn pick_by_right_click(
    mouse: Res<ButtonInput<MouseButton>>,
    pointer: Res<PointerPosition>,
    under: UnderPointer,
    mut pressed_at: Local<Option<Vec2>>,
    mut target: ResMut<Target>,
    mut pointed: MessageWriter<PointedAtMap>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        // A press on the interface isn't a press on the map behind it.
        *pressed_at = pointer.0.filter(|_| under.interface.target.is_none());
    }
    if !mouse.just_released(MouseButton::Right) {
        return;
    }
    let still = match (pressed_at.take(), pointer.0) {
        (Some(from), Some(to)) => from.distance(to) <= CLICK_SLOP,
        _ => false,
    };
    if still {
        point(&under, &mut target);
        if let Some(at) = pointer.0.filter(|_| under.interface.target.is_none()) {
            pointed.write(PointedAtMap(at));
        }
    }
}

/// How a finger held on the map is going.
#[derive(Default)]
pub(super) struct Press {
    /// How long it has been still.
    held: f32,
    /// It targeted someone, or cleared the target, so lifting it isn't a tap
    /// on the map.
    targeted: bool,
}

/// A finger held still for `LONG_PRESS` on a creature targets it, and on
/// nothing clears the target; the tap its lifting makes is taken away
/// (`eat_tap`), so it doesn't walk there.
pub(super) fn pick_by_long_press(
    time: Res<Time>,
    gesture: Res<TouchGesture>,
    under: UnderPointer,
    mut press: ResMut<LongPress>,
    mut target: ResMut<Target>,
    mut pointed: MessageWriter<PointedAtMap>,
) {
    let press = &mut press.0;
    if gesture.still_at.is_none() {
        press.held = 0.0;
        return;
    }
    let before = press.held;
    press.held += time.delta_secs();
    if before >= LONG_PRESS || press.held < LONG_PRESS {
        return;
    }
    if point(&under, &mut target) {
        press.targeted = true;
    }
    if let Some(at) = gesture
        .still_at
        .filter(|_| under.interface.target.is_none())
    {
        pointed.write(PointedAtMap(at));
        press.targeted = true;
    }
}

/// The finger being held, kept between `pick_by_long_press` and `eat_tap`.
#[derive(Resource, Default)]
pub(super) struct LongPress(pub(super) Press);

/// Lifting a finger that targeted someone isn't a tap on the map. Runs before
/// anything reads clicks.
pub(super) fn eat_tap(
    gesture: Res<TouchGesture>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut press: ResMut<LongPress>,
) {
    let press = &mut press.0;
    if gesture.tap && press.targeted {
        mouse.clear_just_pressed(MouseButton::Left);
        press.targeted = false;
    } else if gesture.first_finger.is_none() {
        press.targeted = false;
    }
}
