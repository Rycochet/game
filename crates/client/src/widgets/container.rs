//! A container's frame, the same for every grid of items: a title and controls
//! along its top, its grid, fixed or growing (scrolling past a height), and under
//! it the weight and any slots. What goes in the grid is the caller's, as carried
//! things (`inventory`) and stock (`StockGrid`) are kept differently.

use bevy::prelude::*;

use super::scroll::spawn_scroll_area;
use super::{INK, TextSize};

/// Between its rows: controls, grid and weight.
const ROW_GAP: f32 = 8.0;
/// Between the controls, and between the slots.
const ITEM_GAP: f32 = 4.0;
const FONT: TextSize = TextSize::Medium;

/// How big its grid is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContainerSize {
    /// As big as the grid put in it.
    Fixed,
    /// As big as the grid put in it, scrolling past this height: for grids that grow
    /// with what is in them.
    Grows(Val),
}

/// What a container's frame has.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContainerLook {
    /// Words at the top left, when the panel's title doesn't already say whose it is.
    pub title: Option<&'static str>,
    /// A row at the top right for buttons such as sort and filter.
    pub controls: bool,
    pub size: ContainerSize,
    /// Words under the grid saying what is in it weighs (`weight_words`): not for
    /// what isn't anyone's to carry, such as a shop's stock.
    pub weight: bool,
    /// A row under the grid at the right, for slots such as the bin.
    pub slots: bool,
}

/// Where the parts of a container's frame are, to fill in.
#[derive(Clone, Copy, Debug)]
pub struct ContainerParts {
    /// The frame itself.
    pub frame: Entity,
    pub controls: Option<Entity>,
    /// What the grid goes in.
    pub grid: Entity,
    /// The weight's text, empty until it is written.
    pub weight: Option<Entity>,
    pub slots: Option<Entity>,
}

/// A container's frame, with nothing in it yet.
pub fn spawn_container(parent: &mut ChildSpawnerCommands, look: ContainerLook) -> ContainerParts {
    let mut parts = ContainerParts {
        frame: Entity::PLACEHOLDER,
        controls: None,
        grid: Entity::PLACEHOLDER,
        weight: None,
        slots: None,
    };
    let column = Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(ROW_GAP),
        ..default()
    };
    parts.frame = parent
        .spawn(column)
        .with_children(|frame| {
            if look.title.is_some() || look.controls {
                frame.spawn(row()).with_children(|top| {
                    if let Some(title) = look.title {
                        top.spawn((Text::new(title), FONT.font(), TextColor(INK)));
                    }
                    if look.controls {
                        parts.controls = Some(top.spawn(group()).id());
                    }
                });
            }
            parts.grid = match look.size {
                ContainerSize::Fixed => frame.spawn(Node::default()).id(),
                ContainerSize::Grows(most_height) => spawn_scroll_area(frame, most_height),
            };
            if look.weight || look.slots {
                frame.spawn(row()).with_children(|bottom| {
                    if look.weight {
                        let words = (Text::default(), FONT.font(), TextColor(INK));
                        parts.weight = Some(bottom.spawn(words).id());
                    }
                    if look.slots {
                        parts.slots = Some(bottom.spawn(group()).id());
                    }
                });
            }
        })
        .id();
    parts
}

/// A row with its first part at the left and its last at the right; alone, a part
/// keeps to the right, where controls and slots go.
fn row() -> Node {
    Node {
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        column_gap: px(ITEM_GAP * 2.0),
        ..default()
    }
}

/// Controls or slots side by side.
fn group() -> Node {
    Node {
        column_gap: px(ITEM_GAP),
        margin: UiRect::left(Val::Auto),
        ..default()
    }
}

#[cfg(test)]
mod tests;
