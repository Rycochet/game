use super::*;

#[test]
fn the_frames_are_good_markup() {
    INVENTORY.assert_good();
    BAG.assert_good();
    GROWING_BAG.assert_good();
}

#[test]
fn a_growing_bag_scrolls_past_the_height_the_others_took() {
    // As tall as `ContainerSize::Grows(Val::Vh(60.0))` was.
    assert!(GROWING_BAG.source.contains(r#"max-height="60%""#));
    assert!(!BAG.source.contains("<scroll"));
}
