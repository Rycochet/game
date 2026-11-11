//! Tables: a grid, with each row a strip behind its cells. Rows and cells
//! are all children of the table, as a grid only places its own children.

use bevy::prelude::*;

use super::{Builder, Inside};
use crate::dom::attrs::Attrs;
use crate::dom::check::{Checked, CheckedNode};
use crate::dom::parts::InTable;

impl Builder<'_, '_, '_> {
    pub(super) fn table(&mut self, table: Entity, element: &Checked, inside: Inside) {
        let rows: Vec<&Checked> = elements(&element.children).collect();
        let columns = width_of(&rows);
        let wide = wide_column(&rows);
        self.commands
            .entity(table)
            .insert(InTable::Table { columns, wide });
        self.rows(table, &element.attrs, &rows, 0, columns, inside);
    }

    /// Adds rows to a table of this many columns, the first of them its
    /// row `first`. `attrs` are the table's.
    pub(in crate::dom) fn rows(
        &mut self,
        table: Entity,
        attrs: &Attrs,
        rows: &[&Checked],
        first: u16,
        columns: u16,
        inside: Inside,
    ) {
        let striped = attrs.class.as_deref() == Some("striped");
        let padding = attrs.number("cellpadding").unwrap_or(0.0);
        for (row, tr) in (first..).zip(rows) {
            let (strip, within) = self.shell(table, tr, inside);
            let spot = InTable::Row {
                row,
                columns,
                tinted: striped && row % 2 == 1,
            };
            self.commands.entity(strip).insert(spot);
            let mut column = 0;
            for cell in elements(&tr.children) {
                let (entity, within) = self.shell(table, cell, within);
                let spot = InTable::Cell {
                    of: strip,
                    row,
                    column,
                    span: span(cell),
                    padding,
                };
                self.commands.entity(entity).insert(spot);
                self.nodes(entity, &cell.children, within);
                column += span(cell);
            }
        }
    }
}

/// The elements among these (a table and its rows hold nothing else).
pub(in crate::dom) fn elements(nodes: &[CheckedNode]) -> impl Iterator<Item = &Checked> {
    nodes.iter().filter_map(|node| match node {
        CheckedNode::Element(element) => Some(element),
        CheckedNode::Text(_) => None,
    })
}

fn span(cell: &Checked) -> u16 {
    cell.attrs.number("colspan").map_or(1, |span| span as u16)
}

/// How many columns these rows need: the widest of them, and at least one.
pub(in crate::dom) fn width_of(rows: &[&Checked]) -> u16 {
    let width = |row: &&Checked| elements(&row.children).map(span).sum::<u16>();
    rows.iter().map(width).max().unwrap_or(1).max(1)
}

/// The column of the first cell marked `fills`, which takes the width the
/// others leave.
pub(in crate::dom) fn wide_column(rows: &[&Checked]) -> Option<u16> {
    rows.iter().find_map(|row| {
        let mut column = 0;
        elements(&row.children).find_map(|cell| {
            let at = column;
            column += span(cell);
            cell.attrs.fills.then_some(at)
        })
    })
}
