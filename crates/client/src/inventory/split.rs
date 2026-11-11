//! Splitting a stack (owner's design): shift+click, or a tap, on a stack opens a box
//! with its name, a slider from one to all of it, and the stack at that many below.
//! Pressing that picks up that many from its top, closing the box; letting go goes
//! by the usual rules, and only what was taken moves. A mixed stack opens it on any
//! press, with a row for each kind in it instead (`picker`).

use bevy::prelude::*;
use shared::id::Id;

use super::Carried;
use crate::common::{NO_QUALITY_NAME, quality_colour};
use crate::dom::{Dom, Template, UiEvent, UiValue, document};
use crate::html;
use crate::widgets::{CELL, ItemLook, TextSize, UiId, spawn_item_cells};

/// How far from where it was opened the box goes.
const OFFSET: Vec2 = Vec2::new(16.0, 16.0);

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

/// The box: the stack's name, a slider from one to all of it with the stack at
/// that many under it, or for a mixed stack a row for each kind in it.
static SPLIT: Template = html!(
    "inventory.split",
    r#"
<form id="split">
  <popup id="box" align="center">
    <canvas id="name"/>
    <input id="count" type="range" min="1" max="2" step="1" value="1" width="128"/>
    <canvas id="stack"/>
    <canvas id="rows"/>
  </popup>
</form>
"#
);

/// On the drawn stack's cells: which cell of it, as it lies.
#[derive(Component, Clone, Copy)]
pub(super) struct SplitCell {
    pub(super) grip: (u8, u8),
}

/// What the stack is drawn in, to draw it again at another count.
#[derive(Component)]
pub(super) struct SplitPicture;

/// Opens and closes the box as `Split` changes, and says where it is and how
/// its slider stands.
pub(super) fn show_split(
    split: Res<Split>,
    carried: Res<Carried>,
    pictures: Query<Entity, With<SplitPicture>>,
    mut dom: Dom,
    mut commands: Commands,
    (mut drawn, mut open): (Local<Option<Splitting>>, Local<Option<Entity>>),
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
        // Only how many: the stack drawn again.
        for picture in &pictures {
            commands.entity(picture).despawn_related::<Children>();
            commands
                .entity(picture)
                .with_children(|inside| draw_stack(inside, placed, splitting.count));
        }
        return;
    }
    if let Some(document) = open.take() {
        commands.entity(document).try_despawn();
    }
    let Some((splitting, placed)) = found else {
        return;
    };
    *open = Some(commands.spawn(document(&SPLIT)).id());
    let at = splitting.at + OFFSET;
    dom.set("split/box", "left", &at.x.round().to_string());
    dom.set("split/box", "top", &at.y.round().to_string());
    // Mixed: a row for each kind in it, rather than a count.
    let even = placed.item.is_even();
    dom.set_flag("split/count", "hidden", !even);
    dom.set_flag("split/stack", "hidden", !even);
    dom.set_flag("split/rows", "hidden", even);
    dom.set("split/count", "max", &placed.item.count().to_string());
    dom.set("split/count", "value", &splitting.count.to_string());
}

/// Fills the box's places once it is built: the stack's name in its quality's
/// colour, the stack itself, and a mixed stack's rows.
pub(super) fn bind_split(
    new: Query<(Entity, &UiId), Added<UiId>>,
    split: Res<Split>,
    carried: Res<Carried>,
    mut commands: Commands,
) {
    for (entity, id) in &new {
        let ("split", name) = id.parts() else {
            continue;
        };
        let found = split.0.and_then(|splitting| {
            let (_, placed) = carried.0.placed(splitting.stack)?;
            Some((splitting, placed))
        });
        let Some((splitting, placed)) = found else {
            continue;
        };
        let mut place = commands.entity(entity);
        match name {
            "name" => {
                let top = placed.item.top();
                let colour = top.quality.map_or(NO_QUALITY_NAME, quality_colour);
                let words = (Text::new(top.name()), TextSize::Medium.font());
                place.with_child((words, TextColor(colour)));
            }
            "stack" if placed.item.is_even() => {
                let shape = placed.shape();
                let size = Node {
                    width: px(shape.width as f32 * CELL),
                    height: px(shape.height as f32 * CELL),
                    ..default()
                };
                place.with_children(|inside| {
                    inside
                        .spawn((SplitPicture, size))
                        .with_children(|picture| draw_stack(picture, placed, splitting.count));
                });
            }
            "rows" if !placed.item.is_even() => {
                place.with_children(|inside| picker::spawn_rows(inside, placed));
            }
            _ => {}
        }
    }
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
    mut events: MessageReader<UiEvent>,
    carried: Res<Carried>,
    mut split: ResMut<Split>,
) {
    for event in events.read() {
        if event.id == UiId::new("split", "count")
            && let UiValue::Number(value) = event.value
            && let Some(splitting) = &mut split.0
            && let Some(stack) = carried.0.find(splitting.stack)
        {
            let count = (value.round() as u32).clamp(1, stack.count().max(1));
            if splitting.count != count {
                splitting.count = count;
            }
        }
    }
}

/// The box closing itself (Esc, or a press anywhere else, unless it is locked
/// open), or the stack going (or no longer being one), ends the splitting.
/// Before presses are taken, so one on another stack opens it afresh.
pub(super) fn close_split(
    mut events: MessageReader<UiEvent>,
    mut split: ResMut<Split>,
    carried: Res<Carried>,
) {
    let closed = events
        .read()
        .any(|event| event.id == UiId::new("split", "box") && event.value == UiValue::Closed);
    let Some(splitting) = split.0 else {
        return;
    };
    let gone = carried
        .0
        .placed(splitting.stack)
        .is_none_or(|(_, placed)| placed.item.count() < 2);
    if gone || closed {
        split.0 = None;
    }
}

mod picker;

pub(super) use picker::PickRow;

#[cfg(test)]
mod tests;
