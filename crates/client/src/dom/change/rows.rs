//! Copies of a template of rows (`tr`), added to the end of a table: for
//! lists as tables, a row for each thing.

use bevy::prelude::*;

use super::apply::build;
use crate::dom::build::Inside;
use crate::dom::build::table::{elements, wide_column, width_of};
use crate::dom::check::{Checked, CheckedNode};
use crate::dom::parts::{Element, InTable};
use crate::dom::tag::Tag;

/// Whether a template's body is rows, which only a table takes.
pub(super) fn are_rows(nodes: &[CheckedNode]) -> bool {
    elements(nodes).any(|element| element.tag == Tag::Tr)
}

pub(super) fn copy_rows(
    world: &mut World,
    table: Entity,
    nodes: &[CheckedNode],
    inside: Inside,
) -> Result<(), String> {
    let Some(&InTable::Table { columns, wide }) = world.get::<InTable>(table) else {
        return Err("rows only go in a table".into());
    };
    let rows: Vec<&Checked> = elements(nodes).collect();
    if rows.iter().any(|row| row.tag != Tag::Tr) {
        return Err("a table only takes rows".into());
    }
    let held = world.get::<Children>(table).map(|held| held.to_vec());
    let is_row = |entity: &Entity| matches!(world.get(*entity), Some(InTable::Row { .. }));
    let old: Vec<Entity> = held
        .unwrap_or_default()
        .into_iter()
        .filter(is_row)
        .collect();
    let across = columns.max(width_of(&rows));
    let now = InTable::Table {
        columns: across,
        wide: wide.or_else(|| wide_column(&rows)),
    };
    if now != (InTable::Table { columns, wide }) {
        // The table and its rows are laid out again, to their new width.
        world.entity_mut(table).insert(now);
        for &entity in old.iter().chain([&table]) {
            if let Some(mut spot) = world.get_mut::<InTable>(entity)
                && let InTable::Row { columns, .. } = &mut *spot
            {
                *columns = across;
            }
            if let Some(mut element) = world.get_mut::<Element>(entity) {
                element.set_changed();
            }
        }
    }
    let attrs = world.get::<Element>(table).ok_or("it has gone")?;
    let attrs = attrs.attrs.clone();
    let first = old.len() as u16;
    build(world, |builder| {
        builder.rows(table, &attrs, &rows, first, across, inside);
    });
    Ok(())
}
