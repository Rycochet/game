//! The site level the player is on, as far as they know it.

use bevy::prelude::*;
use shared::dungeon::{Cell, Level, RoomId, Sight};
use shared::movement::FlatGround;
use shared::poi::PoiKind;
use shared::protocol::{LevelChunk, LevelInfo, LevelKind, SeenStairs};
use shared::site::{BuildingLook, FieldLook};
use std::collections::BTreeSet;

use super::floors::FirstFloor;

/// The site level the player is on, as far as the server has told: unknown
/// cells are wall until their cells arrive. Its rules (walls block
/// movement, sight and light) live in `shared::dungeon`. Changed by `site`
/// when the server says the player has arrived, or sends cells.
#[derive(Resource)]
pub struct CurrentDungeon {
    /// What the level is; `None` before any has been entered.
    pub info: Option<LevelInfo>,
    /// The cells the server has sent; the rest are wall.
    pub level: Level,
    /// Which cells the server has sent.
    pub known: Sight,
    /// Which cells the player has seen, now or before: drawn even when out
    /// of sight (in grey). Includes what is in sight now.
    pub remembered: Sight,
    /// The level's known stairs.
    pub stairs: Vec<SeenStairs>,
    /// The doors seen to have a lock (whether they are locked is on `level`,
    /// once found by trying one).
    pub door_locks: std::collections::BTreeSet<(i64, i64)>,
    /// A town's buildings (none anywhere else).
    pub buildings: Vec<BuildingLook>,
    /// A farm's fields (none anywhere else).
    pub fields: Vec<FieldLook>,
    /// Walls round the level's buildings: a walled fort's (none anywhere else).
    pub walls: Vec<shared::site::WallLook>,
    /// In the wilderness, the kind of poi set into it that the player stands
    /// in, as the server says (`ServerMessage::InPoi`).
    pub in_poi: Option<PoiKind>,
    /// What that poi is called, if it has a name of its own (a town's,
    /// castle's or fort's).
    pub in_poi_name: Option<String>,
    /// In the wilderness, the cells round the cutouts of pois the player
    /// knows (`ServerMessage::Borders`): drawn hatched, and a click on one
    /// walks there and leaves for the world map.
    pub borders: BTreeSet<(i64, i64)>,
    /// In the wilderness, the ground kept level that the server has told of
    /// (`ServerMessage::FlatGround`): pois' cutouts.
    pub flats: Vec<FlatGround>,
}

impl Default for CurrentDungeon {
    fn default() -> Self {
        let level = Level::new(0, 0, 1, 1);
        Self {
            info: None,
            known: Sight::new(&level),
            remembered: Sight::new(&level),
            level,
            stairs: Vec::new(),
            door_locks: Default::default(),
            buildings: Vec::new(),
            fields: Vec::new(),
            walls: Vec::new(),
            in_poi: None,
            in_poi_name: None,
            borders: BTreeSet::new(),
            flats: Vec::new(),
        }
    }
}

impl CurrentDungeon {
    /// Arriving on a level: nothing of it known yet.
    pub fn enter(&mut self, info: LevelInfo) {
        let (min_x, min_y, width, height) = info.bounds;
        self.level = Level::new(min_x, min_y, width, height);
        // As the server's copy: natural walls block wider at the foot.
        let natural = info.kind.poi().is_some_and(|kind| kind.natural_walls());
        self.level.set_natural_walls(natural);
        self.known = Sight::new(&self.level);
        self.remembered = Sight::new(&self.level);
        self.stairs.clear();
        self.door_locks.clear();
        self.buildings.clear();
        self.fields.clear();
        self.walls.clear();
        self.in_poi = None;
        self.in_poi_name = None;
        self.borders.clear();
        self.flats.clear();
        self.info = Some(info);
    }

    /// Ground kept level, told by the server: each kept once.
    pub fn add_flats(&mut self, flats: &[FlatGround]) {
        for flat in flats {
            if !self.flats.contains(flat) {
                self.flats.push(*flat);
            }
        }
    }

