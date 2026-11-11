//! The components a document's entities carry.

use bevy::prelude::*;

use super::attrs::Attrs;
use super::check::CheckedNode;
use super::tag::Tag;
use super::template::Template;
use crate::widgets::TextSize;

/// The root of what a template was built into. Spawn it with `document`.
#[derive(Component, Clone, Copy, Debug)]
pub struct Document(pub &'static Template);

/// What to spawn for a template to be built there, by the end of the frame.
pub fn document(template: &'static Template) -> impl Bundle {
    let node = Node {
        flex_direction: FlexDirection::Column,
        ..default()
    };
    (Document(template), node)
}

/// One element of a document. Its `Node` and looks follow it: change it
/// through `Dom`, by its id.
#[derive(Component, Clone, Debug)]
pub struct Element {
    pub tag: Tag,
    pub(super) attrs: Attrs,
}

/// On an element the pointer is never on, nor on anything in it (`inert`).
#[derive(Component, Clone, Copy, Debug)]
pub struct Inert;

/// On anything the pointer can be on, and so hides what is behind it from the
/// pointer. Every element is; of other nodes, only those given this (the old
/// widgets' buttons and panels require it).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Solid;

/// On an element while the pointer is on it or on something in it.
#[derive(Component, Clone, Copy, Debug)]
pub struct Hovered;

/// On a `canvas`: a box for its owner's code to draw in.
#[derive(Component, Clone, Copy, Debug)]
pub struct Canvas;

/// The writing of an element and what is in it, unless that says otherwise.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(super) struct TextStyle {
    pub size: TextSize,
    pub colour: Color,
}

/// On the words of an element.
#[derive(Component)]
pub(super) struct Words;

/// Where a table or one of its parts is in the table's grid. Rows and cells
/// are all children of the table: a row is a strip behind its cells.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(super) enum InTable {
    Table {
        columns: u16,
    },
    Row {
        row: u16,
        columns: u16,
        /// One of the tinted rows of a `striped` table.
        tinted: bool,
    },
    Cell {
        /// Its row's entity.
        of: Entity,
        row: u16,
        column: u16,
        span: u16,
        /// The table's `cellpadding`.
        padding: f32,
    },
}

/// On a `progress` with a value: the bar inside it.
#[derive(Component)]
pub(super) struct Bar(pub Entity);

/// On a `template`: what a copy of it is made from.
#[derive(Component)]
pub(super) struct TemplateBody(pub Vec<CheckedNode>);
