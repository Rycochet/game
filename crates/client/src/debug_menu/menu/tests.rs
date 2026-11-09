use shared::poi::PoiKind;

use super::super::actions::{DebugAction, Facts};
use super::on_show::kinds_on_show;
use super::*;
use crate::common::GameView;

fn facts() -> Facts {
    Facts {
        view: GameView::WorldMap,
        detecting: false,
        god_mode: false,
        truesight: false,
        freezing: false,
        teleporting: false,
        changing_cells: false,
        showing_info: false,
        in_town: false,
        world_time_open: false,
        suns_and_moons_open: false,
        dev_tools: true,
    }
}

#[test]
fn return_with_nothing_highlighted_skips_what_is_disabled() {
    // The doors' tools, none possible on the world map.
    let shown = on_show("door");
    assert_eq!(shown.len(), 3);
    assert_eq!(return_chooses(&shown, None, &facts()), None);
    let shown = on_show("s");
    assert_eq!(
        return_chooses(&shown, None, &facts()),
        Some(DebugAction::ShowDebugInfo),
        "{shown:?}"
    );
}

#[test]
fn return_on_a_disabled_entry_does_nothing() {
    let shown = on_show("door");
    assert_eq!(return_chooses(&shown, Some(0), &facts()), None);
}

#[test]
fn return_on_a_highlighted_entry_chooses_it() {
    let shown = on_show("");
    assert_eq!(
        return_chooses(&shown, Some(9), &facts()),
        Some(DebugAction::GodMode)
    );
}

#[test]
fn groups_stay_whole_in_a_column_where_they_fit() {
    use layout::Slot::{Entry, Heading};
    let headings = [
        Some("A"),
        Some("A"),
        Some("B"),
        Some("C"),
        Some("C"),
        Some("C"),
    ];
    let columns = lay_out(&headings, 5);
    assert_eq!(
        columns,
        [
            vec![Heading("A"), Entry(0), Entry(1), Heading("B"), Entry(2)],
            vec![Heading("C"), Entry(3), Entry(4), Entry(5)],
        ]
    );
    // Too tall for a column: it goes on under its heading again.
    let columns = lay_out(&[Some("A"); 5], 3);
    assert_eq!(
        columns,
        [
            vec![Heading("A"), Entry(0), Entry(1)],
            vec![Heading("A"), Entry(2), Entry(3)],
            vec![Heading("A"), Entry(4)],
        ]
    );
    // With no groups, as the kinds of poi: plain columns.
    assert_eq!(
        lay_out(&[None; 3], 2),
        [vec![Entry(0), Entry(1)], vec![Entry(2)]]
    );
}

#[test]
fn arrows_move_along_the_list_and_across_to_the_nearest_entry() {
    use crate::widgets::HighlightStep::{Down, Left, Right, Up};
    let headings = [
        Some("A"),
        Some("A"),
        Some("B"),
        Some("C"),
        Some("C"),
        Some("C"),
    ];
    let columns = lay_out(&headings, 5);
    assert_eq!(layout::step(&columns, None, 6, Down), Some(0));
    assert_eq!(layout::step(&columns, None, 6, Up), Some(5));
    assert_eq!(layout::step(&columns, Some(1), 6, Down), Some(2));
    // Entry 1 is on the third row; across, the nearest is entry 4.
    assert_eq!(layout::step(&columns, Some(1), 6, Right), Some(4));
    // Entry 5 is on the fourth row: entries 1 and 2 are as near, so the upper.
    assert_eq!(layout::step(&columns, Some(5), 6, Left), Some(1));
    assert_eq!(layout::step(&columns, Some(5), 6, Right), Some(5));
    assert_eq!(layout::step(&columns, None, 0, Down), None);
}

#[test]
fn the_columns_on_show_follow_the_highlight_and_stop_at_the_end() {
    // 40 entries in 10 rows: 4 columns, 3 fitting.
    let columns = lay_out(&[None; 40], 10);
    assert_eq!(first_column(&columns, 0, Some(5), 3), 0);
    assert_eq!(first_column(&columns, 0, Some(35), 3), 1);
    assert_eq!(first_column(&columns, 1, Some(3), 3), 0);
    assert_eq!(first_column(&columns, 9, None, 3), 1);
    let few = lay_out(&[None; 5], 10);
    assert_eq!(first_column(&few, 9, None, 3), 0);
}

#[test]
fn every_kind_is_on_the_submenu_until_filtered() {
    assert_eq!(kinds_on_show("").len(), PoiKind::ALL.len());
    assert_eq!(kinds_on_show("stone tower"), [PoiKind::StoneTower]);
}

#[test]
fn columns_hold_what_fits() {
    assert_eq!(rows_in(720.0, 0), 13);
    assert_eq!(rows_in(10.0, 0), 1);
    assert!(rows_in(720.0, 1) < rows_in(720.0, 0));
}
