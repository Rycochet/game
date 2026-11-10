use super::*;

#[test]
fn every_preset_is_recognised_and_medium_is_the_default() {
    for preset in Preset::ALL {
        assert_eq!(preset.prefs().preset(), Some(preset));
    }
    assert_eq!(GraphicsPrefs::default().preset(), Some(Preset::Medium));
}

#[test]
fn changed_settings_are_no_preset() {
    let prefs = GraphicsPrefs {
        render_scale: 90,
        ..GraphicsPrefs::default()
    };
    assert_eq!(prefs.preset(), None);
}

#[test]
fn presets_leave_the_players_own_settings_alone() {
    let mine = GraphicsPrefs {
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
    let prefs = GraphicsPrefs {
        brightness: 90,
        ..GraphicsPrefs::default()
    };
    assert_eq!(prefs.least_light(), 0.49);
}

#[test]
fn render_share_stays_in_range() {
    let share = |render_scale| {
        GraphicsPrefs {
            render_scale,
            ..GraphicsPrefs::default()
        }
        .render_share()
    };
    assert_eq!(share(100), 1.0);
    assert_eq!(share(75), 0.75);
    assert_eq!(share(10), 0.5);
    assert_eq!(share(200), 1.0);
}

#[test]
fn max_resolution_holds_tall_windows_to_its_lines() {
    let prefs = GraphicsPrefs {
        render_scale: 100,
        max_resolution: MaxResolution::P1080,
        ..GraphicsPrefs::default()
    };
    assert_eq!(prefs.drawn_share(2160), 0.5);
    assert_eq!(prefs.drawn_share(1080), 1.0);
    assert_eq!(prefs.drawn_share(720), 1.0);
    let off = GraphicsPrefs {
        max_resolution: MaxResolution::Off,
        ..prefs
    };
    assert_eq!(off.drawn_share(4320), 1.0);
}

#[test]
fn render_scale_still_applies_below_max_resolution() {
    let prefs = GraphicsPrefs {
        render_scale: 50,
        max_resolution: MaxResolution::P1440,
        ..GraphicsPrefs::default()
    };
    // 1440 of 1620 lines would be 0.89: Render Scale's half is less.
    assert_eq!(prefs.drawn_share(1620), 0.5);
}
