use shared::poi::PoiKind;

use super::super::actions::{DebugAction, Facts};
use super::on_show::kinds_on_show;
use super::*;
use crate::common::GameView;

fn facts() -> Facts {
    Facts {
        view: GameView::WorldMap,
        has_stairs: false,
        has_fields: false,
        has_herbs: false,
        walking: false,
        detecting: false,
        god_mode: false,
        truesight: false,
        killing: false,
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
    // "state" finds Fields State and Herbs State, neither possible here.
    let shown = on_show("state");
    assert_eq!(return_chooses(&shown, None, &facts()), None);
    let shown = on_show("s");
    assert_eq!(
        return_chooses(&shown, None, &facts()),
        Some(DebugAction::DetectMonsters),
        "{shown:?}"
    );
}

#[test]
fn return_on_a_disabled_entry_does_nothing() {
    let shown = on_show("stairs");
    assert_eq!(return_chooses(&shown, Some(0), &facts()), None);
}

#[test]
fn return_on_a_highlighted_entry_chooses_it() {
    let shown = on_show("");
    assert_eq!(
        return_chooses(&shown, Some(7), &facts()),
        Some(DebugAction::ForgetMap)
    );
}

#[test]
fn the_columns_on_show_follow_the_highlight_and_stop_at_the_end() {
    // 40 entries in 10 rows, 3 columns fitting.
    assert_eq!(first_column(0, Some(5), 40, 10, 3), 0);
    assert_eq!(first_column(0, Some(35), 40, 10, 3), 1);
    assert_eq!(first_column(1, Some(3), 40, 10, 3), 0);
    assert_eq!(first_column(9, None, 40, 10, 3), 1);
    assert_eq!(first_column(9, None, 5, 10, 3), 0);
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
