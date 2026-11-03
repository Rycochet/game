use shared::site::StairsStyle;

use super::super::actions::DebugAction;
use super::generate_form::NEW_SEEDS;
use super::steps::{can_step, previous_style};
use super::*;
use crate::common::{CurrentDungeon, GameView};

#[test]
fn new_seeds_are_short_and_differ() {
    let seeds: Vec<u64> = (0..50).map(|made| new_seed(12345, made)).collect();
    assert!(seeds.iter().all(|&seed| seed < NEW_SEEDS));
    assert!(seeds.iter().any(|&seed| seed != seeds[0]));
}

#[test]
fn previous_undoes_next_for_every_style() {
    for style in StairsStyle::ALL {
        assert_eq!(previous_style(style.next()), style);
    }
}

#[test]
fn panels_need_a_site_with_what_they_change() {
    let dungeon = CurrentDungeon::default();
    let in_site = |action| can_step(action, GameView::Dungeon, &dungeon);
    assert!(!in_site(DebugAction::StairsStyle));
    assert!(!in_site(DebugAction::FieldsState));
    assert!(!in_site(DebugAction::HerbsState));
    assert!(!can_step(
        DebugAction::StairsStyle,
        GameView::WorldMap,
        &dungeon
    ));
}
