//! Dragging and raising panels, and their close buttons.

use bevy::prelude::*;

use super::super::{ButtonPressed, ButtonState, Pressable, RaisePanel};
use super::{FIRST_PANEL_Z, MoveablePanel, PanelPart};
use crate::common::{TouchGesture, UiPointer, in_interface};
use crate::dom::{Finder, Hit, Opaque, OwnPointer};

/// What is being moved, between frames.
#[derive(Resource, Default)]
pub(in crate::widgets) struct Dragging {
    /// By the mouse: which panel.
    mouse: Option<Entity>,
    /// The last place the pointer was, while dragging by the mouse.
    from: Option<Vec2>,
    /// Where the finger now dragging landed, and the panel its title bar was
    /// there, if it was.
    touch: Option<(Vec2, Option<Entity>)>,
    /// The highest panel so far.
    top_z: i32,
}

/// What a press on a node does: its part of a panel, whether it is a surface
/// (`dom::Opaque`), and whether its owner reads the pointer itself.
type PressRules<'w, 's> = (
    Query<'w, 's, &'static PanelPart>,
    Query<'w, 's, (), With<Opaque>>,
    Query<'w, 's, (), With<OwnPointer>>,
);

/// What this takes from the mouse: a press on anything of the interface
/// (`dom::Hit` has a target), and it is gone, so nothing under it sees it,
/// unless what is nearest there reads the pointer itself (`dom::OwnPointer`). A
/// tap arrives as a press (see `touch`), so buttons work by touch; moving a
/// panel by touch comes from `TouchGesture`.
#[allow(clippy::too_many_arguments)]
pub(in crate::widgets) fn take_presses(
    mut commands: Commands,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    pointer: Res<UiPointer>,
    (hit, finder): (Res<Hit>, Finder),
    mut gesture: ResMut<TouchGesture>,
    scale: Res<UiScale>,
    (parts, surfaces, owned): PressRules,
    buttons: Query<(Entity, Option<&ButtonState>), With<Pressable>>,
    mut raised: MessageReader<RaisePanel>,
    mut panels: Query<(&mut MoveablePanel, &mut GlobalZIndex)>,
    mut pressed: MessageWriter<ButtonPressed>,
    mut dragging: ResMut<Dragging>,
) {
    let dragging = &mut *dragging;
    // What a press on this does, if it is a part of a panel; a surface
    // (`dom::Opaque`) stops presses as a panel's body does, and what reads
    // the pointer itself (`dom::OwnPointer`) leaves them for its owner.
    let part_of = |entity: Entity| {
        let part = parts.get(entity).ok().copied();
        let owners = owned.contains(entity).then_some(PanelPart::LeftForOwner);
        part.or(owners)
            .or_else(|| surfaces.contains(entity).then_some(PanelPart::Body(entity)))
    };
    let mut raise = |panel: Entity, panels: &mut Query<(&mut MoveablePanel, &mut GlobalZIndex)>| {
        if let Ok((_, mut z)) = panels.get_mut(panel) {
            dragging.top_z = dragging.top_z.max(FIRST_PANEL_Z) + 1;
            z.0 = dragging.top_z;
        }
    };
    for panel in raised.read() {
        raise(panel.0, &mut panels);
    }

    if mouse.just_pressed(MouseButton::Left) {
        // From what is in front (`dom::Hit`) outwards: the first button or
        // part of a panel is what the press is on, so a button behind a panel,
        // or behind anything else in front, gets nothing.
        let nearest = hit.first(|entity| {
            let part = part_of(entity);
            let button = buttons.get(entity).ok();
            (part.is_some() || button.is_some()).then_some((part, button))
        });
        let button = nearest.and_then(|(part, button)| button.filter(|_| part.is_none()));
        let part = hit.first(part_of);
        if let Some((entity, state)) = button {
            mouse.clear_just_pressed(MouseButton::Left);
            if !state.is_some_and(|state| state.is_disabled()) {
                pressed.write(ButtonPressed(entity));
            }
        } else if hit.target.is_some() && !matches!(part, Some(PanelPart::LeftForOwner)) {
            mouse.clear_just_pressed(MouseButton::Left);
        }
        // A press on a button in a panel is a press on the panel, but its
        // title bar and close button don't have buttons on them.
        match part {
            Some(PanelPart::Close(panel)) => {
                commands.entity(panel).despawn();
            }
            Some(PanelPart::TitleBar(panel)) => {
                raise(panel, &mut panels);
                dragging.mouse = Some(panel);
                dragging.from = pointer.0;
            }
            Some(PanelPart::Body(panel)) => raise(panel, &mut panels),
            Some(PanelPart::LeftForOwner) | None => {}
        }
    }
    if !mouse.pressed(MouseButton::Left)
        && let Some(panel) = dragging.mouse.take()
        && let Ok((mut moveable, _)) = panels.get_mut(panel)
    {
        moveable.dropped = true;
    }
    if let (Some(panel), Some(from), Some(to)) = (dragging.mouse, dragging.from, pointer.0) {
        dragging.from = Some(to);
        if let Ok((mut moveable, _)) = panels.get_mut(panel) {
            moveable.at += to - from;
        }
    }

    if gesture.drag != Vec2::ZERO {
        let start = in_interface(gesture.drag_start, &scale);
        if dragging.touch.is_none_or(|(known, _)| known != start) {
            // What the finger landed on: a title bar moves its panel, and
            // anything else of the interface takes the drag and does nothing
            // with it, unless its owner reads fingers itself.
            let chain = finder
                .target_at(start)
                .map(|target| finder.chain_of(target));
            let on_interface = chain.is_some();
            let part = chain.unwrap_or_default().into_iter().find_map(part_of);
            let taken = match part {
                Some(PanelPart::TitleBar(panel)) => Some(panel),
                Some(PanelPart::LeftForOwner) => None,
                _ => on_interface.then_some(Entity::PLACEHOLDER),
            };
            dragging.touch = Some((start, taken));
        }
        if let Some((_, Some(panel))) = dragging.touch {
            if let Ok((mut moveable, _)) = panels.get_mut(panel) {
                moveable.at += in_interface(gesture.drag, &scale);
            }
            gesture.drag = Vec2::ZERO;
        }
    }
    // The finger dragging a panel has lifted.
    if gesture.first_finger.is_none()
        && let Some((_, Some(panel))) = dragging.touch.take()
        && let Ok((mut moveable, _)) = panels.get_mut(panel)
    {
        moveable.dropped = true;
    }
}
