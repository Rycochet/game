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
    let mut prefs = GraphicsPrefs::default();
    assert_eq!(
        slide(&mut prefs, GraphicsControl::Brightness, 1.0),
        Some(49)
    );
    assert_eq!(prefs.brightness, 49);
    // The same place again changes nothing.
    assert_eq!(slide(&mut prefs, GraphicsControl::Brightness, 1.0), None);
    assert_eq!(slide(&mut prefs, GraphicsControl::Shadows, 1.0), None);
}

#[test]
fn a_preset_sets_everything_but_the_players_own() {
    let mut prefs = GraphicsPrefs {
        brightness: 20,
        reduce_motion: true,
        ..Preset::Low.prefs()
    };
    choose(&mut prefs, GraphicsControl::Preset, 2);
    assert_eq!(prefs.preset(), Some(Preset::High));
    assert_eq!((prefs.brightness, prefs.reduce_motion), (20, true));
    *switched(&mut prefs, GraphicsControl::VSync).unwrap() = false;
    assert!(!prefs.vsync);
    assert_eq!(prefs.preset(), None);
    // "Custom", after the presets, changes nothing.
    choose(&mut prefs, GraphicsControl::Preset, Preset::ALL.len());
    assert!(!prefs.vsync);
}

#[test]
fn switches_set_their_own_setting() {
    let mut prefs = GraphicsPrefs::default();
    *switched(&mut prefs, GraphicsControl::Clouds).unwrap() = false;
    assert!(!prefs.clouds);
    *switched(&mut prefs, GraphicsControl::ReduceMotion).unwrap() = true;
    assert!(prefs.reduce_motion);
    assert!(switched(&mut prefs, GraphicsControl::Shadows).is_none());
    // Switches aren't dropdowns: a choice changes nothing.
    choose(&mut prefs, GraphicsControl::Clouds, 0);
    assert!(!prefs.clouds);
    choose(&mut prefs, GraphicsControl::Remembered, 2);
    assert_eq!(prefs.remembered, Remembered::Hide);
}

#[test]
fn the_page_is_built_again_once_the_slider_is_let_go() {
    let prefs = GraphicsPrefs::default();
    let shown = GraphicsShown {
        prefs: Some(prefs),
        stale: true,
    };
    assert!(shown.needs_showing(&prefs, false));
    assert!(!shown.needs_showing(&prefs, true));
    let fresh = GraphicsShown {
        prefs: Some(prefs),
        stale: false,
    };
    assert!(!fresh.needs_showing(&prefs, false));
    assert!(GraphicsShown::default().needs_showing(&prefs, true));
}
