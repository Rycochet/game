//! A site's floors other than the one the player is on, as far as they are
//! known, and how floors are numbered where the player is.

use std::collections::BTreeMap;

use bevy::prelude::*;
use shared::protocol::{LevelInfo, RememberedLevel};

use super::CurrentDungeon;

/// The number of the first floor (level 0): 0 where floors are counted from a
/// ground floor, otherwise 1.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirstFloor(pub u32);

impl Default for FirstFloor {
    fn default() -> Self {
        Self(sys_locale::get_locale().map_or(1, |locale| first_floor_in(&locale)))
    }
}

/// Countries counting from a ground floor (owner's list): the EU, the UK and
/// Latin America, where 419 is the code for all of Latin America.
const GROUND_FLOOR_REGIONS: [&str; 50] = [
    "AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR", "DE", "GR", "HU", "IE", "IT", "LV",
    "LT", "LU", "MT", "NL", "PL", "PT", "RO", "SK", "SI", "ES", "SE", "GB", "AR", "BO", "BR", "CL",
    "CO", "CR", "CU", "DO", "EC", "SV", "GT", "HT", "HN", "MX", "NI", "PA", "PY", "PE", "PR", "UY",
    "VE", "419",
];

/// With no country in the locale (browsers often give just "fr"): languages
/// spoken mostly in those countries. English isn't one of them.
const GROUND_FLOOR_LANGUAGES: [&str; 23] = [
    "bg", "cs", "da", "de", "el", "es", "et", "fi", "fr", "ga", "hr", "hu", "it", "lt", "lv", "mt",
    "nl", "pl", "pt", "ro", "sk", "sl", "sv",
];

/// The first floor's number in a locale such as "en-GB", "es-419", "fr" or
/// "zh-Hant-TW".
fn first_floor_in(locale: &str) -> u32 {
    // (Unix locales are written "en_GB.UTF-8".)
    let locale = locale.split('.').next().unwrap_or_default();
    let mut parts = locale.split(['-', '_']);
    let language = parts.next().unwrap_or_default().to_lowercase();
    let is_region = |part: &&str| {
        part.len() == 2 && part.chars().all(|c| c.is_ascii_alphabetic())
            || part.len() == 3 && part.chars().all(|c| c.is_ascii_digit())
    };
    let ground_floor = match parts.find(is_region) {
        Some(region) => GROUND_FLOOR_REGIONS.contains(&region.to_uppercase().as_str()),
        None => GROUND_FLOOR_LANGUAGES.contains(&language.as_str()),
    };
    if ground_floor { 0 } else { 1 }
}

/// The other levels of the current site that the player knows, for the maps,
/// and which one they are looking at. Changed by `site`, as the server tells.
#[derive(Resource, Default)]
pub struct OtherLevels {
    levels: BTreeMap<u32, CurrentDungeon>,
    showing: Option<u32>,
}

impl OtherLevels {
    /// The player is arriving on a level: the maps go back to it. Coming by
    /// stairs, the level left joins the others; into a site, the server sends
    /// them all.
    pub fn entering(&mut self, current: &mut CurrentDungeon, info: &LevelInfo) {
        self.showing = None;
        if info.entering_site {
            self.levels.clear();
        } else if let Some(left) = current.info.as_ref().map(|info| info.index) {
            let level = std::mem::take(current);
            if level.remembers_anything() {
                self.levels.insert(left, level);
            }
        }
        self.levels.remove(&info.index);
    }

    /// What the server says the player remembers of another level.
    pub fn add(&mut self, current: &CurrentDungeon, remembered: &RememberedLevel) {
        let Some(info) = &current.info else {
            return;
        };
        let mut level = CurrentDungeon::default();
        level.enter(LevelInfo {
            index: remembered.index,
            bounds: remembered.bounds,
            ..info.clone()
        });
        for chunk in &remembered.chunks {
            level.add(chunk);
        }
        self.levels.insert(remembered.index, level);
    }

    /// The floor being looked at, if not the player's.
    pub fn showing(&self) -> Option<u32> {
        self.showing
    }

    /// Looks at this floor. (The player's own isn't among the others.)
    pub fn show(&mut self, floor: u32) {
        self.showing = Some(floor).filter(|floor| self.levels.contains_key(floor));
    }

    /// Looks at the player's own floor.
    pub fn show_own(&mut self) {
        self.showing = None;
    }

    /// The level the maps should draw.
    pub fn shown<'a>(&'a self, current: &'a CurrentDungeon) -> &'a CurrentDungeon {
        self.showing
            .and_then(|floor| self.levels.get(&floor))
            .unwrap_or(current)
    }

