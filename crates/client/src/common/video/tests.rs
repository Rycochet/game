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
fn presets_leave_the_players_own_settings_alone() {
    let mine = VideoPrefs {
        brightness: 30,
        reduce_motion: true,
        ..Preset::Low.prefs()
    };
    assert_eq!(mine.preset(), Some(Preset::Low));
    let high = Preset::High.over(mine);
    assert_eq!((high.brightness, high.reduce_motion), (30, true));
    assert_eq!(high.preset(), Some(Preset::High));
}

#[test]
fn brightness_stays_below_half() {
    let prefs = VideoPrefs {
        brightness: 90,
        ..VideoPrefs::default()
    };
    assert_eq!(prefs.least_light(), 0.49);
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
