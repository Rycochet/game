use super::*;

#[test]
fn render_scale_moves_in_steps() {
    assert_eq!(scale_at(0.0), 50);
    assert_eq!(scale_at(1.0), 100);
    assert_eq!(scale_at(0.5), 75);
    assert_eq!(scale_at(0.51), 75);
    assert_eq!(scale_at(0.56), 80);
    assert_eq!(scale_at(-1.0), 50);
}

#[test]
fn a_preset_sets_everything_and_a_change_leaves_it() {
    let mut prefs = Preset::Low.prefs();
    choose(&mut prefs, VideoControl::Preset, 2);
    assert_eq!(prefs, Preset::High.prefs());
    choose(&mut prefs, VideoControl::VSync, 1);
    assert!(!prefs.vsync);
    assert_eq!(prefs.preset(), None);
    // "Custom", after the presets, changes nothing.
    choose(&mut prefs, VideoControl::Preset, Preset::ALL.len());
    assert!(!prefs.vsync);
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
