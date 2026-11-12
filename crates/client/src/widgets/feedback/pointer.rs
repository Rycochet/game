//! What the pointer is on, and spreading ripples.

use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex, UiGlobalTransform};

use super::super::item::logical_rect;
use super::super::moveable::PanelPart;
use super::super::theme::RIPPLE;
use super::super::{BUTTON_BORDER, ButtonState, Pressable, Switch};
use super::{Pointed, RIPPLE_SECONDS, Ripple, ripple_shape};
use crate::dom::Hit;

/// Notes what the pointer is on, and starts a ripple on a press. Runs before
/// anything takes the press.
#[allow(clippy::type_complexity)]
pub(in crate::widgets) fn track_pointer(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mouse: Res<ButtonInput<MouseButton>>,
    hit: Res<Hit>,
    widgets: Query<
        (
            Entity,
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Option<&CalculatedClip>,
            Option<&ButtonState>,
            Has<Switch>,
        ),
        Or<(With<Pressable>, With<PanelPart>, With<ButtonState>)>,
    >,
    mut pointed: ResMut<Pointed>,
) {
    // The nearest widget to what is in front (`dom::Hit`): the thing itself, or
    // what holds it, so nothing behind something else is ever pointed at.
    let front = hit.first(|entity| widgets.get(entity).ok());
    let usable = front.filter(|(.., state, _)| !state.is_some_and(|state| state.is_disabled()));
    let under = usable.map(|(entity, ..)| entity);
    let pressing = mouse.just_pressed(MouseButton::Left);
    if pressing {
        pointed.pressed = under;
    } else if !mouse.pressed(MouseButton::Left) {
        pointed.pressed = None;
    }
    // Nothing else lights up while something is being dragged about.
    let held_elsewhere = mouse.pressed(MouseButton::Left) && pointed.pressed != under;
    let hovered = under.filter(|_| !held_elsewhere);
    if pointed.hovered != hovered {
        pointed.hovered = hovered;
        pointed.since = time.elapsed_secs();
    }
    if !pressing {
        return;
    }
    // A switch's thumb slides instead.
    let (Some(at), Some((button, _, node, transform, _, _, Some(_), false))) = (hit.at, usable)
    else {
        return;
    };
    let (corner, size) = logical_rect(node, transform);
    commands.entity(button).with_children(|inside| {
        inside
            .spawn((Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                overflow: Overflow::clip(),
                ..default()
            },))
            .with_child((
                Ripple {
                    age: 0.0,
                    centre: at - corner - Vec2::splat(BUTTON_BORDER),
                    // From a corner, it still reaches the far one.
                    reach: 2.0 * size.length(),
                },
                Node {
                    position_type: PositionType::Absolute,
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));
    });
}

/// Spreads ripples, and removes them when they have faded.
pub(in crate::widgets) fn spread_ripples(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut ripples: Query<(&mut Ripple, &mut Node, &mut BackgroundColor, &ChildOf)>,
) {
    for (mut ripple, mut node, mut colour, holder) in &mut ripples {
        ripple.age += time.delta_secs();
        if ripple.age >= RIPPLE_SECONDS {
            commands.entity(holder.parent()).despawn();
            continue;
        }
        let (across, strength) = ripple_shape(ripple.age, ripple.reach);
        node.left = px(ripple.centre.x - across / 2.0);
        node.top = px(ripple.centre.y - across / 2.0);
        node.width = px(across);
        node.height = px(across);
        colour.0 = RIPPLE.with_alpha(strength);
    }
}
