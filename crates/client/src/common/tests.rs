use shared::poi::ChunkPos;
use shared::poi::PoiKind;
use shared::protocol::SeenPoi;

use super::*;

#[test]
fn an_impossibly_wide_area_finds_no_pois() {
    let mut known = KnownPois::new(1000);
    let chunk = ChunkPos { x: 0, y: 1 };
    let poi = SeenPoi {
        kind: PoiKind::Crypt,
        look: shared::site::PoiLook::Cave {
            entrance: shared::site::CaveEntrance::Hole,
        },
        chunk,
        x: 10,
        y: 70,
    };
    known.insert(chunk, Some(poi), None);
    let found = known.in_area((i64::MIN, i64::MIN), (i64::MAX, i64::MAX));
    assert!(found.is_empty());
}

#[test]
fn known_pois_are_found_in_any_frame() {
    let mut known = KnownPois::new(1000);
    let chunk = ChunkPos { x: 0, y: 1 };
    let poi = SeenPoi {
        kind: PoiKind::Crypt,
        look: shared::site::PoiLook::Cave {
            entrance: shared::site::CaveEntrance::Hole,
        },
        chunk,
        x: 10,
        y: 70,
    };
    known.insert(chunk, Some(poi), None);
    known.insert(ChunkPos { x: 1, y: 1 }, None, None);
    let found = |min, max| -> Vec<i64> { known.in_area(min, max).iter().map(|f| f.x).collect() };
    assert_eq!(found((0, 64), (100, 127)), [10]);
    // From west of the seam, and a whole planet east.
    assert_eq!(found((-50, 0), (50, 100)), [10]);
    assert_eq!(found((900, 0), (1100, 100)), [1010]);
    // Outside the area, or rows that weren't sent.
    assert!(found((20, 0), (500, 100)).is_empty());
    assert!(found((0, 128), (100, 300)).is_empty());
}

#[test]
fn screen_up_follows_the_view() {
    // Up the screen, for each way the view can face.
    let up: Vec<_> = (0..4)
        .map(|q| ViewFacing(q).screen_to_grid(0, -1))
        .collect();
    assert_eq!(up, [(0, -1), (1, 0), (0, 1), (-1, 0)]);
    // Right on screen when east is at the top: south.
    assert_eq!(ViewFacing(1).screen_to_grid(1, 0), (0, 1));
}

#[test]
fn keep_drawing_keeps_the_focused_rate_unfocused() {
    let graphics = GraphicsPrefs::default();
    let kept = frame_settings(true, &graphics);
    assert_eq!(kept.unfocused_mode, kept.focused_mode);
    let slowed = frame_settings(false, &graphics);
    assert_ne!(slowed.unfocused_mode, slowed.focused_mode);
    // The player's choice keeps it too.
    let chosen = GraphicsPrefs {
        background: Background::KeepDrawing,
        ..graphics
    };
    let kept = frame_settings(false, &chosen);
    assert_eq!(kept.unfocused_mode, kept.focused_mode);
}

#[test]
fn learned_names_are_kept_by_chunk_until_sent_without() {
    let mut known = KnownPois::new(1000);
    let chunk = ChunkPos { x: 0, y: 1 };
    let poi = SeenPoi {
        kind: PoiKind::Village,
        look: shared::site::PoiLook::Town { buildings: 9 },
        chunk,
        x: 10,
        y: 70,
    };
    known.insert(chunk, Some(poi), None);
    assert_eq!(known.name_of(chunk), None);
    assert_eq!(poi_name(&poi, known.name_of(chunk)), "Village");
    known.insert(chunk, Some(poi), Some("Thornbury Green".into()));
    assert_eq!(poi_name(&poi, known.name_of(chunk)), "Thornbury Green");
    known.insert(chunk, Some(poi), None);
    assert_eq!(known.name_of(chunk), None);
}

#[test]
fn a_ruins_learned_name_says_it_is_ruined() {
    let poi = SeenPoi {
        kind: PoiKind::Castle,
        look: shared::site::PoiLook::Building {
            outline: shared::site::Outline::Rectangle,
            width: 20,
            height: 20,
            floors: 3,
            ruin: 2,
        },
        chunk: ChunkPos { x: 0, y: 0 },
        x: 10,
        y: 10,
    };
    assert_eq!(
        poi_name(&poi, Some("Castle Ravenhold")),
        "Ruined Castle Ravenhold"
    );
    assert_eq!(poi_name(&poi, None), "Ruined castle");
}