    /// The floors known besides the player's.
    pub fn floors(&self) -> impl Iterator<Item = u32> + '_ {
        self.levels.keys().copied()
    }

    /// The place on the shown level that is where the player is on theirs: the
    /// same cell in a building, whose floors line up; the same part of the
    /// level in a cave, whose levels are each their own size.
    pub fn spot(&self, current: &CurrentDungeon, player: (f64, f64)) -> (f64, f64) {
        let shown = self.shown(current);
        let (Some(from), Some(to)) = (&current.info, &shown.info) else {
            return player;
        };
        if from.building {
            return player;
        }
        let across = |at: f64, from_min: i64, from_size: i64, to_min: i64, to_size: i64| {
            let part = (at - from_min as f64) / from_size.max(1) as f64;
            to_min as f64 + part * to_size as f64
        };
        (
            across(
                player.0,
                from.bounds.0,
                from.bounds.2,
                to.bounds.0,
                to.bounds.2,
            ),
            across(
                player.1,
                from.bounds.1,
                from.bounds.3,
                to.bounds.1,
                to.bounds.3,
            ),
        )
    }

    #[cfg(test)]
    pub fn insert_for_tests(&mut self, index: u32, level: CurrentDungeon) {
        self.levels.insert(index, level);
    }
}

#[cfg(test)]
mod tests {
    use shared::poi::PoiKind;
    use shared::protocol::LevelChunk;

    use super::*;

    fn info(index: u32, bounds: (i64, i64, i64, i64), entering_site: bool) -> LevelInfo {
        LevelInfo {
            kind: PoiKind::RockCave,
            ruined: false,
            building: false,
            index,
            levels: 3,
            bounds,
            arrival: (1, 1),
            entering_site,
        }
    }

    #[test]
    fn floors_count_from_nought_in_the_eu_the_uk_and_latin_america() {
        for locale in [
            "en-GB",
            "en_GB.UTF-8",
            "fr-FR",
            "de",
            "es-419",
            "es-MX",
            "pt-BR",
            "en-IE",
            "sr-Latn-HR",
        ] {
            assert_eq!(first_floor_in(locale), 0, "{locale}");
        }
        for locale in ["en-US", "en", "es-US", "fr-CA", "ja-JP", "zh-Hant-TW", ""] {
            assert_eq!(first_floor_in(locale), 1, "{locale}");
        }
    }

    fn remembered(index: u32, bounds: (i64, i64, i64, i64)) -> RememberedLevel {
        RememberedLevel {
            index,
            bounds,
            chunks: vec![LevelChunk {
                chunk: (0, 0),
                mask: 1,
                cells: vec![shared::dungeon::Cell::Floor],
                stairs: Vec::new(),
                remembered: true,
            }],
        }
    }

    #[test]
    fn levels_are_kept_on_taking_stairs_and_forgotten_on_coming_in_again() {
        let mut current = CurrentDungeon::default();
        let mut others = OtherLevels::default();
        let bounds = (0, 0, 8, 8);
        others.entering(&mut current, &info(0, bounds, true));
        current.enter(info(0, bounds, true));
        // Nothing seen, nothing known: it isn't kept.
        others.entering(&mut current, &info(1, bounds, false));
        current.enter(info(1, bounds, false));
        assert_eq!(others.floors().count(), 0);

        others.add(&current, &remembered(2, bounds));
        current.remembered.insert(1, 1);
        others.entering(&mut current, &info(2, bounds, false));
        current.enter(info(2, bounds, false));
        // The floor left is kept; the one arrived on is the current one now.
        assert_eq!(others.floors().collect::<Vec<_>>(), [1]);

        others.show(1);
        assert_eq!(others.showing(), Some(1));
        assert!(others.shown(&current).remembered.is_seen(1, 1));
        others.show(2);
        assert_eq!(others.showing(), None);

        others.entering(&mut current, &info(0, bounds, true));
        assert_eq!(others.floors().count(), 0);
    }

    #[test]
    fn the_players_place_carries_across_by_cell_or_by_share_of_the_level() {
        let mut current = CurrentDungeon::default();
        current.enter(info(0, (0, 0, 10, 20), true));
        let mut others = OtherLevels::default();
        others.add(&current, &remembered(1, (-10, 100, 40, 40)));
        assert_eq!(others.spot(&current, (5.0, 5.0)), (5.0, 5.0));
        others.show(1);
        assert_eq!(others.spot(&current, (5.0, 5.0)), (10.0, 110.0));
        current.info.as_mut().expect("entered").building = true;
        assert_eq!(others.spot(&current, (5.0, 5.0)), (5.0, 5.0));
    }
}
