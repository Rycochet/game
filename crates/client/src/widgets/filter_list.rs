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
mod tests {
    use super::*;

    #[test]
    fn the_filter_ignores_case_and_matches_anywhere() {
        assert!(matches_filter("Teleport Mode", "PORT"));
        assert!(matches_filter("Teleport Mode", ""));
        assert!(!matches_filter("Teleport Mode", "stairs"));
    }

    #[test]
    fn arrows_move_through_the_columns_and_stop_at_the_ends() {
        use HighlightStep::*;
        let go = |from, step| next_highlight(from, 10, 4, step);
        assert_eq!(go(None, Down), Some(0));
        assert_eq!(go(None, Up), Some(9));
        assert_eq!(go(Some(3), Down), Some(4));
        assert_eq!(go(Some(9), Down), Some(9));
        assert_eq!(go(Some(0), Up), Some(0));
        assert_eq!(go(Some(1), Right), Some(5));
        assert_eq!(go(Some(5), Left), Some(1));
        assert_eq!(go(Some(1), Left), Some(1));
        // No such row in the next column: stays.
        assert_eq!(go(Some(7), Right), Some(7));
        assert_eq!(next_highlight(None, 0, 4, Down), None);
    }
}
