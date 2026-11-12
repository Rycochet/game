use shared::site::StairsStyle;

use super::generate_form::NEW_SEEDS;
use super::steps::{Drawn, can_step, previous_style};
use super::teleport_to_world::seed_in;
use super::time_text::{read_time, time_text};
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
    assert!(!in_site(Drawn::Stairs));
    assert!(!in_site(Drawn::Fields));
    assert!(!in_site(Drawn::Herbs));
    assert!(!can_step(Drawn::Stairs, GameView::WorldMap, &dungeon));
}

#[test]
fn world_time_reads_back_what_it_writes() {
    for minutes in [0.0, 59.0, 60.0, 1439.0, 1440.0, 100_000.0] {
        assert_eq!(read_time(&time_text(minutes), 0.0), Some(minutes));
    }
    assert_eq!(time_text(0.0), "Day 1 00:00");
    assert_eq!(time_text(1440.0 + 6.0 * 60.0 + 30.5), "Day 2 06:30");
}

#[test]
fn world_time_alone_is_on_the_day_shown() {
    let day_three = 2.0 * 1440.0 + 500.0;
    assert_eq!(read_time("06:30", day_three), Some(2.0 * 1440.0 + 390.0));
    assert_eq!(
        read_time("  day 3   06:30 ", 0.0),
        Some(2.0 * 1440.0 + 390.0)
    );
}

#[test]
fn world_time_refuses_what_it_cant_read() {
    for text in [
        "",
        "Day",
        "Day 0 00:00",
        "Day -1 00:00",
        "Day 1 24:00",
        "Day 1 10:60",
        "Day 1 10:5",
        "Day 1 1030",
        "Night 1 10:30",
        "Day x 10:30",
        "Day 99999999999 10:30",
    ] {
        assert_eq!(read_time(text, 0.0), None, "{text:?}");
    }
}

/// The qualities are written out in Item Pictures' markup, in order.
#[test]
fn item_pictures_offers_every_quality() {
    let source = pictures_spawn::ITEM_PICTURES.source;
    let mut from = 0;
    for quality in shared::item::Quality::ALL {
        let option = format!(">{quality:?}</option>");
        let at = source[from..].find(&option);
        from += at.unwrap_or_else(|| panic!("{quality:?} is missing, or out of order"));
    }
    assert_eq!(source.matches("</option>").count(), 8, "and the page's own");
}

/// A template's check finds an id used twice, so being good covers that.
#[test]
fn the_panels_written_as_markup_are_good() {
    vitals::VITALS.assert_good();
    steps::DRAWING_STATES.assert_good();
    change_creature::CHANGE_CREATURE.assert_good();
    world_time::WORLD_TIME.assert_good();
    examine_mob::EXAMINE_MOB.assert_good();
    create_mob::CREATE_MOB.assert_good();
    dev_area::DEV_AREA.assert_good();
    learn_skills::LEARN_SKILLS.assert_good();
    give::GIVE_ITEM.assert_good();
    pictures_spawn::ITEM_PICTURES.assert_good();
    generate::GENERATE_POI.assert_good();
    teleport_to_world::TELEPORT_TO_WORLD.assert_good();
}

#[test]
fn a_blank_seed_is_this_world_and_words_are_none() {
    assert_eq!(seed_in(""), Ok(None));
    assert_eq!(seed_in("  "), Ok(None));
    assert_eq!(seed_in(" 42 "), Ok(Some(42)));
    assert_eq!(seed_in("4x2"), Err(()));
}
