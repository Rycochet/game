//! The map of a site level: remembered walkable cells as squares coloured by
//! their ground, with doors and shop signs, and hatching in the wall cells
//! along what they touch. Everything else is left as bare parchment.

use shared::dungeon::Cell;
use shared::site::BuildingKind;

use super::canvas::Canvas;
use super::projection::Projection;
use crate::common::CurrentDungeon;

const WHITE: [u8; 3] = [255, 255, 255];
const LINE: [u8; 3] = [200, 200, 200];
const HATCH: [u8; 3] = [70, 55, 40];
const DOOR: [u8; 3] = [150, 95, 45];
const HOUSE_DOOR: [u8; 3] = [175, 60, 45];
const HOLE: [u8; 3] = [0, 0, 0];

/// A walkable tile's colour, by what it is: pale, so hatching and doors show.
fn ground_colour(cell: Cell) -> [u8; 3] {
    match cell {
        Cell::Ground => [226, 238, 214],
        Cell::Garden => [200, 230, 190],
        Cell::DirtTrack => [222, 205, 180],
        Cell::GravelTrack => [218, 212, 198],
        Cell::WoodenBoards => [232, 212, 178],
        Cell::Cobblestone => [214, 210, 206],
        Cell::Flagstones => [226, 220, 210],
        Cell::DressedStone => [222, 226, 232],
        Cell::Water => [150, 195, 240],
        Cell::Lava => [240, 140, 120],
        Cell::Hole => HOLE,
        Cell::Field => [214, 222, 160],
        Cell::Fence => [150, 110, 70],
        Cell::FieldGate => [226, 238, 214],
        Cell::HerbBed => [205, 222, 180],
        _ => WHITE,
    }
}

/// How far hatching reaches into a wall from its edge, in cells, and its
/// strokes per cell.
const HATCH_REACH: f64 = 0.25;
const HATCH_STROKES: f64 = 3.0;
/// How much of a stroke's width is drawn.
const HATCH_SOLID: f64 = 0.4;

/// How far along each side a corner is cut away, in cells.
const CORNER_CUT: f64 = 0.4;

/// A door's rectangle: half its length along the wall, half its width across,
/// and the half width of the line it covers, all in cells.
const DOOR_HALF_LENGTH: f64 = 0.28;
const DOOR_HALF_WIDTH: f64 = 0.16;
const DOOR_LINE: f64 = 0.05;
/// How thick the rectangle's outline is.
const DOOR_OUTLINE: f64 = 0.07;
/// A house door is a diamond on its line, this far from its middle each way.
const HOUSE_DOOR_REACH: f64 = 0.28;

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

    /// The shop signs in the box: where, and whose.
    pub fn signs(&self) -> impl Iterator<Item = ((i64, i64), BuildingKind)> + '_ {
        self.looks.iter().enumerate().filter_map(|(index, look)| {
            let CellLook::Sign(kind) = look else {
                return None;
            };
            let index = index as i64;
            let at = (
                self.min.0 + index % self.size.0,
                self.min.1 + index / self.size.0,
            );
            Some((at, *kind))
        })
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

/// The colour at a point in a cell (`u` east and `v` south of its centre, in
/// cells), if any. `line` is half a grid line's width.
pub fn colour_at(look: CellLook, u: f64, v: f64, line: f64) -> Option<[u8; 3]> {
    let floor = |colour: [u8; 3]| {
        // (A hair wider than a pixel: exactly a pixel can fall between samples.)
        if u.abs().max(v.abs()) > 0.5 - line * 1.01 {
            LINE
        } else {
            colour
        }
    };
    let along_across = |east_west: bool| {
        if east_west {
            (u.abs(), v.abs())
        } else {
            (v.abs(), u.abs())
        }
    };
    match look {
        CellLook::Empty => None,
        CellLook::Floor(colour) => Some(floor(colour)),
        CellLook::Wall(sides) => {
            cut_colour(sides, u, v).or_else(|| hatched(sides, u, v).then_some(HATCH))
        }
        CellLook::Sign(_) => Some(floor(WHITE)),
        CellLook::HouseDoor { east_west } => {
            let (along, across) = along_across(east_west);
            Some(if along + across <= HOUSE_DOOR_REACH {
                HOUSE_DOOR
            } else if across <= DOOR_LINE {
                HATCH
            } else {
                floor(WHITE)
            })
        }
        CellLook::Window { east_west, sides } => {
            let (_, across) = along_across(east_west);
            let signed = if east_west { v } else { u };
            Some(if across < DOOR_HALF_WIDTH - DOOR_OUTLINE {
                floor(WHITE)
            } else if across <= DOOR_HALF_WIDTH {
                HATCH
            } else {
                floor(sides[usize::from(signed > 0.0)])
            })
        }
        CellLook::Door { east_west } => {
            let (along, across) = along_across(east_west);
            let in_rectangle = along <= DOOR_HALF_LENGTH && across <= DOOR_HALF_WIDTH;
            Some(if in_rectangle {
                let inner = along <= DOOR_HALF_LENGTH - DOOR_OUTLINE
                    && across <= DOOR_HALF_WIDTH - DOOR_OUTLINE;
                if inner { DOOR } else { HATCH }
            } else if across <= DOOR_LINE {
                HATCH
            } else {
                floor(WHITE)
            })
        }
    }
}

