use super::*;

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn no_system_asks_for_what_another_of_its_parameters_has() {
    IntoSystem::into_system(give_set_edges).initialize(&mut World::new());
    IntoSystem::into_system(drift_bands).initialize(&mut World::new());
    IntoSystem::into_system(slide_bands).initialize(&mut World::new());
}

#[test]
fn an_edge_stands_out_only_on_the_sides_it_has() {
    let border = UiRect {
        left: px(3),
        right: px(0),
        top: px(3),
        bottom: px(0),
    };
    let node = edge_node(border, 3.0);
    assert_eq!((node.left, node.top), (px(-3), px(-3)));
    assert_eq!((node.right, node.bottom), (px(0), px(0)));
    assert_eq!(node.border, border);
}

#[test]
fn an_edge_of_all_sides_stands_out_all_round() {
    let node = edge_node(UiRect::all(px(2)), 2.0);
    let sides = [node.left, node.top, node.right, node.bottom];
    assert!(sides.iter().all(|&side| side == px(-2)));
}

#[test]
fn the_bands_drift_slowly_and_stay_within_one_band() {
    let after_a_second = drifted(0.0, 1.0);
    assert!(after_a_second > 0.0 && after_a_second < BAND_PIXELS / 2.0);
    let wrapped = drifted(BAND_PIXELS - 1.0, 1.0);
    assert!((0.0..BAND_PIXELS).contains(&wrapped));
}
