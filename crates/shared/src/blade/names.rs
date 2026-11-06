//! `weapon_names`: every name a weapon of one family may go by, for the skill tree's
//! node for each weapon (`shared::skill`).

use super::table::rows;
use super::{CELLS, Family, overlap};

/// Every name a weapon of this family may be called: its rows' names, best known
/// first, then the generic names of the cells its rows can be made in, each once.
pub fn weapon_names(family: Family) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = Vec::new();
    let family_rows: Vec<_> = rows().filter(|row| row.family == family).collect();
    for row in &family_rows {
        if !names.contains(&row.name) {
            names.push(row.name);
        }
    }
    for cell in &CELLS {
        let reached = family_rows.iter().any(|row| {
            row.build == cell.build
                && overlap(row.blade, cell.blade).is_some()
                && overlap(row.handle, cell.handle).is_some()
        });
        if let Some(name) = cell.generic.filter(|name| reached && !names.contains(name)) {
            names.push(name);
        }
    }
    names
}