/// The colour of a corner cut away from a wall, if the point is in one.
fn cut_colour(sides: Sides, u: f64, v: f64) -> Option<[u8; 3]> {
    let corner = match (u > 0.0, v > 0.0) {
        (true, false) => 0,
        (true, true) => 1,
        (false, true) => 2,
        (false, false) => 3,
    };
    let from_corner = (0.5 - u.abs()) + (0.5 - v.abs());
    sides.cut[corner].filter(|_| from_corner < CORNER_CUT)
}

/// Whether a point in a wall cell is on a stroke of the hatching along a side.
fn hatched(sides: Sides, u: f64, v: f64) -> bool {
    let near = 0.5 - HATCH_REACH;
    let by_edge = (sides.north && v < -near)
        || (sides.south && v > near)
        || (sides.west && u < -near)
        || (sides.east && u > near);
    by_edge && ((u + v) * HATCH_STROKES).rem_euclid(1.0) < HATCH_SOLID
}

/// Paints the level. Pixels further than `clip` from the middle of the picture
/// are left alone.
pub fn draw(canvas: &mut Canvas, view: &Projection, looks: &Looks, parchment: [u8; 3], clip: f32) {
    let line = 0.5 * view.units_per_pixel;
    // When turned, a few samples per pixel keep the lines from breaking up.
    let samples: &[(f32, f32)] = if view.is_square_on() {
        &[(0.5, 0.5)]
    } else {
        &[(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)]
    };
    let colour_of = |x: f32, y: f32| {
        let (grid_x, grid_y) = view.to_grid(x, y);
        let (cell_x, cell_y) = ((grid_x + 0.5).floor(), (grid_y + 0.5).floor());
        let look = looks.at(cell_x as i64, cell_y as i64);
        colour_at(look, grid_x - cell_x, grid_y - cell_y, line).unwrap_or(parchment)
    };
    for y in 0..canvas.height {
        for x in 0..canvas.width {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            if (px - view.middle.0).hypot(py - view.middle.1) > clip {
                continue;
            }
            let mut sum = [0u32; 3];
            for (offset_x, offset_y) in samples {
                let colour = colour_of(x as f32 + offset_x, y as f32 + offset_y);
                for (total, channel) in sum.iter_mut().zip(colour) {
                    *total += u32::from(channel);
                }
            }
            canvas.set(x, y, sum.map(|total| (total / samples.len() as u32) as u8));
        }
    }
}

#[cfg(test)]
mod tests {
    use shared::dungeon::{Level, RoomId};
    use shared::poi::PoiKind;
    use shared::protocol::LevelInfo;
    use shared::site::{BuildingLook, Rect, RoofKind, WallKind};

    use super::*;

    /// White between the lines; beyond them, what is known to be there.
    #[test]
    fn a_window_shows_what_it_looks_onto() {
        let mut dungeon = dungeon();
        dungeon.level.set(2, 1, Cell::Window);
        dungeon.level.set(2, 0, Cell::Ground);
        let look = look_of(&dungeon, 2, 1);
        assert_eq!(
            look,
            CellLook::Window {
                east_west: true,
                sides: [ground_colour(Cell::Ground), WHITE],
            }
        );
        let at = |v: f64| colour_at(look, 0.0, v, 0.0);
        assert_eq!(at(0.0), Some(WHITE));
        assert_eq!(at(-DOOR_HALF_WIDTH + 0.01), Some(HATCH));
        assert_eq!(at(DOOR_HALF_WIDTH - 0.01), Some(HATCH));
        assert_eq!(at(-0.4), Some(ground_colour(Cell::Ground)));
        assert_eq!(at(0.4), Some(WHITE));
    }

