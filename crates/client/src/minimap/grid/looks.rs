//! What each cell in a box looks like on the map, and the shop signs.

use shared::dungeon::Cell;
use shared::site::BuildingKind;

use super::{WHITE, ground_colour};
use crate::common::CurrentDungeon;

/// Which sides of a wall cell are hatched.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sides {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
    /// Corners cut away because the wall only touches another wall there, with
    /// a way between them: `[north-east, south-east, south-west, north-west]`,
    /// each in the colour of the floor it opens onto.
    pub cut: [Option<[u8; 3]>; 4],
}

/// What a cell looks like on the map.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CellLook {
    /// Not remembered, or nothing to show: bare parchment.
    #[default]
    Empty,
    /// Anything that can be walked on or jumped over, in its colour.
    Floor([u8; 3]),
    /// A wall: parchment, hatched along the sides that show it.
    Wall(Sides),
    /// A dungeon's door in a wall running east-west, or north-south.
    Door { east_west: bool },
    /// A secret door, found or seen by truesight: drawn as a door, with an "S"
    /// stamped over it.
    SecretDoor { east_west: bool },
    /// A window in a wall running east-west, or north-south, with the colours
    /// of what it is known to look onto, to the north or west and the south or
    /// east.
    Window {
        east_west: bool,
        sides: [[u8; 3]; 2],
    },
    /// A town house's door, the same way.
    HouseDoor { east_west: bool },
    /// A shop's door: a walkable-looking cell carrying its sign's picture.
    Sign(BuildingKind),
}

/// How the cells in a box look.
pub struct Looks {
    min: (i64, i64),
    size: (i64, i64),
    looks: Vec<CellLook>,
}

impl Looks {
    /// Works out the cells from `min` to `max`, inclusive.
    pub fn of(dungeon: &CurrentDungeon, min: (i64, i64), max: (i64, i64)) -> Self {
        let size = (max.0 - min.0 + 1, max.1 - min.1 + 1);
        let mut looks = Vec::with_capacity((size.0 * size.1) as usize);
        for y in min.1..=max.1 {
            for x in min.0..=max.0 {
                looks.push(look_of(dungeon, x, y));
            }
        }
        Self { min, size, looks }
    }

    pub fn at(&self, x: i64, y: i64) -> CellLook {
        let (column, row) = (x - self.min.0, y - self.min.1);
        if (0..self.size.0).contains(&column) && (0..self.size.1).contains(&row) {
            self.looks[(row * self.size.0 + column) as usize]
        } else {
            CellLook::Empty
        }
    }

    /// Every cell in the box: where, and how it looks.
    fn cells(&self) -> impl Iterator<Item = ((i64, i64), CellLook)> + '_ {
        self.looks.iter().enumerate().map(|(index, look)| {
            let index = index as i64;
            let at = (
                self.min.0 + index % self.size.0,
                self.min.1 + index / self.size.0,
            );
            (at, *look)
        })
    }

    /// The shop signs in the box: where, and whose.
    pub fn signs(&self) -> impl Iterator<Item = ((i64, i64), BuildingKind)> + '_ {
        self.cells().filter_map(|(at, look)| match look {
            CellLook::Sign(kind) => Some((at, kind)),
            _ => None,
        })
    }

    /// Where the secret doors in the box are.
    pub fn secret_doors(&self) -> impl Iterator<Item = (i64, i64)> + '_ {
        self.cells()
            .filter_map(|(at, look)| matches!(look, CellLook::SecretDoor { .. }).then_some(at))
    }
}

pub fn look_of(dungeon: &CurrentDungeon, x: i64, y: i64) -> CellLook {
    if !dungeon.remembered.is_seen(x, y) {
        return CellLook::Empty;
    }
    let level = &dungeon.level;
    // A wall shows where it meets something open, or another building (a
    // town's buildings touch). A town building's door is only a look on its
    // wall, and counts as neither: nothing is hatched beside it.
    let room = level.room(x, y);
    let shows = |x: i64, y: i64| {
        let cell = level.cell(x, y);
        let other_building = cell.is_wall()
            && room.is_some()
            && level.room(x, y).is_some()
            && level.room(x, y) != room;
        dungeon.remembered.is_seen(x, y)
            && (cell.is_walkable() || cell.is_jumpable() || cell == Cell::Window || other_building)
    };
    match level.cell(x, y) {
        Cell::Door => CellLook::Door {
            east_west: level.opening_runs_east_west(x, y),
        },
        // Only sent to truesight: otherwise it arrives as a wall.
        Cell::SecretDoor => CellLook::SecretDoor {
            east_west: level.opening_runs_east_west(x, y),
        },
        Cell::Window => {
            let east_west = level.opening_runs_east_west(x, y);
            let side = |sign: i64| {
                let (sx, sy) = if east_west {
                    (x, y + sign)
                } else {
                    (x + sign, y)
                };
                let cell = level.cell(sx, sy);
                let known = dungeon.remembered.is_seen(sx, sy);
                if known && (cell.is_walkable() || cell.is_jumpable()) {
                    ground_colour(cell)
                } else {
                    WHITE
                }
            };
            CellLook::Window {
                east_west,
                sides: [side(-1), side(1)],
            }
        }
        Cell::BuildingDoor => match dungeon.buildings.iter().find(|b| b.door == (x, y)) {
            Some(building) if building.kind != BuildingKind::House => CellLook::Sign(building.kind),
            // A door facing north or south is in a wall running east-west.
            building => CellLook::HouseDoor {
                east_west: building.is_none_or(|b| b.faces.0 == 0),
            },
        },
        cell if cell.is_walkable() || cell.is_jumpable() => CellLook::Floor(ground_colour(cell)),
        cell if cell.is_wall() => CellLook::Wall(Sides {
            north: shows(x, y - 1),
            east: shows(x + 1, y),
            south: shows(x, y + 1),
            west: shows(x - 1, y),
            cut: [(1, -1), (1, 1), (-1, 1), (-1, -1)].map(|(dx, dy)| {
                let open = |x: i64, y: i64| {
                    let cell = level.cell(x, y);
                    ((cell.is_walkable() || cell.is_jumpable() || cell == Cell::Window)
                        && dungeon.remembered.is_seen(x, y))
                    .then_some(cell)
                };
                let beside = open(x + dx, y)?;
                open(x, y + dy)?;
                let wall = level.cell(x + dx, y + dy);
                (wall.is_wall() && dungeon.remembered.is_seen(x + dx, y + dy))
                    .then(|| ground_colour(beside))
            }),
        }),
        _ => CellLook::Empty,
    }
}
