//! The site level the player is on, as far as they know it.

use bevy::prelude::*;
use shared::dungeon::{Cell, Level, RoomId, Sight};
use shared::poi::PoiKind;
use shared::protocol::{LevelChunk, LevelInfo, LevelKind, SeenStairs};
use shared::site::{BuildingLook, FieldLook};

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
    /// A town's buildings (none anywhere else).
    pub buildings: Vec<BuildingLook>,
    /// A farm's fields (none anywhere else).
    pub fields: Vec<FieldLook>,
    /// In the wilderness, the kind of poi set into it that the player stands
    /// in, as the server says (`ServerMessage::InPoi`).
    pub in_poi: Option<PoiKind>,
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
            buildings: Vec::new(),
            fields: Vec::new(),
            in_poi: None,
        }
    }
}

impl CurrentDungeon {
    /// Arriving on a level: nothing of it known yet.
    pub fn enter(&mut self, info: LevelInfo) {
        let (min_x, min_y, width, height) = info.bounds;
        self.level = Level::new(min_x, min_y, width, height);
        self.known = Sight::new(&self.level);
        self.remembered = Sight::new(&self.level);
        self.stairs.clear();
        self.buildings.clear();
        self.fields.clear();
        self.in_poi = None;
        self.info = Some(info);
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
    }

    /// Whether any of the level has been seen.
    pub fn remembers_anything(&self) -> bool {
        let (min_x, min_y, width, height) = self.level.bounds();
        (min_y..min_y + height)
            .any(|y| (min_x..min_x + width).any(|x| self.remembered.is_seen(x, y)))
    }

    /// Where the player is, for the HUD, e.g. "Castle, floor 2 of 3", or
    /// "Wilderness: RockCave" in a poi set into the wilderness.
    pub fn describe(&self) -> String {
        let name = self.floor_name(FirstFloor(1));
        if let Some(kind) = self.in_poi.filter(|_| self.is_wilderness()) {
            return format!("{name}: {kind:?}");
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
            return format!("{kind:?}");
        }
        let what = if info.building { "floor" } else { "level" };
        let ruined = if info.ruined { "Ruined" } else { "" };
        if info.storey(info.index) < 0 {
            return format!("{ruined}{kind:?}, basement");
        }
        format!("{ruined}{kind:?}, {what} {}", info.index + first.0)
    }
}