    /// A 5 × 5 level, all remembered: wall but for a floor in the middle.
    fn dungeon() -> CurrentDungeon {
        let mut dungeon = CurrentDungeon {
            level: Level::new(0, 0, 5, 5),
            ..Default::default()
        };
        dungeon.enter(LevelInfo {
            kind: PoiKind::Crypt,
            ruined: false,
            building: false,
            index: 0,
            levels: 1,
            basement: false,
            bounds: (0, 0, 5, 5),
            arrival: (2, 2),
            entering_site: true,
        });
        for y in 0..5 {
            for x in 0..5 {
                dungeon.remembered.insert(x, y);
            }
        }
        dungeon.level.set(2, 2, Cell::Floor);
        dungeon
    }

    fn wall_sides(dungeon: &CurrentDungeon, x: i64, y: i64) -> Sides {
        let CellLook::Wall(sides) = look_of(dungeon, x, y) else {
            panic!("not a wall");
        };
        sides
    }

    #[test]
    fn walls_are_hatched_towards_remembered_floor_only() {
        let mut dungeon = dungeon();
        assert_eq!(look_of(&dungeon, 2, 2), CellLook::Floor(WHITE));
        let south_only = Sides {
            south: true,
            ..Default::default()
        };
        assert_eq!(wall_sides(&dungeon, 2, 1), south_only);
        assert_eq!(wall_sides(&dungeon, 0, 0), Sides::default());

        dungeon.remembered = shared::dungeon::Sight::new(&dungeon.level);
        dungeon.remembered.insert(2, 1);
        assert_eq!(wall_sides(&dungeon, 2, 1), Sides::default());
        // Anything not remembered is bare parchment.
        assert_eq!(look_of(&dungeon, 0, 0), CellLook::Empty);
    }

    /// Walls meeting only at a corner, with a way between them, each lose that
    /// corner; walls with no way between them keep theirs.
    #[test]
    fn corners_are_cut_where_a_diagonal_goes_between_walls() {
        let mut dungeon = dungeon();
        dungeon.level.set(2, 2, Cell::Wall);
        dungeon.level.set(2, 1, Cell::Floor);
        dungeon.level.set(1, 2, Cell::Floor);
        let look = look_of(&dungeon, 2, 2);
        let cut = wall_sides(&dungeon, 2, 2).cut;
        assert_eq!(cut, [None, None, None, Some(WHITE)]);
        assert_eq!(wall_sides(&dungeon, 1, 1).cut[1], Some(WHITE));
        // The corner shows floor; the rest of the wall doesn't.
        assert_eq!(colour_at(look, -0.45, -0.45, 0.05), Some(WHITE));
        assert_ne!(colour_at(look, 0.45, 0.45, 0.05), Some(WHITE));
        dungeon.level.set(1, 2, Cell::Wall);
        assert_eq!(wall_sides(&dungeon, 2, 2).cut, [None; 4]);
    }

    #[test]
    fn touching_buildings_are_hatched_on_both_sides() {
        let mut dungeon = dungeon();
        dungeon.level.set(2, 2, Cell::Wall);
        dungeon.level.set_room(1, 1, Some(RoomId(1)));
        dungeon.level.set_room(2, 1, Some(RoomId(1)));
        dungeon.level.set_room(3, 1, Some(RoomId(2)));
        let east = Sides {
            east: true,
            ..Default::default()
        };
        let west = Sides {
            west: true,
            ..Default::default()
        };
        assert_eq!(wall_sides(&dungeon, 2, 1), east);
        assert_eq!(wall_sides(&dungeon, 3, 1), west);
        // Within a building, or walls in no building, nothing.
        assert_eq!(wall_sides(&dungeon, 1, 1), Sides::default());
        assert_eq!(wall_sides(&dungeon, 1, 3), Sides::default());
    }

