use super::*;

#[test]
fn zooming_keeps_what_is_under_the_pointer_there() {
    let mut view = SkillsView {
        pan: Vec2::new(30.0, -10.0),
        ..default()
    };
    let pointer = Vec2::new(120.0, 80.0);
    let on_tree = (pointer - view.pan) / view.zoom;
    view.zoom_about(1.5, pointer);
    assert!((view.zoom - 1.5).abs() < 1e-6);
    assert!(view.on_page(on_tree).distance(pointer) < 1e-3);
}

#[test]
fn zoom_stays_within_its_limits() {
    let mut view = SkillsView::default();
    view.zoom_about(100.0, Vec2::ZERO);
    assert_eq!(view.zoom, view.max_zoom);
    view.zoom_about(0.0001, Vec2::ZERO);
    assert_eq!(view.zoom, MIN_ZOOM);
}

#[test]
fn fitting_centres_the_tree_and_shows_all_of_it() {
    let mut view = SkillsView {
        pan: Vec2::new(500.0, 500.0),
        ..default()
    };
    // A tree reaching out to the right and up.
    let extent = (Vec2::new(-50.0, -900.0), Vec2::new(1150.0, 50.0));
    view.fit(Vec2::new(800.0, 600.0), extent);
    let (low, high) = (view.on_page(extent.0), view.on_page(extent.1));
    assert!(((low + high) / 2.0).length() < 1e-3, "centred");
    assert!(high.x - low.x <= 800.0 && high.y - low.y <= 600.0);
    // A small tree isn't blown up past its usual size.
    view.fit(
        Vec2::new(800.0, 600.0),
        (Vec2::splat(-100.0), Vec2::splat(100.0)),
    );
    assert_eq!(view.zoom, 1.0);
}

#[test]
fn zooming_in_stops_when_the_smallest_circles_are_twice_the_fitted_middle() {
    let mut view = SkillsView::default();
    view.fit(
        Vec2::new(800.0, 600.0),
        (Vec2::splat(-1000.0), Vec2::splat(1000.0)),
    );
    let middle = SIZES[0] * view.zoom;
    view.zoom_about(1000.0, Vec2::ZERO);
    let smallest = SIZES[SIZES.len() - 1] * view.zoom;
    assert!(
        (smallest - 2.0 * middle).abs() < 1e-3,
        "{smallest} {middle}"
    );
}
