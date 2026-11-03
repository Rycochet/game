//! An area whose contents scroll when they are taller than it can be, by the wheel,
//! a finger, or the bar beside it.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::ui::{ComputedStackIndex, UiGlobalTransform};

use super::moveable::PanelPart;
use super::{GRID_LINE, INK};
use crate::common::{TouchGesture, UiPointer, in_interface};

const BAR_WIDTH: f32 = 10.0;
const BAR_GAP: f32 = 4.0;
/// How far one notch of the wheel goes.
const PIXELS_PER_NOTCH: f32 = 48.0;

/// What scrolls: its contents go in this.
#[derive(Component)]
pub(super) struct ScrollArea;

/// The bar beside an area.
#[derive(Component, Clone, Copy)]
pub(super) struct ScrollBar(Entity);

/// The part of a bar saying where the area is.
#[derive(Component, Clone, Copy)]
pub(super) struct ScrollThumb(Entity);

/// An area no taller than `max_height`, as tall as its contents until then. Returns
/// the entity to put its contents in. They scroll, and don't take presses where they
/// are out of sight: check `shown_at` when looking for what is under the pointer.
pub fn spawn_scroll_area(parent: &mut ChildSpawnerCommands, max_height: Val) -> Entity {
    let mut area = Entity::PLACEHOLDER;
    parent
        .spawn(Node {
            column_gap: px(BAR_GAP),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|row| {
            area = row
                .spawn((
                    ScrollArea,
                    ScrollPosition::default(),
                    Node {
                        flex_direction: FlexDirection::Column,
                        max_height,
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                ))
                .id();
            row.spawn((
                ScrollBar(area),
                Node {
                    width: px(BAR_WIDTH),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(GRID_LINE),
            ))
            .with_child((
                ScrollThumb(area),
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    ..default()
                },
                BackgroundColor(INK),
            ));
        });
    area
}

/// How far an area's contents can go: their height beyond its own.
fn furthest(node: &ComputedNode) -> f32 {
    (node.content_size.y - node.size.y).max(0.0) * node.inverse_scale_factor
}

/// Where a pointer at this share of a bar's height takes an area.
fn at_share(share: f32, furthest: f32) -> f32 {
    share.clamp(0.0, 1.0) * furthest
}

/// The wheel and fingers over an area, and the mouse on its bar. Takes them, so
/// the camera doesn't move with them.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn scroll_areas(
    pointer: Res<UiPointer>,
    ui_scale: Res<UiScale>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut wheel: ResMut<AccumulatedMouseScroll>,
    mut gesture: ResMut<TouchGesture>,
    mut areas: Query<
        (
            Entity,
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            &mut ScrollPosition,
        ),
        With<ScrollArea>,
    >,
    bars: Query<(
        &ScrollBar,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
    parts: Query<
        (
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
        ),
        With<PanelPart>,
    >,
    mut held_bar: Local<Option<Entity>>,
) {
    let on_top = |at: Vec2, stack: u32| {
        !parts.iter().any(|(other, node, transform, visible)| {
            visible.get()
                && other.0 > stack
                && node.contains_point(*transform, at / node.inverse_scale_factor)
        })
    };
    let over = |at: Vec2,
                areas: &Query<
        (
            Entity,
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            &mut ScrollPosition,
        ),
        With<ScrollArea>,
    >| {
        areas
            .iter()
            .filter(|(_, stack, node, transform, visible, _)| {
                visible.get()
                    && node.contains_point(**transform, at / node.inverse_scale_factor)
                    && on_top(at, stack.0)
            })
            .max_by_key(|(_, stack, ..)| stack.0)
            .map(|(area, ..)| area)
    };

    let mut moved = None;
    if wheel.delta.y != 0.0
        && let Some(area) = pointer.0.and_then(|at| over(at, &areas))
    {
        let pixels = match wheel.unit {
            MouseScrollUnit::Line => wheel.delta.y * PIXELS_PER_NOTCH,
            MouseScrollUnit::Pixel => wheel.delta.y,
        };
        moved = Some((area, -pixels));
        wheel.delta = Vec2::ZERO;
    } else if gesture.drag != Vec2::ZERO
        && let Some(area) = over(in_interface(gesture.drag_start, &ui_scale), &areas)
    {
        moved = Some((area, -in_interface(gesture.drag, &ui_scale).y));
        gesture.drag = Vec2::ZERO;
    }
    if let Some((area, by)) = moved
        && let Ok((.., node, _, _, mut position)) = areas.get_mut(area)
    {
        position.y = (position.y + by).clamp(0.0, furthest(node));
    }

    if !mouse.pressed(MouseButton::Left) {
        *held_bar = None;
    }
    let Some(at) = pointer.0 else {
        return;
    };
    let bar_at = |(_, node, transform, visible): &(
        &ScrollBar,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )| {
        visible.get() && node.contains_point(**transform, at / node.inverse_scale_factor)
    };
    if mouse.just_pressed(MouseButton::Left) {
        *held_bar = bars.iter().find(bar_at).map(|(bar, ..)| bar.0);
    }
    let Some(area) = *held_bar else {
        return;
    };
    let bar = bars.iter().find(|(bar, ..)| bar.0 == area);
    if let Some((_, node, transform, _)) = bar
        && let Ok((.., area_node, _, _, mut position)) = areas.get_mut(area)
    {
        let height = node.size.y * node.inverse_scale_factor;
        let top = transform.translation.y * node.inverse_scale_factor - height / 2.0;
        position.y = at_share((at.y - top) / height.max(1.0), furthest(area_node));
    }
}

/// Bars only show when there is something to scroll, and an area with less in it
/// than before comes back.
#[allow(clippy::type_complexity)]
pub(super) fn show_scroll_bars(
    mut areas: Query<(&ComputedNode, &mut ScrollPosition), With<ScrollArea>>,
    mut bars: Query<(&ScrollBar, &mut Node), Without<ScrollThumb>>,
    mut thumbs: Query<(&ScrollThumb, &mut Node), Without<ScrollBar>>,
) {
    for (bar, mut node) in &mut bars {
        let Ok((area, _)) = areas.get(bar.0) else {
            continue;
        };
        let display = if furthest(area) > 0.0 {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (thumb, mut node) in &mut thumbs {
        let Ok((area, mut position)) = areas.get_mut(thumb.0) else {
            continue;
        };
        let most = furthest(area);
        if position.y > most {
            position.y = most;
        }
        let whole = area.content_size.y.max(1.0);
        let (top, height) = (
            percent(position.y / area.inverse_scale_factor / whole * 100.0),
            percent(area.size.y / whole * 100.0),
        );
        if node.top != top || node.height != height {
            node.top = top;
            node.height = height;
        }
    }
}

#[cfg(test)]
mod tests;
