use super::*;

#[test]
fn every_preset_is_recognised_and_medium_is_the_default() {
    for preset in Preset::ALL {
        assert_eq!(preset.prefs().preset(), Some(preset));
    }
    assert_eq!(VideoPrefs::default().preset(), Some(Preset::Medium));
}

#[test]
fn changed_settings_are_no_preset() {
    let prefs = VideoPrefs {
        render_scale: 90,
        ..VideoPrefs::default()
    };
    assert_eq!(prefs.preset(), None);
}

#[test]
fn render_share_stays_in_range() {
    let share = |render_scale| {
        VideoPrefs {
            render_scale,
            ..VideoPrefs::default()
        }
        .render_share()
    };
    assert_eq!(share(100), 1.0);
    assert_eq!(share(75), 0.75);
    assert_eq!(share(10), 0.5);
    assert_eq!(share(200), 1.0);
}
