use bevy::window::WindowResolution;

use super::*;

#[test]
fn the_picture_has_the_windows_logical_size() {
    let mut window = Window {
        resolution: WindowResolution::new(1600, 900),
        ..default()
    };
    window.resolution.set_scale_factor(2.0);
    let (size, scale) = picture_size(&window, 0.5);
    assert_eq!(size, (window.physical_size().as_vec2() * 0.5).as_uvec2());
    assert_eq!(size.as_vec2() / scale, window.size());
}
