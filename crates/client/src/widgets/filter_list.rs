//! What lists with a text box to filter them have in common (the debug menu,
//! a dropdown with search): which names match, and how arrow keys move through
//! what is on show. No Bevy systems here, so it can be tested alone.

/// Whether a name matches what has been typed: anywhere in it, whatever the case.
pub fn matches_filter(name: &str, filter: &str) -> bool {
    name.to_lowercase().contains(&filter.to_lowercase())
}

/// A press of an arrow key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HighlightStep {
    Up,
    Down,
    Left,
    Right,
}

/// Where the highlight goes among `count` entries filling columns of `rows`, top
/// to bottom. A single column is `rows` of `count`.
pub fn next_highlight(
    highlight: Option<usize>,
    count: usize,
    rows: usize,
    step: HighlightStep,
) -> Option<usize> {
    if count == 0 {
        return None;
    }
    let last = count - 1;
    Some(match (highlight, step) {
        (None, HighlightStep::Up) => last,
        (None, _) => 0,
        (Some(at), HighlightStep::Up) => at.saturating_sub(1),
        (Some(at), HighlightStep::Down) => (at + 1).min(last),
        (Some(at), HighlightStep::Left) => at.checked_sub(rows).unwrap_or(at),
        (Some(at), HighlightStep::Right) => {
            if at + rows <= last {
                at + rows
            } else {
                at
            }
        }
    })
}

#[cfg(test)]
mod tests;
