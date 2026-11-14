use super::{FULL_MAP, MINIMAP};
use crate::minimap::floors::{BUTTON, BUTTON_GAP, BUTTON_WIDTH};
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

/// The code that turns a place on the picture into one on the land works
/// from these constants, so the markup has to place things by them (less
/// the parchment's edge).
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

/// `floors::column` places the buttons by these sizes.
#[test]
fn the_floor_buttons_are_the_size_the_search_expects() {
    let source = FULL_MAP.source;
    assert!(has(source, "go", "width", BUTTON_WIDTH));
    assert!(has(source, "go", "height", BUTTON));
    // (A `tight` div's gap.)
    assert_eq!(BUTTON_GAP, 4.0);
    assert!(source.contains(r#"<div id="floors" form="full_map" class="tight""#));
}
