//! `popup`: a dark box in front of the panels, at a place, for a choice to
//! make or for more about something. It closes by itself on a press anywhere
//! else, unless the padlock at its top right is locked: then it stays, as a
//! box of details to keep open, and can be dragged by its top (`drag`). Esc
//! closes it either way. Closing is told by its id
//! (`UiValue::Closed`); its owner then despawns its document.

mod drag;

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::DomInput;
use super::controls::{UiEvent, UiValue};
use crate::common::{PointerSystems, UiPointer};
use crate::widgets::{
    ButtonPressed, HELD_ITEM_Z, HoverTip, Pressable, UiId, logical_rect, tooltip_box,
};

/// How big the padlock is, and how far in from the corner.
const LOCK: f32 = 16.0;
const LOCK_INSET: f32 = 2.0;
const OPEN_ICON: &str = "icons/delapouite/padlock-open.png";
const LOCKED_ICON: &str = "icons/delapouite/plain-padlock.png";

/// When popups close by themselves (`PreUpdate`, before the widgets take
/// presses and the menus see Esc). An owner's system reading that one closed
/// goes `.after` this.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClosePopups;

/// On a popup's box: whether it is locked open.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Popup {
    pub locked: bool,
}

/// On a popup's padlock: the popup it locks.
#[derive(Component, Clone, Copy)]
pub(super) struct PopupLock(Entity);

/// A popup's box, with its padlock: in front of the panels, taking presses,
/// and placed by `follow_surfaces` as its element's `left` and `top` say.
pub(super) fn spawn_popup(commands: &mut Commands, assets: &AssetServer) -> Entity {
    let node = Node {
        position_type: PositionType::Absolute,
        ..default()
    };
    let front = GlobalZIndex(HELD_ITEM_Z - 1);
    let popup = commands
        .spawn((Popup::default(), tooltip_box(node), front))
        .id();
    commands.entity(popup).insert(super::Opaque);
    let corner = Node {
        position_type: PositionType::Absolute,
        top: px(LOCK_INSET),
        right: px(LOCK_INSET),
        width: px(LOCK),
        height: px(LOCK),
        ..default()
    };
    let lock = (
        PopupLock(popup),
        Pressable,
        ImageNode::new(assets.load(OPEN_ICON)),
        HoverTip("Keep this open".to_string()),
    );
    commands.spawn((lock, corner, ChildOf(popup)));
    popup
}

/// A press on a padlock locks its popup open, or lets it close again.
pub(super) fn press_locks(
    mut pressed: MessageReader<ButtonPressed>,
    mut locks: Query<(&PopupLock, &mut ImageNode, &mut HoverTip)>,
    mut popups: Query<&mut Popup>,
    assets: Res<AssetServer>,
) {
    for press in pressed.read() {
        let Ok((PopupLock(popup), mut image, mut tip)) = locks.get_mut(press.0) else {
            continue;
        };
        let Ok(mut popup) = popups.get_mut(*popup) else {
            continue;
        };
        popup.locked = !popup.locked;
        let (icon, words) = match popup.locked {
            true => (LOCKED_ICON, "Let this close"),
            false => (OPEN_ICON, "Keep this open"),
        };
        image.image = assets.load(icon);
        tip.0 = words.to_string();
    }
}

/// Closes every popup on Esc, and each that isn't locked on a press anywhere
/// but on it.
pub(super) fn close_popups(
    popups: Query<(&Popup, &UiId, &Node, &ComputedNode, &UiGlobalTransform)>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    pointer: Res<UiPointer>,
    mut events: MessageWriter<UiEvent>,
) {
    let open = popups
        .iter()
        .filter(|(_, _, node, ..)| node.display != Display::None);
    let open: Vec<_> = open.collect();
    if open.is_empty() {
        return;
    }
    // (Taken, so the menus don't open on it.)
    let escaped = keys.clear_just_pressed(KeyCode::Escape);
    let pressed = mouse.just_pressed(MouseButton::Left);
    for (popup, id, _, node, transform) in open {
        let (corner, size) = logical_rect(node, transform);
        let on_it = pointer
            .0
            .is_some_and(|at| Rect::from_corners(corner, corner + size).contains(at));
        // Locked, it stays through presses elsewhere: only Esc closes it.
        if escaped || (pressed && !on_it && !popup.locked) {
            let value = UiValue::Closed;
            events.write(UiEvent { id: *id, value });
        }
    }
}

pub(super) fn add(app: &mut App) {
    app.configure_sets(
        PreUpdate,
        ClosePopups.after(PointerSystems).before(DomInput::Presses),
    )
    // A press that takes hold of a popup isn't one elsewhere.
    .add_systems(
        PreUpdate,
        (drag::drag_popups, close_popups)
            .chain()
            .in_set(ClosePopups),
    )
    .add_systems(Update, press_locks);
}

#[cfg(test)]
mod tests;
