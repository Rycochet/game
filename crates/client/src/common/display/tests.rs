use super::*;

fn mode(width: u32, height: u32, hertz: u32) -> VideoMode {
    VideoMode {
        physical_size: UVec2::new(width, height),
        bit_depth: 32,
        refresh_rate_millihertz: hertz * 1000,
    }
}

#[test]
fn resolutions_are_largest_first_at_their_fastest() {
    let modes = [
        mode(1280, 720, 60),
        mode(1920, 1080, 60),
        mode(1920, 1080, 144),
        mode(1280, 720, 120),
    ];
    let found = resolutions(&modes);
    assert_eq!(found.len(), 2);
    assert_eq!(
        (found[0].width, found[0].refresh_millihertz),
        (1920, 144_000)
    );
    assert_eq!(
        (found[1].width, found[1].refresh_millihertz),
        (1280, 120_000)
    );
    assert_eq!(found[0].label(), "1920 × 1080, 144 Hz");
}

#[test]
fn the_closest_mode_is_the_same_size_then_the_nearest() {
    let modes = [
        mode(1920, 1080, 60),
        mode(1920, 1080, 144),
        mode(1280, 720, 60),
    ];
    let wanted = |width, height, hertz: u32| Resolution {
        width,
        height,
        refresh_millihertz: hertz * 1000,
    };
    let found = |wanted| closest_mode(&modes, wanted).unwrap().physical_size;
    assert_eq!(closest_mode(&modes, wanted(1920, 1080, 60)), Some(modes[0]));
    // No refresh given: the fastest.
    assert_eq!(closest_mode(&modes, wanted(1920, 1080, 0)), Some(modes[1]));
    assert_eq!(found(wanted(1366, 768, 60)), UVec2::new(1280, 720));
    assert_eq!(closest_mode(&[], wanted(800, 600, 60)), None);
}

#[test]
fn alt_enter_goes_to_the_last_full_screen_and_back() {
    let windowed = DisplayChoice::default();
    let full = windowed.toggled(DisplayMode::FullScreen);
    assert_eq!(full.mode, DisplayMode::FullScreen);
    assert_eq!(full.toggled(DisplayMode::FullScreen), windowed);
}

#[test]
fn keeping_a_full_screen_mode_remembers_it_for_alt_enter() {
    let mut saved = WindowPrefs::default();
    assert!(saved.maximised);
    assert_eq!(saved.last_full_screen, DisplayMode::Borderless);
    saved.keep(DisplayChoice {
        mode: DisplayMode::FullScreen,
        resolution: None,
    });
    saved.keep(DisplayChoice::default());
    assert_eq!(saved.display.mode, DisplayMode::Windowed);
    assert_eq!(saved.last_full_screen, DisplayMode::FullScreen);
}
