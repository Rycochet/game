//! Laying a list out in columns: each group under its heading, groups kept
//! whole in a column where they fit, and moving the highlight about them.

use crate::widgets::HighlightStep;

/// One row of a column: a group's heading, or an entry (by its place among
/// those on show).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Slot {
    Heading(&'static str),
    Entry(usize),
}

/// The columns, top to bottom.
pub(super) type Columns = Vec<Vec<Slot>>;

/// Lays out entries with these headings (`None` for a list with no groups,
/// such as Generate POI's kinds) in columns of `rows`. A group starts a new
/// column when it doesn't fit below the last; one taller than a column goes on
/// in the next, under its heading again.
pub(super) fn lay_out(headings: &[Option<&'static str>], rows: usize) -> Columns {
    let rows = rows.max(1);
    if headings.iter().all(Option::is_none) {
        return (0..headings.len())
            .collect::<Vec<_>>()
            .chunks(rows)
            .map(|chunk| chunk.iter().map(|&at| Slot::Entry(at)).collect())
            .collect();
    }
    // A heading with nothing under it is no use.
    let rows = rows.max(2);
    let mut columns: Columns = Vec::new();
    let mut at = 0;
    while at < headings.len() {
        let heading = headings[at];
        let size = headings[at..].iter().take_while(|&&h| h == heading).count();
        let fits_below = columns
            .last()
            .is_some_and(|column| column.len() + 1 + size <= rows);
        if !fits_below {
            columns.push(Vec::new());
        }
        for entry in at..at + size {
            let column = columns.last_mut().expect("one was pushed");
            let full = column.len() == rows;
            if full {
                columns.push(Vec::new());
            }
            let column = columns.last_mut().expect("one was pushed");
            if (entry == at || full)
                && let Some(name) = heading
            {
                column.push(Slot::Heading(name));
            }
            column.push(Slot::Entry(entry));
        }
        at += size;
    }
    columns
}

/// The column and row an entry is in.
pub(super) fn place_of(columns: &Columns, entry: usize) -> Option<(usize, usize)> {
    columns.iter().enumerate().find_map(|(column, slots)| {
        let row = slots.iter().position(|&slot| slot == Slot::Entry(entry))?;
        Some((column, row))
    })
}

/// Where the arrow keys move the highlight: Up and Down along the list, Left
/// and Right to the nearest entry in the next column over.
pub(super) fn step(
    columns: &Columns,
    highlight: Option<usize>,
    count: usize,
    step: HighlightStep,
) -> Option<usize> {
    let last = count.checked_sub(1)?;
    let Some(at) = highlight else {
        return Some(if step == HighlightStep::Up { last } else { 0 });
    };
    match step {
        HighlightStep::Up => Some(at.saturating_sub(1)),
        HighlightStep::Down => Some((at + 1).min(last)),
        HighlightStep::Left | HighlightStep::Right => {
            let Some((column, row)) = place_of(columns, at) else {
                return Some(at);
            };
            let next = if step == HighlightStep::Left {
                column.checked_sub(1)
            } else {
                Some(column + 1)
            };
            let nearest = next.and_then(|next| columns.get(next)).and_then(|slots| {
                slots
                    .iter()
                    .enumerate()
                    .filter_map(|(r, &slot)| match slot {
                        Slot::Entry(entry) => Some((r.abs_diff(row), entry)),
                        Slot::Heading(_) => None,
                    })
                    .min()
                    .map(|(_, entry)| entry)
            });
            Some(nearest.unwrap_or(at))
        }
    }
}

/// The first column to draw, from `first`, so the highlighted entry shows and
/// no room is wasted at the end.
pub(super) fn first_column(
    columns: &Columns,
    first: usize,
    highlight: Option<usize>,
    fit: usize,
) -> usize {
    let first = match highlight.and_then(|at| place_of(columns, at)) {
        Some((column, _)) if column < first => column,
        Some((column, _)) if column >= first + fit => column + 1 - fit,
        _ => first,
    };
    first.min(columns.len().saturating_sub(fit))
}
