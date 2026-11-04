use super::*;

#[test]
fn render_scale_moves_in_steps() {
    let scale = |value| percent_at(value, RENDER_SCALES, SCALE_STEP);
    assert_eq!(scale(0.0), 50);
    assert_eq!(scale(1.0), 100);
    assert_eq!(scale(0.5), 75);
    assert_eq!(scale(0.51), 75);
    assert_eq!(scale(0.56), 80);
    assert_eq!(scale(-1.0), 50);
}

#[test]
fn brightness_stops_below_half() {
    let mut prefs = VideoPrefs::default();
    assert_eq!(slide(&mut prefs, VideoControl::Brightness, 1.0), Some(49));
    assert_eq!(prefs.brightness, 49);
    // The same place again changes nothing.
    assert_eq!(slide(&mut prefs, VideoControl::Brightness, 1.0), None);
    assert_eq!(slide(&mut prefs, VideoControl::Shadows, 1.0), None);
}

#[test]
fn a_preset_sets_everything_but_the_players_own() {
    let mut prefs = VideoPrefs {
        brightness: 20,
        reduce_motion: true,
        ..Preset::Low.prefs()
    };
    choose(&mut prefs, VideoControl::Preset, 2);
    assert_eq!(prefs.preset(), Some(Preset::High));
    assert_eq!((prefs.brightness, prefs.reduce_motion), (20, true));
    choose(&mut prefs, VideoControl::VSync, 1);
    assert!(!prefs.vsync);
    assert_eq!(prefs.preset(), None);
    // "Custom", after the presets, changes nothing.
    choose(&mut prefs, VideoControl::Preset, Preset::ALL.len());
    assert!(!prefs.vsync);
}

#[test]
fn on_off_lists_on_first() {
    let mut prefs = VideoPrefs::default();
    choose(&mut prefs, VideoControl::Clouds, 1);
    assert!(!prefs.clouds);
    choose(&mut prefs, VideoControl::Clouds, 0);
    assert!(prefs.clouds);
    choose(&mut prefs, VideoControl::Remembered, 2);
    assert_eq!(prefs.remembered, Remembered::Hide);
}

#[test]
fn the_page_is_built_again_once_the_slider_is_let_go() {
    let prefs = VideoPrefs::default();
    let shown = VideoShown {
        prefs: Some(prefs),
        stale: true,
    };
    assert!(shown.needs_building(&prefs, false));
    assert!(!shown.needs_building(&prefs, true));
    let fresh = VideoShown {
        prefs: Some(prefs),
        stale: false,
    };
    assert!(!fresh.needs_building(&prefs, false));
    assert!(VideoShown::default().needs_building(&prefs, true));
}
