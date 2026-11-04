//! The Video page's sections for what is drawn: Lighting, World Map, Sites and
//! Motion. Their rows are made as in `options`.

use bevy::prelude::*;

use super::super::rows::spawn_section;
use super::options::{VideoControl, dropdown, on_off, setting_row, slider_row};
use crate::common::{
    BRIGHTNESS_MOST, Precipitation, Remembered, Shadows, VideoPrefs, ViewDistance,
};

const BRIGHTNESS: &str = "Raises the darkest parts of what you can see, at night and \
    underground, up to just under half of daylight. It never shows anything your \
    character can't see. Little difference to speed.";
const SHADOWS: &str = "Shadows cast by the sun or moons on the world map. Low has softer \
    edges; Off saves graphics power. A medium difference.";
const MIDDAY_SUN: &str = "Lights the world map as at noon, by the main sun alone, \
    whatever the time, so its light never moves. Sites still follow the time of day. \
    A light difference.";
const VIEW_DISTANCE: &str = "How much land is drawn round you on the world map. Near \
    draws less; Far keeps all of it drawn when you zoom in. A medium difference, most \
    while travelling.";
const CLOUDS: &str = "Shows the cloud layer above the world map. With Shadows on, clouds \
    also darken the ground below them. A light difference.";
const WEATHER: &str = "Rain, snow and hail falling, on the world map and in places open \
    to the sky. Light draws half as much. A light difference.";
const FOLIAGE: &str = "Paints trees and low growth onto the world map's land. A light \
    difference.";
const FLOOR_TEXTURES: &str = "Draws floors and walls in sites with their pictures, or in \
    plain colours that make the grid and what lies on it easier to read. Little \
    difference to speed.";
const REMEMBERED: &str = "How places you have seen, but can't see now, are drawn in \
    sites: as they are, darker, or not at all. Little difference to speed.";
const REDUCE_MOTION: &str = "Stills the world map's water, and makes tokens and the \
    camera jump to their places instead of gliding. For anyone who finds movement \
    uncomfortable. Little difference to speed.";

pub(super) fn spawn_lighting_section(sections: &mut ChildSpawnerCommands, prefs: &VideoPrefs) {
    spawn_section(sections, "Lighting", |section| {
        let at = (0, BRIGHTNESS_MOST, prefs.brightness.min(BRIGHTNESS_MOST));
        slider_row(
            section,
            VideoControl::Brightness,
            "Brightness",
            BRIGHTNESS,
            at,
        );
        let list = dropdown(&Shadows::ALL, Shadows::name, prefs.shadows, Vec::new());
        setting_row(
            section,
            VideoControl::Shadows,
            "Shadows",
            (SHADOWS, None),
            list,
        );
        let list = on_off(prefs.midday_sun);
        let row = (MIDDAY_SUN, None);
        setting_row(section, VideoControl::MiddaySun, "Midday Sun", row, list);
    });
}

pub(super) fn spawn_world_map_section(sections: &mut ChildSpawnerCommands, prefs: &VideoPrefs) {
    spawn_section(sections, "World Map", |section| {
        let all = &ViewDistance::ALL;
        let list = dropdown(all, ViewDistance::name, prefs.view_distance, Vec::new());
        let row = (VIEW_DISTANCE, None);
        setting_row(
            section,
            VideoControl::ViewDistance,
            "View Distance",
            row,
            list,
        );
        let list = on_off(prefs.clouds);
        setting_row(
            section,
            VideoControl::Clouds,
            "Clouds",
            (CLOUDS, None),
            list,
        );
        let all = &Precipitation::ALL;
        let list = dropdown(all, Precipitation::name, prefs.precipitation, Vec::new());
        setting_row(
            section,
            VideoControl::Precipitation,
            "Weather",
            (WEATHER, None),
            list,
        );
        let list = on_off(prefs.foliage);
        setting_row(
            section,
            VideoControl::Foliage,
            "Foliage",
            (FOLIAGE, None),
            list,
        );
    });
}

pub(super) fn spawn_sites_section(sections: &mut ChildSpawnerCommands, prefs: &VideoPrefs) {
    spawn_section(sections, "Sites", |section| {
        let list = on_off(prefs.floor_textures);
        let row = (FLOOR_TEXTURES, None);
        setting_row(
            section,
            VideoControl::FloorTextures,
            "Floor Textures",
            row,
            list,
        );
        let list = dropdown(
            &Remembered::ALL,
            Remembered::name,
            prefs.remembered,
            Vec::new(),
        );
        let row = (REMEMBERED, None);
        setting_row(
            section,
            VideoControl::Remembered,
            "Remembered Places",
            row,
            list,
        );
    });
}

pub(super) fn spawn_motion_section(sections: &mut ChildSpawnerCommands, prefs: &VideoPrefs) {
    spawn_section(sections, "Motion", |section| {
        let list = on_off(prefs.reduce_motion);
        let row = (REDUCE_MOTION, None);
        setting_row(
            section,
            VideoControl::ReduceMotion,
            "Reduce Motion",
            row,
            list,
        );
    });
}
