use shared::poi::PoiKind;
use shared::protocol::LevelChunk;

use super::*;

fn info(index: u32, bounds: (i64, i64, i64, i64), entering_site: bool) -> LevelInfo {
    LevelInfo {
        kind: shared::protocol::LevelKind::Poi(PoiKind::RockCave),
        ruined: false,
        building: false,
        index,
        levels: 3,
        basement: false,
        bounds,
        arrival: (1, 1),
        entering_site,
    }
}

#[test]
fn floors_count_from_nought_in_the_eu_the_uk_and_latin_america() {
    for locale in [
        "en-GB",
        "en_GB.UTF-8",
        "fr-FR",
        "de",
        "es-419",
        "es-MX",
        "pt-BR",
        "en-IE",
        "sr-Latn-HR",
    ] {
        assert_eq!(first_floor_in(locale), 0, "{locale}");
    }
    for locale in ["en-US", "en", "es-US", "fr-CA", "ja-JP", "zh-Hant-TW", ""] {
        assert_eq!(first_floor_in(locale), 1, "{locale}");
    }
}

fn remembered(index: u32, bounds: (i64, i64, i64, i64)) -> RememberedLevel {
    RememberedLevel {
        index,
        bounds,
        chunks: vec![LevelChunk {
            chunk: (0, 0),
            mask: 1,
            cells: vec![shared::dungeon::Cell::Floor],
            stairs: Vec::new(),
            remembered: true,
        }],
    }
}

#[test]
fn levels_are_kept_on_taking_stairs_and_forgotten_on_coming_in_again() {
    let mut current = CurrentDungeon::default();
    let mut others = OtherLevels::default();
    let bounds = (0, 0, 8, 8);
    others.entering(&mut current, &info(0, bounds, true));
    current.enter(info(0, bounds, true));
    // Nothing seen, nothing known: it isn't kept.
    others.entering(&mut current, &info(1, bounds, false));
    current.enter(info(1, bounds, false));
    assert_eq!(others.floors().count(), 0);

    others.add(&current, &remembered(2, bounds));
    current.remembered.insert(1, 1);
    others.entering(&mut current, &info(2, bounds, false));
    current.enter(info(2, bounds, false));
    // The floor left is kept; the one arrived on is the current one now.
    assert_eq!(others.floors().collect::<Vec<_>>(), [1]);

    others.show(1);
    assert_eq!(others.showing(), Some(1));
    assert!(others.shown(&current).remembered.is_seen(1, 1));
    others.show(2);
    assert_eq!(others.showing(), None);

    others.entering(&mut current, &info(0, bounds, true));
    assert_eq!(others.floors().count(), 0);
}

#[test]
fn the_players_place_carries_across_by_cell_or_by_share_of_the_level() {
    let mut current = CurrentDungeon::default();
    current.enter(info(0, (0, 0, 10, 20), true));
    let mut others = OtherLevels::default();
    others.add(&current, &remembered(1, (-10, 100, 40, 40)));
    assert_eq!(others.spot(&current, (5.0, 5.0)), (5.0, 5.0));
    others.show(1);
    assert_eq!(others.spot(&current, (5.0, 5.0)), (10.0, 110.0));
    current.info.as_mut().expect("entered").building = true;
    assert_eq!(others.spot(&current, (5.0, 5.0)), (5.0, 5.0));
}