    #[test]
    fn shops_show_a_sign_and_houses_a_door_across_their_wall() {
        let mut dungeon = dungeon();
        dungeon.level.set(2, 2, Cell::BuildingDoor);
        let building = |kind, faces| BuildingLook {
            id: 0,
            kind,
            walls: WallKind::Stone,
            roof: RoofKind::SlateTiles,
            rects: vec![Rect {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            }],
            door: (2, 2),
            faces,
        };
        dungeon.buildings = vec![building(BuildingKind::Bank, (0, 1))];
        assert!(matches!(
            look_of(&dungeon, 2, 2),
            CellLook::Sign(BuildingKind::Bank)
        ));
        dungeon.buildings = vec![building(BuildingKind::House, (1, 0))];
        assert_eq!(
            look_of(&dungeon, 2, 2),
            CellLook::HouseDoor { east_west: false }
        );
        // Nothing hatches towards a building's door, from its wall or across
        // the street.
        assert!(!wall_sides(&dungeon, 1, 2).east);
        dungeon.level.set(2, 3, Cell::Floor);
        assert_eq!(look_of(&dungeon, 2, 3), CellLook::Floor(WHITE));
    }

    #[test]
    fn colours_follow_the_owners_drawing() {
        let hatched_north = CellLook::Wall(Sides {
            north: true,
            ..Default::default()
        });
        assert_eq!(colour_at(CellLook::Empty, 0.0, 0.0, 0.05), None);
        assert_eq!(
            colour_at(CellLook::Floor(WHITE), 0.0, 0.0, 0.05),
            Some(WHITE)
        );
        assert_eq!(
            colour_at(CellLook::Floor(WHITE), 0.48, 0.0, 0.05),
            Some(LINE)
        );
        // A wall is parchment, with hatching in the quarter by its edge, and
        // some gaps in it.
        assert_eq!(colour_at(hatched_north, 0.0, 0.0, 0.05), None);
        let band: Vec<_> = (0..20)
            .map(|i| colour_at(hatched_north, i as f64 / 20.0 - 0.5, -0.4, 0.0))
            .collect();
        assert!(band.contains(&Some(HATCH)) && band.contains(&None));
        // A door runs along its wall.
        let door = CellLook::Door { east_west: true };
        assert_eq!(colour_at(door, 0.0, 0.0, 0.05), Some(DOOR));
        assert_eq!(colour_at(door, 0.4, 0.0, 0.05), Some(HATCH));
        assert_eq!(colour_at(door, 0.0, 0.4, 0.05), Some(WHITE));
    }

    #[test]
    fn walls_beside_a_dungeon_door_are_hatched_towards_it() {
        let mut dungeon = dungeon();
        dungeon.level.set(2, 2, Cell::Door);
        dungeon.level.set(2, 1, Cell::Floor);
        dungeon.level.set(2, 3, Cell::Floor);
        assert_eq!(look_of(&dungeon, 2, 2), CellLook::Door { east_west: true });
        assert!(wall_sides(&dungeon, 1, 2).east);
        assert!(wall_sides(&dungeon, 3, 2).west);
    }

    #[test]
    fn tiles_take_the_colour_of_their_ground() {
        let mut dungeon = dungeon();
        dungeon.level.set(2, 2, Cell::Water);
        let CellLook::Floor(water) = look_of(&dungeon, 2, 2) else {
            panic!("water should show");
        };
        assert!(water[2] > water[0]);
        dungeon.level.set(2, 2, Cell::Lava);
        let CellLook::Floor(lava) = look_of(&dungeon, 2, 2) else {
            panic!("lava should show");
        };
        assert!(lava[0] > lava[2]);
        dungeon.level.set(2, 2, Cell::Hole);
        assert_eq!(look_of(&dungeon, 2, 2), CellLook::Floor(HOLE));
        dungeon.level.set(2, 2, Cell::Cobblestone);
        assert!(matches!(look_of(&dungeon, 2, 2), CellLook::Floor(c) if c != WHITE));
    }

    #[test]
    fn looks_list_their_signs() {
        let mut dungeon = dungeon();
        dungeon.level.set(2, 2, Cell::BuildingDoor);
        dungeon.buildings = vec![BuildingLook {
            id: 0,
            kind: BuildingKind::Inn,
            walls: WallKind::Stone,
            roof: RoofKind::SlateTiles,
            rects: Vec::new(),
            door: (2, 2),
            faces: (0, 1),
        }];
        let looks = Looks::of(&dungeon, (1, 1), (3, 3));
        let signs: Vec<_> = looks.signs().collect();
        assert_eq!(signs, [((2, 2), BuildingKind::Inn)]);
        assert_eq!(looks.at(100, 100), CellLook::Empty);
    }
}
