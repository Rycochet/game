//! Splitting a stack (owner's design): shift+click, or a tap, on a stack opens a box
//! with its name, a slider from one to all of it, and the stack at that many below.
//! Pressing that picks up that many from its top, closing the box; letting go goes
//! by the usual rules, and only what was taken moves. A mixed stack opens it on any
//! press, with a row for each kind in it instead (`picker`).

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use shared::id::Id;

use super::Carried;
use crate::common::{NO_QUALITY_NAME, UiPointer, quality_colour};
use crate::widgets::{
    CELL, HELD_ITEM_Z, ItemLook, SliderMoved, TextSize, logical_rect, spawn_item_cells,
    spawn_slider, stops_presses, tooltip_box,
};

/// How far from where it was opened the box goes.
const OFFSET: Vec2 = Vec2::new(16.0, 16.0);
const SLIDER_WIDTH: f32 = 128.0;
const GAP: f32 = 4.0;

/// The stack being split, if the box is open.
#[derive(Resource, Default)]
pub(super) struct Split(pub(super) Option<Splitting>);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Splitting {
    pub(super) stack: Id,
    /// How many to take from its top.
    pub(super) count: u32,
    /// Where it was opened, in the interface's pixels.
    pub(super) at: Vec2,
}

impl Splitting {
    /// Opening on a stack of `all`: half of it.
    pub(super) fn open(stack: Id, all: u32, at: Vec2) -> Self {
        Self {
            stack,
            count: (all / 2).max(1),
            at,
        }
    }
}

/// On the drawn stack's cells: which cell of it, as it lies.
#[derive(Component, Clone, Copy)]
pub(super) struct SplitCell {
    pub(super) grip: (u8, u8),
}

#[derive(Component)]
pub(super) struct SplitBox;

#[derive(Component)]
pub(super) struct SplitSlider;

/// What the stack is drawn in, to draw it again at another count.
#[derive(Component)]
pub(super) struct SplitPicture;

/// How many of `all` a slider at `value` (`0..=1`) takes: from one to all.
pub(super) fn count_at(value: f32, all: u32) -> u32 {
    let steps = all.saturating_sub(1) as f32;
    (1 + (value.clamp(0.0, 1.0) * steps).round() as u32).min(all.max(1))
}

/// Where a slider stands for taking `count` of `all`.
pub(super) fn value_of(count: u32, all: u32) -> f32 {
    match all {
        0 | 1 => 1.0,
        _ => (count.saturating_sub(1)) as f32 / (all - 1) as f32,
    }
}

/// Opens, closes and redraws the box as `Split` changes.
pub(super) fn show_split(
    split: Res<Split>,
    carried: Res<Carried>,
    boxes: Query<Entity, With<SplitBox>>,
    pictures: Query<Entity, With<SplitPicture>>,
    mut commands: Commands,
    mut drawn: Local<Option<Splitting>>,
) {
    if *drawn == split.0 {
        return;
    }
    let same_stack = drawn
        .zip(split.0)
        .is_some_and(|(was, now)| was.stack == now.stack);
    *drawn = split.0;
    let found = split.0.and_then(|splitting| {
        let (_, placed) = carried.0.placed(splitting.stack)?;
        Some((splitting, placed.clone()))
    });
    if same_stack && let Some((splitting, placed)) = &found {
        for picture in &pictures {
            commands.entity(picture).despawn_related::<Children>();
            commands
                .entity(picture)
                .with_children(|inside| draw_stack(inside, placed, splitting.count));
        }
        return;
    }
    for entity in &boxes {
        commands.entity(entity).despawn();
    }
    let Some((splitting, placed)) = found else {
        return;
    };
    let top = placed.item.top();
    let colour = top.quality.map_or(NO_QUALITY_NAME, quality_colour);
    let all = placed.item.count();
    let at = splitting.at + OFFSET;
    let node = Node {
        position_type: PositionType::Absolute,
        left: px(at.x),
        top: px(at.y),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(GAP),
        ..default()
    };
    let mut panel = commands.spawn((SplitBox, tooltip_box(node), GlobalZIndex(HELD_ITEM_Z - 1)));
    let id = panel.id();
    panel.insert(stops_presses(id)).with_children(|inside| {
        inside.spawn((
            Text::new(top.name()),
            TextSize::Medium.font(),
            TextColor(colour),
        ));
        if !placed.item.is_even() {
            // Mixed: a row for each kind in it, rather than a count.
            picker::spawn_rows(inside, &placed);
            return;
        }
        spawn_slider(inside, value_of(splitting.count, all), None, SLIDER_WIDTH)
            .insert((SplitSlider, crate::widgets::UiId::new("split", "count")));
        let shape = placed.shape();
        let size = Node {
            width: px(shape.width as f32 * CELL),
            height: px(shape.height as f32 * CELL),
            ..default()
        };
        inside
            .spawn((SplitPicture, size))
            .with_children(|picture| draw_stack(picture, &placed, splitting.count));
    });
}

fn draw_stack(parent: &mut ChildSpawnerCommands, placed: &shared::item::Placed, count: u32) {
    let look = ItemLook {
        count,
        ..ItemLook::of(&placed.item)
    };
    spawn_item_cells(
        parent,
        look.turned(placed.rotation),
        placed.shape(),
        (0.0, 0.0),
        true,
        |x, y| SplitCell { grip: (x, y) },
    );
}

/// The slider sets how many.
pub(super) fn move_split(
    mut moved: MessageReader<SliderMoved>,
    sliders: Query<(), With<SplitSlider>>,
    carried: Res<Carried>,
    mut split: ResMut<Split>,
) {
    for SliderMoved(slider, value) in moved.read() {
        if sliders.contains(*slider)
            && let Some(splitting) = &mut split.0
            && let Some(stack) = carried.0.find(splitting.stack)
        {
            let count = count_at(*value, stack.count());
            if splitting.count != count {
                splitting.count = count;
            }
        }
    }
}

/// Esc, a press anywhere else, or the stack going (or no longer being one) closes it.
/// Before the menus see Esc.
pub(super) fn close_split(
    mut split: ResMut<Split>,
    carried: Res<Carried>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    pointer: Res<UiPointer>,
    boxes: Query<(&ComputedNode, &UiGlobalTransform), With<SplitBox>>,
) {
    let Some(splitting) = split.0 else {
        return;
    };
    let gone = carried
        .0
        .placed(splitting.stack)
        .is_none_or(|(_, placed)| placed.item.count() < 2);
    let escaped = keys.clear_just_pressed(KeyCode::Escape);
    let inside = |at: Vec2| {
        boxes.iter().any(|(node, transform)| {
            let (corner, size) = logical_rect(node, transform);
            Rect::from_corners(corner, corner + size).contains(at)
        })
    };
    let elsewhere = mouse.just_pressed(MouseButton::Left) && !pointer.0.is_some_and(inside);
    if gone || escaped || elsewhere {
        split.0 = None;
    }
}

mod picker;

pub(super) use picker::PickRow;

#[cfg(test)]
mod tests;
