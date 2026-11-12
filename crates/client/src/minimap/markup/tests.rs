use super::{FULL_MAP, MINIMAP};
use crate::minimap::gate_runes::RUNES_HEIGHT;
use crate::minimap::{CORNER_GAP, FULL_MAP_MARGIN, HOVER_LIFT, TITLE_HEIGHT};
use crate::widgets::PANEL_EDGE;

/// Whether `template` gives the element with this id this attribute.
fn has(source: &str, id: &str, attribute: &str, value: f32) -> bool {
    let wanted = format!(r#"{attribute}="{value}""#);
    let element = format!(r#"id="{id}""#);
    source
        .lines()
        .any(|line| line.contains(&element) && line.contains(&wanted))
}

#[test]
fn the_minimap_is_good_markup_in_its_corner() {
    MINIMAP.assert_good();
    assert!(has(MINIMAP.source, "map", "top", CORNER_GAP));
    assert!(has(MINIMAP.source, "map", "right", CORNER_GAP));
}

/// The code that finds what the pointer is on works from these constants,
/// so the markup has to place things by them (less the parchment's edge).
#[test]
fn the_full_map_is_good_markup_placed_by_the_constants() {
    FULL_MAP.assert_good();
    let source = FULL_MAP.source;
    let margin = FULL_MAP_MARGIN - PANEL_EDGE;
    assert!(has(source, "picture", "left", margin));
    assert!(has(source, "picture", "top", margin + TITLE_HEIGHT));
    assert!(has(source, "hover", "bottom", margin + HOVER_LIFT));
    assert!(has(source, "code", "height", RUNES_HEIGHT));
}
