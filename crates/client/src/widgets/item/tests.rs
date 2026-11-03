use shared::item::{Quality, Shape};

use super::cells::cell_edges;
use super::*;

#[test]
fn a_picture_across_cells_has_an_edge_only_round_the_outside() {
    let edges = cell_edges(Shape::rectangle(2, 2), 0, 0, 2.0);
    assert_eq!((edges.left, edges.top), (px(2), px(2)));
    assert_eq!((edges.right, edges.bottom), (px(0), px(0)));
    let alone = cell_edges(Shape::rectangle(1, 1), 0, 0, 2.0);
    assert_eq!(alone, UiRect::all(px(2)));
}

#[test]
fn quality_makes_an_edge_wider() {
    assert!(item_edge_width(Some(Quality::Poor)) > item_edge_width(None));
}

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn showing_a_name_asks_for_nothing_twice() {
    IntoSystem::into_system(show_item_tooltip).initialize(&mut World::new());
    IntoSystem::into_system(name_item_at_pointer).initialize(&mut World::new());
}