    /// The wilderness's level is now `bounds` (`ServerMessage::WildernessWindow`):
    /// what is known of it where the two overlap stays.
    pub fn move_window(&mut self, bounds: (i64, i64, i64, i64)) {
        let (min_x, min_y, width, height) = bounds;
        let mut level = Level::new(min_x, min_y, width, height);
        let mut known = Sight::new(&level);
        let mut remembered = Sight::new(&level);
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                if self.known.is_seen(x, y) {
                    level.set(x, y, self.level.cell(x, y));
                    level.set_room(x, y, self.level.room(x, y));
                    // A door with no swing counts as open, so how each
                    // hangs must come along too: the server won't send
                    // cells the client already has again.
                    if self.level.is_locked(x, y) {
                        level.set_locked(x, y, true);
                    }
                    if let Some(swing) = self.level.swing(x, y) {
                        level.set_swing(x, y, swing);
                    }
                    known.insert(x, y);
                }
                if self.remembered.is_seen(x, y) {
                    remembered.insert(x, y);
                }
            }
        }
        (self.level, self.known, self.remembered) = (level, known, remembered);
        let level = &self.level;
        self.stairs
            .retain(|stairs| level.contains(stairs.x, stairs.y));
        if let Some(info) = &mut self.info {
            info.bounds = bounds;
        }
    }

    /// Whether the player is in the wilderness.
    pub fn is_wilderness(&self) -> bool {
        self.info
            .as_ref()
            .is_some_and(|info| info.kind == LevelKind::Wilderness)
    }

    /// A town's buildings: their cells are marked as theirs on the level.
    pub fn add_buildings(&mut self, buildings: &[BuildingLook]) {
        for building in buildings {
            for rect in &building.rects {
                for y in rect.y..rect.y + rect.height {
                    for x in rect.x..rect.x + rect.width {
                        self.level.set_room(x, y, Some(RoomId(building.id)));
                    }
                }
            }
        }
        self.buildings = buildings.to_vec();
    }

    /// Whether the level is a town.
    pub fn level_has(&self, cell: Cell) -> bool {
        let (min_x, min_y, max_x, max_y) = match self.info.as_ref() {
            Some(info) => info.bounds,
            None => return false,
        };
        (min_y..=max_y).any(|y| (min_x..=max_x).any(|x| self.level.cell(x, y) == cell))
    }

    /// Whether any cell known on this level is out in the open, under the sky.
    pub fn knows_outside(&self) -> bool {
        let (min_x, min_y, width, height) = self.level.bounds();
        (min_y..min_y + height).any(|y| {
            (min_x..min_x + width)
                .any(|x| self.known.is_seen(x, y) && self.level.cell(x, y).is_outside())
        })
    }

    pub fn is_town(&self) -> bool {
        self.info.as_ref().is_some_and(|info| info.kind.is_town())
    }

    /// The building whose door faces a cell, if any.
    pub fn door_facing(&self, x: i64, y: i64) -> Option<&BuildingLook> {
        self.buildings
            .iter()
            .find(|b| (b.door.0 + b.faces.0, b.door.1 + b.faces.1) == (x, y))
    }

    /// Cells the server has sent.
    pub fn add(&mut self, chunk: &LevelChunk) {
        for (x, y, cell) in chunk.cells() {
            self.level.set(x, y, cell);
            self.known.insert(x, y);
            if chunk.remembered {
                self.remembered.insert(x, y);
            }
        }
        for stairs in &chunk.stairs {
            if !self.stairs.contains(stairs) {
                self.stairs.push(*stairs);
            }
        }
        for door in &chunk.doors {
            self.level.set_swing(door.x, door.y, door.swing);
            // Seen open, or with no lock, it isn't locked (any more).
            if !door.lock || !door.swing.is_shut() {
                self.level.set_locked(door.x, door.y, false);
            }
            if door.lock {
                self.door_locks.insert((door.x, door.y));
            } else {
                self.door_locks.remove(&(door.x, door.y));
            }
        }
    }

    /// Whether any of the level has been seen.
    pub fn remembers_anything(&self) -> bool {
        let (min_x, min_y, width, height) = self.level.bounds();
        (min_y..min_y + height)
            .any(|y| (min_x..min_x + width).any(|x| self.remembered.is_seen(x, y)))
    }

    /// Where the player is, for the HUD, e.g. "Castle, floor 2 of 3", or
    /// "Wilderness: RockCave" in a poi set into the wilderness ("Wilderness:
    /// Thornbury" in a town, by its name; "Castle Ravenhold, floor 2 of 3").
    /// A named place is written as it is, not in code style.
    pub fn describe(&self) -> String {
        let name = self.floor_name(FirstFloor(1));
        if let Some(kind) = self.in_poi.filter(|_| self.is_wilderness()) {
            return match &self.in_poi_name {
                Some(poi_name) => format!("{name}: {poi_name}"),
                None => format!("{name}: {kind:?}"),
            };
        }
        match &self.info {
            Some(info)
                if info.kind.poi().is_some_and(|kind| !kind.is_town())
                    && info.storey(info.index) >= 0 =>
            {
                format!("{name} of {}", info.levels)
            }
            _ => name,
        }
    }

    /// What the level is called, e.g. "Castle, floor 2", counting floors from
    /// `first`.
    pub fn floor_name(&self, first: FirstFloor) -> String {
        let Some(info) = &self.info else {
            return "Nowhere".into();
        };
        let kind = match info.kind {
            LevelKind::Poi(kind) => kind,
            LevelKind::Hall { .. } => return "Hall of Worlds".into(),
            LevelKind::Wilderness => return "Wilderness".into(),
        };
        if kind.is_town() {
            return info.place_name.clone().unwrap_or(format!("{kind:?}"));
        }
        let what = if info.building { "floor" } else { "level" };
        let place = match &info.place_name {
            Some(name) if info.ruined => format!("Ruined {name}"),
            Some(name) => name.clone(),
            None if info.ruined => format!("Ruined{kind:?}"),
            None => format!("{kind:?}"),
        };
        if info.storey(info.index) < 0 {
            return format!("{place}, basement");
        }
        format!("{place}, {what} {}", info.index + first.0)
    }
}

#[cfg(test)]
mod tests;
