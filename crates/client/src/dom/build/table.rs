//! Tables: a grid, with each row a strip behind its cells. Rows and cells
//! are all children of the table, as a grid only places its own children.

use bevy::prelude::*;

use super::{Builder, Inside};
use crate::dom::check::{Checked, CheckedNode};
use crate::dom::parts::InTable;

impl Builder<'_, '_, '_> {
    pub(super) fn table(&mut self, table: Entity, element: &Checked, inside: Inside) {
        let rows: Vec<&Checked> = elements(&element.children).collect();
        let width = |row: &&Checked| elements(&row.children).map(span).sum::<u16>();
        let columns = rows.iter().map(width).max().unwrap_or(1).max(1);
        let striped = element.attrs.class.as_deref() == Some("striped");
        let padding = element.attrs.cellpadding.unwrap_or(0.0);
        self.commands
            .entity(table)
            .insert(InTable::Table { columns });

        for (row, tr) in (0u16..).zip(rows) {
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
fn elements(nodes: &[CheckedNode]) -> impl Iterator<Item = &Checked> {
    nodes.iter().filter_map(|node| match node {
        CheckedNode::Element(element) => Some(element),
        CheckedNode::Text(_) => None,
    })
}

fn span(cell: &Checked) -> u16 {
    cell.attrs.colspan.unwrap_or(1)
}
