use super::*;

#[test]
fn auto_keeps_the_interface_its_share_of_the_window() {
    let auto =
        |width, height| interface_scale(InterfaceSize::Auto, false, Vec2::new(width, height));
    assert_eq!(auto(1920.0, 1080.0), 1.0);
    assert_eq!(auto(3840.0, 2160.0), 2.0);
    // The narrower way decides, so panels still fit across.
    assert_eq!(auto(3840.0, 1080.0), 1.0);
    assert_eq!(auto(1920.0, 2160.0), 1.0);
    assert_eq!(auto(960.0, 540.0), 0.5);
    assert_eq!(auto(100.0, 100.0), SMALLEST);
    assert_eq!(auto(10_000.0, 10_000.0), LARGEST);
}

#[test]
fn a_chosen_size_ignores_the_window() {
    let window = Vec2::new(1280.0, 720.0);
    assert_eq!(
        interface_scale(InterfaceSize::Percent(200), false, window),
        2.0
    );
    assert_eq!(
        interface_scale(InterfaceSize::Percent(50), false, window),
        0.5
    );
    assert_eq!(
        interface_scale(InterfaceSize::Percent(1000), false, window),
        LARGEST
    );
}

#[test]
fn touch_never_draws_it_smaller_than_its_own_size() {
    let small = Vec2::new(960.0, 540.0);
    assert_eq!(interface_scale(InterfaceSize::Auto, true, small), 1.0);
    assert_eq!(
        interface_scale(InterfaceSize::Percent(75), true, small),
        1.0
    );
    assert_eq!(
        interface_scale(InterfaceSize::Percent(150), true, small),
        1.5
    );
}
